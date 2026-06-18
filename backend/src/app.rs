use std::sync::Arc;

use tokio::sync::RwLock;

use crate::application::paper_workflow::PaperWorkflow;
use crate::application::ports::{SharedConceptExpansionCache, SharedPdfExtractor};
use crate::config::Config;
use crate::domain::repositories::SharedPaperRepository;
use crate::domain::research::SharedResearchSource;
use crate::infrastructure::search::{WebSearchClient, WebSearchConfig};
use crate::interfaces::http::{self, AppState};
use crate::llm::{Interpreter, LlmClient};
use crate::pdf::PdfExtractAdapter;
use crate::store::SqliteStore;

/// 构建完整 Router
pub async fn build(config: Config) -> anyhow::Result<axum::Router> {
    // 初始化存储
    let store = SqliteStore::new(&config.db_path).await?;
    let store = Arc::new(store);
    let papers: SharedPaperRepository = store.clone();
    let concept_expansions: SharedConceptExpansionCache = store.clone();

    // 初始化 LLM
    let llm = LlmClient::new(
        config.openai_api_key.clone(),
        config.openai_base_url.clone(),
        config.openai_model.clone(),
    );
    let interpreter = Interpreter::new(llm.clone());
    let research: SharedResearchSource =
        Arc::new(WebSearchClient::new(WebSearchConfig::from(&config)));
    let pdfs: SharedPdfExtractor = Arc::new(PdfExtractAdapter);

    let progress = Arc::new(RwLock::new(std::collections::HashMap::new()));
    let workflow = PaperWorkflow::new(
        papers.clone(),
        pdfs.clone(),
        concept_expansions.clone(),
        config.concept_prewarm_limit,
        config.concept_cache_ttl_days,
        llm.clone(),
        interpreter.clone(),
        research.clone(),
        progress.clone(),
    );
    workflow.recover_interrupted_work().await?;

    let state = AppState { workflow };

    Ok(http::router(&config, state))
}
