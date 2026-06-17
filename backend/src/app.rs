use std::sync::Arc;

use tokio::sync::RwLock;

use crate::application::paper_workflow::PaperWorkflow;
use crate::config::Config;
use crate::domain::repositories::SharedPaperRepository;
use crate::domain::research::SharedResearchSource;
use crate::infrastructure::search::{WebSearchClient, WebSearchConfig};
use crate::interfaces::http::{self, AppState};
use crate::llm::{Interpreter, LlmClient};
use crate::store::SqliteStore;

/// 构建完整 Router
pub async fn build(config: Config) -> anyhow::Result<axum::Router> {
    // 初始化存储
    let store = SqliteStore::new(&config.db_path).await?;
    let store: SharedPaperRepository = Arc::new(store);

    // 初始化 LLM
    let llm = LlmClient::new(
        config.openai_api_key.clone(),
        config.openai_base_url.clone(),
        config.openai_model.clone(),
    );
    let interpreter = Interpreter::new(llm.clone());
    let research: SharedResearchSource =
        Arc::new(WebSearchClient::new(WebSearchConfig::from(&config)));

    let progress = Arc::new(RwLock::new(std::collections::HashMap::new()));
    let workflow = PaperWorkflow::new(
        store.clone(),
        llm.clone(),
        interpreter.clone(),
        research.clone(),
        progress.clone(),
    );

    let state = AppState { workflow };

    Ok(http::router(&config, state))
}
