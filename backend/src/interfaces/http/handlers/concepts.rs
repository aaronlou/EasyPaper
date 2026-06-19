use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::error::AppResult;
use crate::interfaces::http::AppState;
use crate::models::api::{ConceptExpansion, LlmProfileRequest};

/// POST /api/papers/:id/concepts/:concept_id/expand
pub async fn expand_concept(
    State(state): State<AppState>,
    Path((paper_id, concept_id)): Path<(Uuid, String)>,
    body: Option<Json<LlmProfileRequest>>,
) -> AppResult<Json<ConceptExpansion>> {
    let workflow = state
        .workflow
        .with_client_llm_profile(body.and_then(|Json(body)| body.llm_profile));
    let expansion = workflow.expand_concept(paper_id, concept_id).await?;
    Ok(Json(expansion))
}
