use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 一篇上传的论文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paper {
    pub id: Uuid,
    pub filename: String,
    /// 从 PDF 元信息或首页推断出的标题
    pub title: String,
    /// 从首页推断出的作者列表
    pub authors: Vec<String>,
    /// 提取出的全文（M1 直接存全文，M2 之后考虑分段）
    pub full_text: String,
    /// 字符数
    pub char_count: usize,
    /// 处理状态
    pub status: PaperStatus,
    /// 创建时间（RFC3339）
    pub created_at: String,
    /// 完成时间
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PaperStatus {
    /// 刚上传，文本已提取
    Uploaded,
    /// LLM 解读中
    Processing,
    /// 解读完成
    Completed,
    /// 解读失败
    Failed,
}

/// 论文摘要（列表用，不含全文）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperSummary {
    pub id: Uuid,
    pub filename: String,
    pub title: String,
    pub authors: Vec<String>,
    pub char_count: usize,
    pub status: PaperStatus,
    pub created_at: String,
    pub completed_at: Option<String>,
}

impl From<Paper> for PaperSummary {
    fn from(p: Paper) -> Self {
        Self {
            id: p.id,
            filename: p.filename,
            title: p.title,
            authors: p.authors,
            char_count: p.char_count,
            status: p.status,
            created_at: p.created_at,
            completed_at: p.completed_at,
        }
    }
}
