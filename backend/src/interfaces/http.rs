use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
use uuid::Uuid;

use crate::application::paper_workflow::PaperWorkflow;
use crate::config::Config;
use crate::models::api::ProgressInfo;

pub mod handlers;

/// HTTP handler 共享状态。
#[derive(Clone)]
pub struct AppState {
    pub workflow: PaperWorkflow,
}

impl AppState {
    /// 更新某篇论文的最新进度
    pub async fn update_progress(&self, paper_id: Uuid, info: ProgressInfo) {
        self.workflow.update_progress(paper_id, info).await;
    }

    /// 获取某篇论文的最新进度
    pub async fn get_progress(&self, paper_id: Uuid) -> Option<ProgressInfo> {
        self.workflow.get_progress(paper_id).await
    }
}

/// 构建 HTTP 入口适配器。
pub fn router(config: &Config, state: AppState) -> Router {
    Router::new()
        .nest("/api", api_router(state))
        .fallback_service(ServeDir::new(&config.static_dir))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
}

fn api_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/papers", post(handlers::upload::upload_paper))
        .route("/papers", get(handlers::paper::list_papers))
        .route("/papers/{id}", get(handlers::paper::get_paper))
        .route(
            "/papers/{id}/retry",
            post(handlers::paper::retry_interpretation),
        )
        .route(
            "/papers/{id}/progress",
            get(handlers::progress::get_progress),
        )
        .route(
            "/papers/{id}/concepts/{concept_id}/expand",
            post(handlers::concepts::expand_concept),
        )
        .layer(DefaultBodyLimit::max(
            handlers::upload::MAX_PDF_SIZE + 1024 * 1024,
        ))
        .with_state(state)
}
