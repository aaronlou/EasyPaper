use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::interpretation::Interpretation;
use super::paper::PaperSummary;

/// POST /api/papers 上传成功后的响应
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub paper: PaperSummary,
}

/// GET /api/papers/:id 返回的完整论文 + 解读
#[derive(Debug, Serialize)]
pub struct PaperDetail {
    pub paper: PaperSummary,
    /// 解读结果（status != completed 时为 None）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interpretation: Option<Interpretation>,
}

/// GET /api/papers/:id/progress 推送的进度事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "phase", rename_all = "snake_case")]
pub enum ProgressEvent {
    /// 文本提取完成
    Extracted { char_count: usize },
    /// 开始 LLM 解读
    Interpreting { stage: String, message: String },
    /// 某阶段完成
    StageDone { stage: String },
    /// 全部完成
    Completed { interpretation: Interpretation },
    /// 失败
    Failed { message: String },
}

/// 用于占位的辅助：生成新 block id
pub fn new_block_id() -> String {
    Uuid::new_v4().to_string()[..8].to_string()
}
