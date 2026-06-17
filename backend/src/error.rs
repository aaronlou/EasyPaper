use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// 应用级错误类型
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("请求格式错误: {0}")]
    BadRequest(String),

    #[error("资源未找到: {0}")]
    NotFound(String),

    #[error("文件过大（最大 {limit} 字节）")]
    PayloadTooLarge { limit: usize },

    #[error("PDF 解析失败: {0}")]
    PdfExtract(String),

    #[error("LLM 调用失败: {0}")]
    LlmCall(String),

    #[error("LLM 输出无效: {0}")]
    InvalidLlmOutput(String),

    #[error("LLM 未配置（缺少 OPENAI_API_KEY）")]
    LlmNotConfigured,

    #[error("数据库错误: {0}")]
    Database(String),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("内部错误: {0}")]
    Internal(#[from] anyhow::Error),
}

/// 统一的错误响应体
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    code: &'static str,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            AppError::PayloadTooLarge { limit } => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "PAYLOAD_TOO_LARGE",
                format!("文件过大，最大 {limit} 字节"),
            ),
            AppError::PdfExtract(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "PDF_EXTRACT_FAILED",
                msg.clone(),
            ),
            AppError::LlmCall(msg) => (StatusCode::BAD_GATEWAY, "LLM_CALL_FAILED", msg.clone()),
            AppError::InvalidLlmOutput(msg) => {
                (StatusCode::BAD_GATEWAY, "INVALID_LLM_OUTPUT", msg.clone())
            }
            AppError::LlmNotConfigured => (
                StatusCode::SERVICE_UNAVAILABLE,
                "LLM_NOT_CONFIGURED",
                "未配置 OPENAI_API_KEY".into(),
            ),
            AppError::Database(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                msg.clone(),
            ),
            AppError::Io(e) => (StatusCode::INTERNAL_SERVER_ERROR, "IO_ERROR", e.to_string()),
            AppError::Internal(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                e.to_string(),
            ),
        };

        tracing::error!(code = code, status = %status, "请求失败: {message}");

        let body = ErrorResponse {
            error: code.to_string(),
            code,
            message,
        };

        (status, axum::Json(body)).into_response()
    }
}

/// Handler 通用返回类型
pub type AppResult<T> = Result<T, AppError>;
