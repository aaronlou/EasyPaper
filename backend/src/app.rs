use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};

use crate::config::Config;
use crate::llm::{Interpreter, LlmClient};
use crate::store::{PaperStore, SqliteStore};

/// 全局共享状态，注入到所有 handler
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub store: Arc<dyn PaperStore>,
    pub llm: LlmClient,
    pub interpreter: Interpreter,
}

/// 构建完整 Router
pub async fn build(config: Config) -> anyhow::Result<Router> {
    // 初始化存储
    let store = SqliteStore::new(&config.db_path).await?;
    let store: Arc<dyn PaperStore> = Arc::new(store);

    // 初始化 LLM
    let llm = LlmClient::new(
        config.openai_api_key.clone(),
        config.openai_base_url.clone(),
        config.openai_model.clone(),
    );
    let interpreter = Interpreter::new(llm.clone());

    let state = AppState {
        config: config.clone(),
        store,
        llm,
        interpreter,
    };

    // API 路由
    let api = Router::new()
        .route("/health", get(crate::routes::health))
        .route("/papers", post(crate::routes::upload::upload_paper))
        .route("/papers", get(crate::routes::paper::list_papers))
        .route("/papers/{id}", get(crate::routes::paper::get_paper))
        .with_state(state);

    // 主 Router：API + 静态文件兜底
    let app = Router::new()
        .nest("/api", api)
        .fallback_service(ServeDir::new(&config.static_dir))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive()); // 开发期用宽松 CORS，生产收紧

    Ok(app)
}
