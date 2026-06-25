use serde_json::Value;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use crate::error::{AppError, AppResult};

const DEFAULT_PROVIDER_ID: &str = "default";
const DEFAULT_ROLE: &str = "default";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LlmRole {
    Default,
    Reader,
    Specialist,
    Concept,
    Repair,
    Study,
    Literature,
    Translation,
}

impl LlmRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => DEFAULT_ROLE,
            Self::Reader => "reader",
            Self::Specialist => "specialist",
            Self::Concept => "concept",
            Self::Repair => "repair",
            Self::Study => "study",
            Self::Literature => "literature",
            Self::Translation => "translation",
        }
    }
}

impl std::fmt::Display for LlmRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for LlmRole {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            DEFAULT_ROLE => Ok(Self::Default),
            "reader" | "readers" => Ok(Self::Reader),
            "specialist" | "specialists" => Ok(Self::Specialist),
            "concept" | "concepts" | "expand" | "expansion" => Ok(Self::Concept),
            "repair" | "json_repair" | "json-repair" => Ok(Self::Repair),
            "study" | "study_pack" | "study-pack" => Ok(Self::Study),
            "literature" | "literature_review" | "literature-review" => Ok(Self::Literature),
            "translation" | "translate" => Ok(Self::Translation),
            other => Err(anyhow::anyhow!("未知 LLM role: {other}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LlmProviderConfig {
    pub id: String,
    pub api_key: Option<String>,
    pub base_url: String,
    pub model: String,
    pub temperature: f32,
    pub prefer_responses_api: bool,
}

impl LlmProviderConfig {
    pub fn default_compatible(api_key: Option<String>, base_url: String, model: String) -> Self {
        let prefer_responses_api = base_url.contains("api.openai.com");
        Self {
            id: DEFAULT_PROVIDER_ID.to_string(),
            api_key,
            base_url,
            model,
            temperature: 0.4,
            prefer_responses_api,
        }
    }

    fn is_configured(&self) -> bool {
        self.api_key.as_ref().is_some_and(|key| !key.is_empty())
    }
}

#[derive(Debug, Clone)]
pub struct LlmProfileConfig {
    pub providers: Vec<LlmProviderConfig>,
    pub role_routes: HashMap<LlmRole, Vec<String>>,
}

impl LlmProfileConfig {
    pub fn single_provider(provider: LlmProviderConfig) -> Self {
        let mut role_routes = HashMap::new();
        role_routes.insert(LlmRole::Default, vec![provider.id.clone()]);
        Self {
            providers: vec![provider],
            role_routes,
        }
    }
}

#[derive(Clone)]
pub struct LlmClient {
    http: reqwest::Client,
    providers: HashMap<String, LlmProviderConfig>,
    role_routes: HashMap<LlmRole, Vec<String>>,
    default_route: Vec<String>,
}

impl LlmClient {
    pub fn new(api_key: Option<String>, base_url: String, model: String) -> Self {
        Self::from_profile(LlmProfileConfig::single_provider(
            LlmProviderConfig::default_compatible(api_key, base_url, model),
        ))
    }

    pub fn from_profile(profile: LlmProfileConfig) -> Self {
        let providers = profile
            .providers
            .into_iter()
            .map(|provider| (provider.id.clone(), provider))
            .collect::<HashMap<_, _>>();
        let default_route = profile
            .role_routes
            .get(&LlmRole::Default)
            .cloned()
            .unwrap_or_else(|| providers.keys().cloned().collect());

        Self {
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("failed to build reqwest client"),
            providers,
            role_routes: profile.role_routes,
            default_route,
        }
    }

    pub fn is_configured(&self) -> bool {
        self.providers
            .values()
            .any(LlmProviderConfig::is_configured)
    }

    pub fn configured_providers(&self) -> Vec<&str> {
        let mut providers = self
            .providers
            .values()
            .filter(|provider| provider.is_configured())
            .map(|provider| provider.id.as_str())
            .collect::<Vec<_>>();
        providers.sort_unstable();
        providers
    }

    pub fn cache_namespace(&self) -> String {
        let mut hasher = DefaultHasher::new();
        let mut providers = self
            .providers
            .values()
            .map(|provider| {
                (
                    provider.id.as_str(),
                    provider.base_url.as_str(),
                    provider.model.as_str(),
                    provider.temperature.to_bits(),
                    provider.prefer_responses_api,
                )
            })
            .collect::<Vec<_>>();
        providers.sort_unstable_by(|a, b| a.0.cmp(b.0));
        providers.hash(&mut hasher);
        let mut routes = self
            .role_routes
            .iter()
            .map(|(role, route)| (role.as_str(), route.as_slice()))
            .collect::<Vec<_>>();
        routes.sort_unstable_by(|a, b| a.0.cmp(b.0));
        routes.hash(&mut hasher);
        format!(
            "concept-expansion-v4-teaching-flow-{:016x}",
            hasher.finish()
        )
    }

    /// 调用 LLM 并期望返回 JSON（解析成 serde_json::Value）
    pub async fn call_json(&self, system: &str, user: &str) -> AppResult<Value> {
        self.call_json_with_role(LlmRole::Default, system, user)
            .await
    }

    pub async fn call_json_with_role(
        &self,
        role: LlmRole,
        system: &str,
        user: &str,
    ) -> AppResult<Value> {
        let raw = self.call_text_with_role(role, system, user, true).await?;
        match parse_json_lenient(&raw) {
            Ok(value) => Ok(value),
            Err(first_error) => {
                let mut raw_for_repair = raw;
                let mut parse_error = first_error;
                if is_probably_truncated_json(&raw_for_repair, &parse_error) {
                    tracing::warn!(
                        role = %role,
                        "LLM JSON 看起来被截断，使用紧凑约束重试: {parse_error}; excerpt={}",
                        error_excerpt(&raw_for_repair, parse_error.line(), parse_error.column())
                    );
                    let compact_user = compact_json_retry_prompt(user);
                    match self
                        .call_text_with_role(role, system, &compact_user, true)
                        .await
                    {
                        Ok(compact_raw) => match parse_json_lenient(&compact_raw) {
                            Ok(value) => return Ok(value),
                            Err(compact_error) => {
                                tracing::warn!(
                                    role = %role,
                                    "紧凑 JSON 重试仍解析失败，尝试修复: {compact_error}; excerpt={}",
                                    error_excerpt(
                                        &compact_raw,
                                        compact_error.line(),
                                        compact_error.column()
                                    )
                                );
                                raw_for_repair = compact_raw;
                                parse_error = compact_error;
                            }
                        },
                        Err(err) => {
                            tracing::warn!(
                                role = %role,
                                "紧凑 JSON 重试调用失败，回退到原始输出修复: {err}"
                            );
                        }
                    }
                }
                tracing::warn!(
                    role = %role,
                    "LLM JSON 解析失败，尝试自动修复: {parse_error}; excerpt={}",
                    error_excerpt(&raw_for_repair, parse_error.line(), parse_error.column())
                );
                let repaired = self
                    .repair_json(system, &raw_for_repair, &parse_error)
                    .await?;
                parse_json_lenient(&repaired).map_err(|second_error| {
                    AppError::InvalidLlmOutput(format!(
                        "{second_error}; repair_failed_after={parse_error}; excerpt={}",
                        error_excerpt(&repaired, second_error.line(), second_error.column())
                    ))
                })
            }
        }
    }

    /// 调用 LLM 返回纯文本。
    pub async fn call_text(&self, system: &str, user: &str) -> AppResult<String> {
        self.call_text_with_role(LlmRole::Default, system, user, false)
            .await
    }

    pub async fn call_text_with_role(
        &self,
        role: LlmRole,
        system: &str,
        user: &str,
        wants_json: bool,
    ) -> AppResult<String> {
        let mut errors = Vec::new();
        for provider in self.providers_for_role(role) {
            match self
                .call_text_with_provider(provider, system, user, wants_json)
                .await
            {
                Ok(content) => return Ok(content),
                Err(err) => {
                    tracing::warn!(
                        role = %role,
                        provider = %provider.id,
                        "LLM provider 调用失败，尝试 fallback: {err}"
                    );
                    errors.push(format!("{}: {err}", provider.id));
                }
            }
        }

        if errors.is_empty() {
            Err(AppError::LlmNotConfigured)
        } else {
            Err(AppError::LlmCall(format!(
                "所有 LLM provider 均失败: {}",
                errors.join(" | ")
            )))
        }
    }

    async fn call_text_with_provider(
        &self,
        provider: &LlmProviderConfig,
        system: &str,
        user: &str,
        wants_json: bool,
    ) -> AppResult<String> {
        if !provider.is_configured() {
            return Err(AppError::LlmNotConfigured);
        }

        let mut last_error = None;
        for attempt in 1..=2 {
            match self
                .call_text_once(provider, system, user, wants_json)
                .await
            {
                Ok(content) => return Ok(content),
                Err(err) if attempt < 2 && err.is_retryable_llm_error() => {
                    tracing::warn!(
                        provider = %provider.id,
                        "LLM 调用失败，将在同 provider 内重试一次: {err}"
                    );
                    last_error = Some(err);
                    tokio::time::sleep(std::time::Duration::from_millis(700)).await;
                }
                Err(err) => return Err(err),
            }
        }

        Err(last_error.unwrap_or_else(|| AppError::LlmCall("LLM 调用失败".to_string())))
    }

    async fn call_text_once(
        &self,
        provider: &LlmProviderConfig,
        system: &str,
        user: &str,
        wants_json: bool,
    ) -> AppResult<String> {
        let api_key = provider
            .api_key
            .as_ref()
            .ok_or(AppError::LlmNotConfigured)?;

        let url = if provider.prefer_responses_api {
            format!("{}/responses", provider.base_url.trim_end_matches('/'))
        } else {
            format!(
                "{}/chat/completions",
                provider.base_url.trim_end_matches('/')
            )
        };

        let body = if provider.prefer_responses_api {
            serde_json::json!({
                "model": provider.model,
                "input": [
                    { "role": "developer", "content": system },
                    { "role": "user", "content": user }
                ],
            })
        } else {
            let mut body = serde_json::json!({
                "model": provider.model,
                "messages": [
                    { "role": "system", "content": system },
                    { "role": "user", "content": user }
                ],
                "temperature": provider.temperature,
            });
            if wants_json {
                body["response_format"] = serde_json::json!({ "type": "json_object" });
            }
            body
        };

        let resp = self
            .http
            .post(&url)
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::LlmCall(format!("HTTP 请求失败: {e}")))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::LlmCall(format!(
                "LLM 返回 {status}: {}",
                truncate(&text, 500)
            )));
        }

        let response_text = resp
            .text()
            .await
            .map_err(|e| AppError::LlmCall(format!("响应体读取失败: {e}")))?;

        let resp_json: Value = serde_json::from_str(&response_text).map_err(|e| {
            AppError::LlmCall(format!(
                "响应 JSON 解析失败: {e}; body={}",
                truncate(&response_text, 800)
            ))
        })?;

        Ok(if provider.prefer_responses_api {
            parse_responses_content(&resp_json)
        } else {
            parse_chat_completion_content(&resp_json)
        })
    }

    async fn repair_json(
        &self,
        original_system: &str,
        raw_json: &str,
        error: &serde_json::Error,
    ) -> AppResult<String> {
        let repair_system = "你是 JSON 修复器。你只修复语法错误，不改写字段含义，不新增解释文字。必须只输出一段严格合法的 JSON。";
        let repair_user = format!(
            "下面这段 JSON 由另一个模型生成，但解析失败。\n\
             原始任务约束：\n{original_system}\n\n\
             解析错误：{error}\n\n\
             请只修复 JSON 语法，保持原有结构和内容，不要输出 markdown，不要输出解释。\n\n\
             待修复 JSON：\n{raw_json}",
        );

        self.call_text_with_role(LlmRole::Repair, repair_system, &repair_user, false)
            .await
    }

    fn providers_for_role(&self, role: LlmRole) -> Vec<&LlmProviderConfig> {
        let route = self
            .role_routes
            .get(&role)
            .or_else(|| self.role_routes.get(&LlmRole::Default))
            .unwrap_or(&self.default_route);
        let mut seen = HashSet::new();
        route
            .iter()
            .chain(self.default_route.iter())
            .filter_map(|id| {
                if !seen.insert(id) {
                    return None;
                }
                self.providers.get(id)
            })
            .filter(|provider| provider.is_configured())
            .collect()
    }
}

fn parse_responses_content(resp_json: &Value) -> String {
    resp_json
        .get("output")
        .and_then(|o| o.get(0))
        .and_then(|o| o.get("content"))
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string()
}

fn parse_chat_completion_content(resp_json: &Value) -> String {
    resp_json
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string()
}

/// 容错 JSON 解析：容忍 ```json ... ``` 代码块包裹
fn parse_json_lenient(raw: &str) -> Result<Value, serde_json::Error> {
    let trimmed = raw.trim();
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        return serde_json::from_str(trimmed);
    }
    if let Some(start) = trimmed.find('{')
        && let Some(end) = trimmed.rfind('}')
    {
        return serde_json::from_str(&trimmed[start..=end]);
    }
    serde_json::from_str(trimmed)
}

fn compact_json_retry_prompt(user: &str) -> String {
    format!(
        "{user}\n\n\
         【重要重试约束】上一次输出 JSON 因长度或截断无法解析。请重新生成一份更紧凑但完整闭合的严格 JSON：\n\
         - 优先保证 JSON 语法完整，所有对象和数组必须闭合\n\
         - 所有数组使用要求中的最低数量\n\
         - 所有文本字段压缩到 1-2 句\n\
         - original_excerpt 不超过 160 个字符\n\
         - 不要输出 markdown 或 JSON 之外的解释"
    )
}

fn is_probably_truncated_json(raw: &str, error: &serde_json::Error) -> bool {
    matches!(error.classify(), serde_json::error::Category::Eof)
        || !raw.trim_end().ends_with(['}', ']']) && error.to_string().contains("EOF")
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

fn error_excerpt(raw: &str, line: usize, column: usize) -> String {
    let lines: Vec<&str> = raw.lines().collect();
    if lines.is_empty() {
        return truncate(raw, 240);
    }

    let line_index = line.saturating_sub(1).min(lines.len().saturating_sub(1));
    let start = line_index.saturating_sub(2);
    let end = (line_index + 3).min(lines.len());
    let excerpt = lines[start..end]
        .iter()
        .enumerate()
        .map(|(idx, text)| {
            let current_line = start + idx + 1;
            if current_line == line {
                format!("{current_line}:{column}: {text}")
            } else {
                format!("{current_line}: {text}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    truncate(&excerpt, 800)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_parsing_accepts_agent_aliases() {
        assert_eq!("reader".parse::<LlmRole>().unwrap(), LlmRole::Reader);
        assert_eq!("json-repair".parse::<LlmRole>().unwrap(), LlmRole::Repair);
        assert!("unknown".parse::<LlmRole>().is_err());
    }

    #[test]
    fn role_route_falls_back_to_default_provider() {
        let primary = LlmProviderConfig {
            id: "primary".to_string(),
            api_key: None,
            base_url: "https://example.test/v1".to_string(),
            model: "primary-model".to_string(),
            temperature: 0.4,
            prefer_responses_api: false,
        };
        let fallback = LlmProviderConfig {
            id: "fallback".to_string(),
            api_key: Some("key".to_string()),
            base_url: "https://fallback.test/v1".to_string(),
            model: "fallback-model".to_string(),
            temperature: 0.2,
            prefer_responses_api: false,
        };
        let mut routes = HashMap::new();
        routes.insert(LlmRole::Default, vec!["fallback".to_string()]);
        routes.insert(LlmRole::Reader, vec!["primary".to_string()]);
        let client = LlmClient::from_profile(LlmProfileConfig {
            providers: vec![primary, fallback],
            role_routes: routes,
        });

        let providers = client.providers_for_role(LlmRole::Reader);
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].id, "fallback");
    }

    #[test]
    fn detects_truncated_json_string() {
        let raw =
            r#"{"translation":{"sections":[{"heading":"Implementation","original_excerpt":"The"#;
        let error = parse_json_lenient(raw).expect_err("truncated JSON should fail");

        assert!(is_probably_truncated_json(raw, &error));
    }

    #[test]
    fn does_not_treat_closed_invalid_json_as_truncated() {
        let raw = r#"{"items":[1,]}"#;
        let error = parse_json_lenient(raw).expect_err("invalid JSON should fail");

        assert!(!is_probably_truncated_json(raw, &error));
    }
}
