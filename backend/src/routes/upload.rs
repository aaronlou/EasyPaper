use axum::{
    extract::{Multipart, State},
    Json,
};
use uuid::Uuid;

use crate::{
    app::AppState,
    error::{AppError, AppResult},
    models::api::UploadResponse,
    models::paper::{Paper, PaperStatus},
};

/// 最大 PDF 大小：50 MB
const MAX_PDF_SIZE: usize = 50 * 1024 * 1024;

/// POST /api/papers  —— 接收 PDF 上传
///
/// 同步流程：
///   1. 接收 multipart 文件
///   2. 提取文本 + 元信息
///   3. 存入 SQLite，状态 Uploaded
///   4. 异步触发 LLM 解读（不阻塞响应）
///
/// 返回 paper summary，前端据此轮询 /api/papers/:id/progress 或直接拉详情
pub async fn upload_paper(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> AppResult<Json<UploadResponse>> {
    let mut filename = None;
    let mut pdf_bytes: Option<Vec<u8>> = None;

    // 解析 multipart
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("multipart 解析失败: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        let file_name = field.file_name().map(|s| s.to_string());

        if name == "file" {
            let fname = file_name.unwrap_or_else(|| "paper.pdf".to_string());
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(format!("读取文件失败: {e}")))?;

            if data.len() > MAX_PDF_SIZE {
                return Err(AppError::PayloadTooLarge { limit: MAX_PDF_SIZE });
            }

            filename = Some(fname);
            pdf_bytes = Some(data.to_vec());
        }
    }

    let filename = filename.ok_or_else(|| AppError::BadRequest("缺少 file 字段".into()))?;
    let pdf_bytes = pdf_bytes.ok_or_else(|| AppError::BadRequest("文件内容为空".into()))?;

    tracing::info!(filename = %filename, size = pdf_bytes.len(), "收到 PDF 上传");

    // 提取文本
    let extracted = crate::pdf::extract_text(&pdf_bytes)
        .map_err(|e| {
            tracing::warn!(filename = %filename, "PDF 提取失败: {e}");
            e
        })?;

    tracing::info!(
        filename = %filename,
        title = %extracted.title,
        char_count = extracted.full_text.chars().count(),
        "PDF 文本提取完成"
    );

    let now = chrono::Utc::now().to_rfc3339();
    let paper = Paper {
        id: Uuid::new_v4(),
        filename,
        title: extracted.title,
        authors: extracted.authors,
        full_text: extracted.full_text,
        char_count: 0, // 下面填
        status: PaperStatus::Uploaded,
        created_at: now.clone(),
        completed_at: None,
    };
    let paper = Paper {
        char_count: paper.full_text.chars().count(),
        ..paper
    };

    // 持久化
    state
        .store
        .insert_paper(&paper)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // 异步触发解读（fire and forget，不阻塞响应）
    if state.llm.is_configured() {
        spawn_interpretation(state.clone(), paper.clone());
    } else {
        tracing::warn!("LLM 未配置，跳过解读。论文已保存，可在配置 OPENAI_API_KEY 后手动触发。");
    }

    Ok(Json(UploadResponse {
        paper: paper.into(),
    }))
}

/// 异步触发解读任务
fn spawn_interpretation(state: AppState, paper: Paper) {
    let store = state.store.clone();
    let interpreter = state.interpreter.clone();
    let paper_id = paper.id;
    let title = paper.title.clone();
    let text = paper.full_text.clone();

    tokio::spawn(async move {
        // 标记为处理中
        if let Err(e) = store
            .update_status(paper_id, PaperStatus::Processing)
            .await
        {
            tracing::error!(paper_id = %paper_id, "更新状态失败: {e}");
            return;
        }

        tracing::info!(paper_id = %paper_id, "开始 LLM 解读");

        match interpreter.interpret(paper_id, &title, &text).await {
            Ok(interp) => {
                if let Err(e) = store.save_interpretation(&interp).await {
                    tracing::error!(paper_id = %paper_id, "保存解读失败: {e}");
                    let _ = store.update_status(paper_id, PaperStatus::Failed).await;
                    return;
                }
                if let Err(e) = store.update_status(paper_id, PaperStatus::Completed).await {
                    tracing::error!(paper_id = %paper_id, "更新状态失败: {e}");
                }
                tracing::info!(paper_id = %paper_id, "解读完成 ✓");
            }
            Err(e) => {
                tracing::error!(paper_id = %paper_id, "解读失败: {e}");
                let _ = store.update_status(paper_id, PaperStatus::Failed).await;
            }
        }
    });
}
