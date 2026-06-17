use std::sync::Arc;

use async_trait::async_trait;

/// 外部研究资料检索结果。
///
/// 领域层只关心“可引用的研究线索”，不关心它来自 Tavily、SearXNG 还是其他服务。
#[derive(Debug, Clone)]
pub struct WebSearchResult {
    pub title: String,
    pub url: Option<String>,
    pub snippet: String,
}

/// 概念深潜用例依赖的外部研究端口。
#[async_trait]
pub trait ResearchSource: Send + Sync + 'static {
    async fn search(&self, query: &str) -> Vec<WebSearchResult>;
}

pub type SharedResearchSource = Arc<dyn ResearchSource>;
