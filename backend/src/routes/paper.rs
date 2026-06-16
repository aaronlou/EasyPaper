use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::app::AppState;
use crate::error::{AppError, AppResult};
use crate::models::api::PaperDetail;
use crate::models::paper::{PaperStatus, PaperSummary};

/// GET /api/papers  —— 列出所有已上传的论文
pub async fn list_papers(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<PaperSummary>>> {
    let papers = state
        .store
        .list_papers()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(Json(papers))
}

/// GET /api/papers/:id  —— 获取单篇论文 + 解读
pub async fn get_paper(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<PaperDetail>> {
    let paper = state
        .store
        .get_paper(id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("论文 {id} 不存在")))?;

    let interpretation = if matches!(paper.status, PaperStatus::Completed) {
        state
            .store
            .get_interpretation(id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
    } else {
        None
    };

    Ok(Json(PaperDetail {
        paper: paper.into(),
        interpretation,
    }))
}
