use std::path::PathBuf;

use std::collections::HashMap;

use crate::llm::{LlmProfileConfig, LlmProviderConfig, LlmRole};

/// 应用配置。全部从环境变量读取（dotenvy 自动加载 .env）
#[derive(Debug, Clone)]
pub struct Config {
    /// 监听地址
    pub bind_addr: String,
    /// OpenAI 兼容的 API Key
    pub openai_api_key: Option<String>,
    /// OpenAI 兼容的 base url（可指向 deepseek 等）
    pub openai_base_url: String,
    /// 模型名
    pub openai_model: String,
    /// 多 LLM provider 与 agent route 配置。
    pub llm_profile: LlmProfileConfig,
    /// 静态文件目录（前端 dist）
    pub static_dir: PathBuf,
    /// SQLite 数据库路径
    pub db_path: PathBuf,
    /// 上传文件临时目录
    pub upload_dir: PathBuf,
    /// 可选 CORS 白名单，逗号分隔。未配置时沿用开发态 permissive。
    pub cors_origins: Vec<String>,
    /// 可选 Web 检索端点。支持 Tavily POST 或 SearXNG JSON GET。
    pub web_search_url: Option<String>,
    /// 可选 Web 检索 API Key（例如 Tavily）。
    pub web_search_api_key: Option<String>,
    /// 每个概念最多注入多少条外部检索结果。
    pub web_search_max_results: usize,
    /// 解读完成后后台预热多少个概念深潜结果。0 表示关闭。
    pub concept_prewarm_limit: usize,
    /// 后台概念预热的全局并发数。
    pub concept_prewarm_concurrency: usize,
    /// 概念深潜缓存保留天数。
    pub concept_cache_ttl_days: i64,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        // 用 CARGO_MANIFEST_DIR 定位 workspace 根
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent() // 跳出 backend/
            .ok_or_else(|| anyhow::anyhow!("无法定位 workspace 根目录"))?
            .to_path_buf();

        let bind_addr =
            std::env::var("EASYPAPER_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8787".into());

        let openai_api_key = std::env::var("OPENAI_API_KEY")
            .ok()
            .filter(|s| !s.is_empty());

        let openai_base_url =
            std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".into());

        let openai_model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| {
            // 沿用 letters 的智能默认：deepseek 域名用 deepseek-chat，否则用 gpt
            if openai_base_url.contains("deepseek") {
                "deepseek-chat".into()
            } else {
                "gpt-4o-mini".into()
            }
        });
        let llm_profile = load_llm_profile(
            openai_api_key.clone(),
            openai_base_url.clone(),
            openai_model.clone(),
        )?;

        let static_dir = std::env::var("STATIC_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| workspace_root.join("dist"));

        let db_path = std::env::var("EASYPAPER_DB_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| workspace_root.join("data").join("easypaper.db"));

        let upload_dir = std::env::var("EASYPAPER_UPLOAD_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| workspace_root.join("data").join("uploads"));

        let cors_origins = std::env::var("EASYPAPER_CORS_ORIGINS")
            .ok()
            .map(|origins| {
                origins
                    .split(',')
                    .map(str::trim)
                    .filter(|origin| !origin.is_empty())
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let web_search_api_key = std::env::var("EASYPAPER_WEB_SEARCH_API_KEY")
            .ok()
            .filter(|s| !s.is_empty());

        let web_search_url = std::env::var("EASYPAPER_WEB_SEARCH_URL")
            .ok()
            .filter(|s| !s.is_empty())
            .or_else(|| {
                web_search_api_key
                    .as_ref()
                    .map(|_| "https://api.tavily.com/search".to_string())
            });

        let web_search_max_results = std::env::var("EASYPAPER_WEB_SEARCH_MAX_RESULTS")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(4)
            .clamp(1, 8);

        let concept_prewarm_limit = std::env::var("EASYPAPER_CONCEPT_PREWARM_LIMIT")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(3)
            .min(8);

        let concept_prewarm_concurrency = std::env::var("EASYPAPER_CONCEPT_PREWARM_CONCURRENCY")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(1)
            .clamp(1, 4);

        let concept_cache_ttl_days = std::env::var("EASYPAPER_CONCEPT_CACHE_TTL_DAYS")
            .ok()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(7)
            .clamp(1, 90);

        // 确保数据目录存在
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::create_dir_all(&upload_dir).ok();

        Ok(Self {
            bind_addr,
            openai_api_key,
            openai_base_url,
            openai_model,
            llm_profile,
            static_dir,
            db_path,
            upload_dir,
            cors_origins,
            web_search_url,
            web_search_api_key,
            web_search_max_results,
            concept_prewarm_limit,
            concept_prewarm_concurrency,
            concept_cache_ttl_days,
        })
    }
}

fn load_llm_profile(
    default_api_key: Option<String>,
    default_base_url: String,
    default_model: String,
) -> anyhow::Result<LlmProfileConfig> {
    let provider_ids = std::env::var("EASYPAPER_LLM_PROVIDERS")
        .ok()
        .map(|value| parse_csv(&value))
        .filter(|ids| !ids.is_empty());

    let providers = match provider_ids {
        Some(ids) => ids
            .into_iter()
            .map(|id| load_named_provider(&id))
            .collect::<anyhow::Result<Vec<_>>>()?,
        None => vec![LlmProviderConfig::default_compatible(
            default_api_key,
            default_base_url,
            default_model,
        )],
    };

    let provider_ids = providers
        .iter()
        .map(|provider| provider.id.clone())
        .collect::<Vec<_>>();
    let mut role_routes = HashMap::new();
    role_routes.insert(
        LlmRole::Default,
        parse_route("EASYPAPER_LLM_ROUTE_DEFAULT").unwrap_or_else(|| provider_ids.clone()),
    );
    for role in [
        LlmRole::Reader,
        LlmRole::Specialist,
        LlmRole::Concept,
        LlmRole::Repair,
        LlmRole::Study,
        LlmRole::Literature,
        LlmRole::Translation,
    ] {
        let env_name = format!("EASYPAPER_LLM_ROUTE_{}", role.as_str().to_ascii_uppercase());
        if let Some(route) = parse_route(&env_name) {
            role_routes.insert(role, route);
        }
    }

    Ok(LlmProfileConfig {
        providers,
        role_routes,
    })
}

fn load_named_provider(id: &str) -> anyhow::Result<LlmProviderConfig> {
    let prefix = format!(
        "EASYPAPER_LLM_PROVIDER_{}",
        id.to_ascii_uppercase().replace('-', "_")
    );
    let base_url = std::env::var(format!("{prefix}_BASE_URL"))
        .map_err(|_| anyhow::anyhow!("缺少 {prefix}_BASE_URL"))?;
    let model = std::env::var(format!("{prefix}_MODEL"))
        .map_err(|_| anyhow::anyhow!("缺少 {prefix}_MODEL"))?;
    let api_key = std::env::var(format!("{prefix}_API_KEY"))
        .ok()
        .filter(|key| !key.is_empty())
        .or_else(|| {
            std::env::var(format!("{prefix}_API_KEY_ENV"))
                .ok()
                .and_then(|env_name| std::env::var(env_name).ok())
        });
    let temperature = std::env::var(format!("{prefix}_TEMPERATURE"))
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or(0.4)
        .clamp(0.0, 2.0);
    let prefer_responses_api = std::env::var(format!("{prefix}_RESPONSES_API"))
        .ok()
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or_else(|| base_url.contains("api.openai.com"));

    Ok(LlmProviderConfig {
        id: id.to_string(),
        api_key,
        base_url,
        model,
        temperature,
        prefer_responses_api,
    })
}

fn parse_route(env_name: &str) -> Option<Vec<String>> {
    std::env::var(env_name)
        .ok()
        .map(|value| parse_csv(&value))
        .filter(|items| !items.is_empty())
}

fn parse_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
}
