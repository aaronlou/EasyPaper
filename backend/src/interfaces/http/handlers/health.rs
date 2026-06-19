use axum::{Json, extract::State};
use serde::Serialize;

use crate::error::AppResult;
use crate::interfaces::http::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
    /// LLM 是否已配置（决定能否解读）
    llm_configured: bool,
    /// 已配置 API Key 的 LLM provider。
    llm_providers: Vec<String>,
}

/// GET /api/health
pub async fn health(State(state): State<AppState>) -> AppResult<Json<HealthResponse>> {
    Ok(Json(HealthResponse {
        status: "ok",
        service: "easypaper-backend",
        version: env!("CARGO_PKG_VERSION"),
        llm_configured: state.workflow.llm_is_configured(),
        llm_providers: state
            .workflow
            .configured_llm_providers()
            .into_iter()
            .map(str::to_string)
            .collect(),
    }))
}
