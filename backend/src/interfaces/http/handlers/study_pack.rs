use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::domain::study_pack::StudyPack;
use crate::error::AppResult;
use crate::interfaces::http::AppState;
use crate::models::api::LlmProfileRequest;

/// POST /api/papers/:id/study-pack
pub async fn get_or_generate_study_pack(
    State(state): State<AppState>,
    Path(paper_id): Path<Uuid>,
    body: Option<Json<LlmProfileRequest>>,
) -> AppResult<Json<StudyPack>> {
    let pack = state
        .workflow
        .get_or_generate_study_pack(paper_id, body.and_then(|Json(body)| body.llm_profile))
        .await?;
    Ok(Json(pack))
}
