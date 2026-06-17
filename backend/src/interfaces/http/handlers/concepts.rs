use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::error::AppResult;
use crate::interfaces::http::AppState;
use crate::models::api::ConceptExpansion;

/// POST /api/papers/:id/concepts/:concept_id/expand
pub async fn expand_concept(
    State(state): State<AppState>,
    Path((paper_id, concept_id)): Path<(Uuid, String)>,
) -> AppResult<Json<ConceptExpansion>> {
    let expansion = state.workflow.expand_concept(paper_id, concept_id).await?;
    Ok(Json(expansion))
}
