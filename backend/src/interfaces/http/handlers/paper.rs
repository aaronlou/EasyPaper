use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::domain::paper::PaperSummary;
use crate::error::AppResult;
use crate::interfaces::http::AppState;
use crate::models::api::{PaperDetail, UploadResponse};

/// GET /api/papers  —— 列出所有已上传的论文
pub async fn list_papers(State(state): State<AppState>) -> AppResult<Json<Vec<PaperSummary>>> {
    let papers = state.workflow.list_papers().await?;
    Ok(Json(papers))
}

/// GET /api/papers/:id  —— 获取单篇论文 + 解读
pub async fn get_paper(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<PaperDetail>> {
    let (paper, interpretation) = state.workflow.get_paper_detail(id).await?;

    Ok(Json(PaperDetail {
        paper: paper.into(),
        interpretation,
    }))
}

/// POST /api/papers/:id/retry —— 重新发起失败论文的 AI 解读
pub async fn retry_interpretation(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<UploadResponse>> {
    let paper = state.workflow.retry_interpretation(id).await?;
    Ok(Json(UploadResponse { paper }))
}
