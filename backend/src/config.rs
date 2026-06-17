use std::path::PathBuf;

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
    /// 静态文件目录（前端 dist）
    pub static_dir: PathBuf,
    /// SQLite 数据库路径
    pub db_path: PathBuf,
    /// 上传文件临时目录
    pub upload_dir: PathBuf,
    /// 可选 Web 检索端点。支持 Tavily POST 或 SearXNG JSON GET。
    pub web_search_url: Option<String>,
    /// 可选 Web 检索 API Key（例如 Tavily）。
    pub web_search_api_key: Option<String>,
    /// 每个概念最多注入多少条外部检索结果。
    pub web_search_max_results: usize,
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

        let static_dir = std::env::var("STATIC_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| workspace_root.join("dist"));

        let db_path = std::env::var("EASYPAPER_DB_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| workspace_root.join("data").join("easypaper.db"));

        let upload_dir = std::env::var("EASYPAPER_UPLOAD_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| workspace_root.join("data").join("uploads"));

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
            static_dir,
            db_path,
            upload_dir,
            web_search_url,
            web_search_api_key,
            web_search_max_results,
        })
    }
}
