use async_trait::async_trait;
use serde::Deserialize;

use crate::config::Config;
use crate::domain::research::{ResearchSource, WebSearchResult};
use crate::error::{AppError, AppResult};

/// Web 检索基础设施配置。
#[derive(Debug, Clone)]
pub struct WebSearchConfig {
    pub url: Option<String>,
    pub api_key: Option<String>,
    pub max_results: usize,
}

impl From<&Config> for WebSearchConfig {
    fn from(config: &Config) -> Self {
        Self {
            url: config.web_search_url.clone(),
            api_key: config.web_search_api_key.clone(),
            max_results: config.web_search_max_results,
        }
    }
}

/// 外部研究检索适配器。支持 Tavily POST 或 SearXNG JSON GET。
#[derive(Clone)]
pub struct WebSearchClient {
    config: WebSearchConfig,
    http: reqwest::Client,
}

impl WebSearchClient {
    pub fn new(config: WebSearchConfig) -> Self {
        Self {
            config,
            http: reqwest::Client::new(),
        }
    }

    async fn search_tavily(&self, url: &str, query: &str) -> AppResult<Vec<WebSearchResult>> {
        let api_key = self.config.api_key.as_ref().ok_or_else(|| {
            AppError::BadRequest("Tavily 检索需要 EASYPAPER_WEB_SEARCH_API_KEY".into())
        })?;

        let response = self
            .http
            .post(url)
            .json(&serde_json::json!({
                "api_key": api_key,
                "query": query,
                "search_depth": "basic",
                "max_results": self.config.max_results,
                "include_answer": false,
                "include_raw_content": false
            }))
            .send()
            .await
            .map_err(|e| AppError::LlmCall(format!("Web 检索请求失败: {e}")))?;

        if !response.status().is_success() {
            return Err(AppError::LlmCall(format!(
                "Web 检索返回 {}",
                response.status()
            )));
        }

        let parsed: TavilyResponse = response
            .json()
            .await
            .map_err(|e| AppError::LlmCall(format!("Web 检索响应解析失败: {e}")))?;

        Ok(parsed
            .results
            .into_iter()
            .take(self.config.max_results)
            .map(|item| WebSearchResult {
                title: item.title,
                url: item.url,
                snippet: item.content,
            })
            .filter(|item| !item.title.is_empty() || !item.snippet.is_empty())
            .collect())
    }

    async fn search_searxng(&self, url: &str, query: &str) -> AppResult<Vec<WebSearchResult>> {
        let response = self
            .http
            .get(url)
            .query(&[
                ("q", query),
                ("format", "json"),
                ("language", "en"),
                ("safesearch", "1"),
            ])
            .send()
            .await
            .map_err(|e| AppError::LlmCall(format!("Web 检索请求失败: {e}")))?;

        if !response.status().is_success() {
            return Err(AppError::LlmCall(format!(
                "Web 检索返回 {}",
                response.status()
            )));
        }

        let parsed: SearxngResponse = response
            .json()
            .await
            .map_err(|e| AppError::LlmCall(format!("Web 检索响应解析失败: {e}")))?;

        Ok(parsed
            .results
            .into_iter()
            .take(self.config.max_results)
            .map(|item| WebSearchResult {
                title: item.title,
                url: item.url,
                snippet: item.content.or(item.snippet).unwrap_or_default(),
            })
            .filter(|item| !item.title.is_empty() || !item.snippet.is_empty())
            .collect())
    }
}

#[async_trait]
impl ResearchSource for WebSearchClient {
    async fn search(&self, query: &str) -> Vec<WebSearchResult> {
        let Some(url) = self.config.url.as_ref() else {
            return Vec::new();
        };

        let result = if url.contains("tavily.com") || self.config.api_key.is_some() {
            self.search_tavily(url, query).await
        } else {
            self.search_searxng(url, query).await
        };

        match result {
            Ok(results) => results,
            Err(err) => {
                tracing::warn!(query = %query, "概念外部检索失败: {err}");
                Vec::new()
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct TavilyResponse {
    #[serde(default)]
    results: Vec<TavilyResult>,
}

#[derive(Debug, Deserialize)]
struct TavilyResult {
    #[serde(default)]
    title: String,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    content: String,
}

#[derive(Debug, Deserialize)]
struct SearxngResponse {
    #[serde(default)]
    results: Vec<SearxngResult>,
}

#[derive(Debug, Deserialize)]
struct SearxngResult {
    #[serde(default)]
    title: String,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    snippet: Option<String>,
}
