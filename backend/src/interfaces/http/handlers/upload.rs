use axum::{
    Json,
    extract::{Multipart, State},
};

use crate::error::{AppError, AppResult};
use crate::interfaces::http::AppState;
use crate::models::api::UploadResponse;

/// 最大 PDF 大小：50 MB
pub const MAX_PDF_SIZE: usize = 50 * 1024 * 1024;

/// POST /api/papers  —— 接收 PDF 上传
///
/// HTTP 层只负责协议适配：解析 multipart、做尺寸校验、调用应用用例。
pub async fn upload_paper(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> AppResult<Json<UploadResponse>> {
    let mut filename = None;
    let mut pdf_bytes: Option<Vec<u8>> = None;

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
                return Err(AppError::PayloadTooLarge {
                    limit: MAX_PDF_SIZE,
                });
            }

            filename = Some(fname);
            pdf_bytes = Some(data.to_vec());
        }
    }

    let filename = filename.ok_or_else(|| AppError::BadRequest("缺少 file 字段".into()))?;
    let pdf_bytes = pdf_bytes.ok_or_else(|| AppError::BadRequest("文件内容为空".into()))?;

    tracing::info!(filename = %filename, size = pdf_bytes.len(), "收到 PDF 上传");

    let paper = state.workflow.upload_paper(filename, pdf_bytes).await?;

    Ok(Json(UploadResponse { paper }))
}
