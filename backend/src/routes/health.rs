use axum::Json;
use serde::Serialize;

use crate::error::AppResult;

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
    /// LLM 是否已配置（决定能否解读）
    llm_configured: bool,
}

/// GET /api/health
pub async fn health(
    axum::extract::State(state): axum::extract::State<crate::app::AppState>,
) -> AppResult<Json<HealthResponse>> {
    Ok(Json(HealthResponse {
        status: "ok",
        service: "easypaper-backend",
        version: env!("CARGO_PKG_VERSION"),
        llm_configured: state.llm.is_configured(),
    }))
}
