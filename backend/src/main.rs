use tracing_subscriber::EnvFilter;

mod app;
mod config;
mod error;
mod llm;
mod models;
mod pdf;
mod prompt;
mod routes;
mod store;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 加载 .env（开发环境用，生产环境忽略找不到的情况）
    let _ = dotenvy::dotenv();

    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                EnvFilter::new("easypaper_backend=info,tower_http=info")
            }),
        )
        .init();

    // 读取配置
    let config = config::Config::from_env()?;
    tracing::info!("EasyPaper 后端启动中，监听 {}", config.bind_addr);

    // 构建 Router
    let app = app::build(config.clone()).await?;

    // 绑定监听
    let bind_addr = config.bind_addr.clone();
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("✅ 服务就绪：http://{bind_addr}");

    // 启动 + graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("服务已关闭");
    Ok(())
}

/// 监听 Ctrl-C 和 SIGTERM，优雅关闭
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl-C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("收到 Ctrl-C 信号"),
        _ = terminate => tracing::info!("收到 SIGTERM 信号"),
    }
}
