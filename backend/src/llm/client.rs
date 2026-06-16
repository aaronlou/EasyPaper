use serde::Deserialize;
use serde_json::Value;

use crate::error::{AppError, AppResult};

/// OpenAI 兼容客户端（沿用 letters 的双协议设计）
#[derive(Clone)]
pub struct LlmClient {
    http: reqwest::Client,
    api_key: Option<String>,
    base_url: String,
    model: String,
}

impl LlmClient {
    pub fn new(api_key: Option<String>, base_url: String, model: String) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("failed to build reqwest client"),
            api_key,
            base_url,
            model,
        }
    }

    pub fn is_configured(&self) -> bool {
        self.api_key.is_some()
    }

    /// 是否走新版 Responses API（仅 OpenAI 官方）
    fn uses_responses_api(&self) -> bool {
        self.base_url.contains("api.openai.com")
    }

    /// 调用 LLM 并期望返回 JSON（解析成 serde_json::Value）
    pub async fn call_json(&self, system: &str, user: &str) -> AppResult<Value> {
        let raw = self.call_text(system, user).await?;
        parse_json_lenient(&raw)
            .map_err(|e| AppError::InvalidLlmOutput(format!("{e}")))
    }

    /// 调用 LLM 返回纯文本
    pub async fn call_text(&self, system: &str, user: &str) -> AppResult<String> {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or(AppError::LlmNotConfigured)?;

        let url = if self.uses_responses_api() {
            format!("{}/responses", self.base_url.trim_end_matches('/'))
        } else {
            format!("{}/chat/completions", self.base_url.trim_end_matches('/'))
        };

        let body = if self.uses_responses_api() {
            serde_json::json!({
                "model": self.model,
                "input": [
                    { "role": "developer", "content": system },
                    { "role": "user", "content": user }
                ],
            })
        } else {
            serde_json::json!({
                "model": self.model,
                "messages": [
                    { "role": "system", "content": system },
                    { "role": "user", "content": user }
                ],
                "temperature": 0.4,
                "response_format": { "type": "json_object" },
            })
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

        let resp_json: Value = resp
            .json()
            .await
            .map_err(|e| AppError::LlmCall(format!("响应解析失败: {e}")))?;

        // 兼容两种 API 的响应结构
        let content = if self.uses_responses_api() {
            resp_json
                .get("output")
                .and_then(|o| o.get(0))
                .and_then(|o| o.get("content"))
                .and_then(|c| c.get(0))
                .and_then(|c| c.get("text"))
                .and_then(|t| t.as_str())
                .unwrap_or("")
        } else {
            resp_json
                .get("choices")
                .and_then(|c| c.get(0))
                .and_then(|c| c.get("message"))
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_str())
                .unwrap_or("")
        };

        Ok(content.to_string())
    }
}

/// 容错 JSON 解析：容忍 ```json ... ``` 代码块包裹
fn parse_json_lenient(raw: &str) -> Result<Value, serde_json::Error> {
    let trimmed = raw.trim();
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        return serde_json::from_str(trimmed);
    }
    // 尝试去掉 markdown 代码块
    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            return serde_json::from_str(&trimmed[start..=end]);
        }
    }
    serde_json::from_str(trimmed)
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

/// 占位类型让 #[derive(Deserialize)] 不被警告
#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}
#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}
#[derive(Deserialize)]
struct ChatMessage {
    content: String,
}

// 抑制未使用警告（保留备用）
impl ChatResponse {
    #[allow(dead_code)]
    fn into_content(self) -> String {
        self.choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default()
    }
}
