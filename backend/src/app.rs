use std::sync::Arc;

use tokio::sync::{Mutex, RwLock};

use crate::application::paper_workflow::{PaperWorkflow, PaperWorkflowDeps};
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
    let llm = LlmClient::from_profile(config.llm_profile.clone());
    let interpreter = Interpreter::new(llm.clone());
    let research: SharedResearchSource =
        Arc::new(WebSearchClient::new(WebSearchConfig::from(&config)));
    let pdfs: SharedPdfExtractor = Arc::new(PdfExtractAdapter);

    let progress = Arc::new(RwLock::new(std::collections::HashMap::new()));
    let study_pack_in_flight = Arc::new(Mutex::new(std::collections::HashSet::new()));
    let workflow = PaperWorkflow::new(PaperWorkflowDeps {
        papers: papers.clone(),
        pdfs: pdfs.clone(),
        concept_expansions: concept_expansions.clone(),
        concept_prewarm_limit: config.concept_prewarm_limit,
        concept_cache_ttl_days: config.concept_cache_ttl_days,
        concept_prewarm_concurrency: config.concept_prewarm_concurrency,
        llm: llm.clone(),
        interpreter: interpreter.clone(),
        research: research.clone(),
        progress: progress.clone(),
        study_pack_in_flight,
    });
    workflow.recover_interrupted_work().await?;

    let state = AppState { workflow };

    Ok(http::router(&config, state))
}
