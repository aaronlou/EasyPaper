use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

/// 一篇上传的论文
#[derive(Debug, Clone, Serialize)]
pub struct Paper {
    id: Uuid,
    filename: String,
    /// 从 PDF 元信息或首页推断出的标题
    title: String,
    /// 从首页推断出的作者列表
    authors: Vec<String>,
    /// 提取出的全文（M1 直接存全文，M2 之后考虑分段）
    full_text: String,
    /// 字符数
    char_count: usize,
    /// 处理状态
    status: PaperStatus,
    /// 创建时间（RFC3339）
    created_at: String,
    /// 完成时间
    completed_at: Option<String>,
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

impl PaperStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Uploaded => "uploaded",
            Self::Processing => "processing",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

impl fmt::Display for PaperStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, thiserror::Error)]
#[error("未知论文状态: {0}")]
pub struct PaperStatusParseError(String);

impl FromStr for PaperStatus {
    type Err = PaperStatusParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "uploaded" => Ok(Self::Uploaded),
            "processing" => Ok(Self::Processing),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            other => Err(PaperStatusParseError(other.to_string())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PaperLifecycleError {
    #[error("论文状态不能从 {from} 切换到 {to}")]
    InvalidTransition { from: PaperStatus, to: PaperStatus },
}

/// Persisted representation used to rebuild a paper aggregate from storage.
#[derive(Debug, Clone)]
pub struct PaperRecord {
    pub id: Uuid,
    pub filename: String,
    pub title: String,
    pub authors: Vec<String>,
    pub full_text: String,
    pub status: PaperStatus,
    pub created_at: String,
    pub completed_at: Option<String>,
}

impl Paper {
    pub fn new_uploaded(
        filename: String,
        title: String,
        authors: Vec<String>,
        full_text: String,
    ) -> Self {
        let char_count = full_text.chars().count();
        Self {
            id: Uuid::new_v4(),
            filename,
            title,
            authors,
            full_text,
            char_count,
            status: PaperStatus::Uploaded,
            created_at: chrono::Utc::now().to_rfc3339(),
            completed_at: None,
        }
    }

    /// Rebuild a paper aggregate from persisted data.
    ///
    /// The derived `char_count` is intentionally recomputed from `full_text` so
    /// the aggregate keeps a single source of truth for this invariant.
    pub fn rehydrate(record: PaperRecord) -> Self {
        let char_count = record.full_text.chars().count();
        Self {
            id: record.id,
            filename: record.filename,
            title: record.title,
            authors: record.authors,
            full_text: record.full_text,
            char_count,
            status: record.status,
            created_at: record.created_at,
            completed_at: record.completed_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn authors(&self) -> &[String] {
        &self.authors
    }

    pub fn full_text(&self) -> &str {
        &self.full_text
    }

    pub fn char_count(&self) -> usize {
        self.char_count
    }

    pub fn status(&self) -> PaperStatus {
        self.status
    }

    pub fn created_at(&self) -> &str {
        &self.created_at
    }

    pub fn completed_at(&self) -> Option<&str> {
        self.completed_at.as_deref()
    }

    pub fn is_completed(&self) -> bool {
        matches!(self.status, PaperStatus::Completed)
    }

    pub fn queue_for_retry(&mut self) -> Result<(), PaperLifecycleError> {
        if matches!(self.status, PaperStatus::Processing) {
            return Err(PaperLifecycleError::InvalidTransition {
                from: self.status,
                to: PaperStatus::Uploaded,
            });
        }

        self.status = PaperStatus::Uploaded;
        self.completed_at = None;
        Ok(())
    }

    pub fn start_processing(&mut self) -> Result<(), PaperLifecycleError> {
        self.transition_to(PaperStatus::Processing, &[PaperStatus::Uploaded])
    }

    pub fn complete(&mut self) -> Result<(), PaperLifecycleError> {
        self.transition_to(PaperStatus::Completed, &[PaperStatus::Processing])?;
        self.completed_at = Some(chrono::Utc::now().to_rfc3339());
        Ok(())
    }

    pub fn fail(&mut self) -> Result<(), PaperLifecycleError> {
        self.transition_to(
            PaperStatus::Failed,
            &[PaperStatus::Uploaded, PaperStatus::Processing],
        )?;
        self.completed_at = None;
        Ok(())
    }

    fn transition_to(
        &mut self,
        next: PaperStatus,
        allowed_from: &[PaperStatus],
    ) -> Result<(), PaperLifecycleError> {
        if allowed_from.contains(&self.status) {
            self.status = next;
            Ok(())
        } else {
            Err(PaperLifecycleError::InvalidTransition {
                from: self.status,
                to: next,
            })
        }
    }
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
        Self::from(&p)
    }
}

impl From<&Paper> for PaperSummary {
    fn from(p: &Paper) -> Self {
        Self {
            id: p.id,
            filename: p.filename.clone(),
            title: p.title.clone(),
            authors: p.authors.clone(),
            char_count: p.char_count,
            status: p.status,
            created_at: p.created_at.clone(),
            completed_at: p.completed_at.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_uploaded_paper_owns_its_derived_fields() {
        let paper = Paper::new_uploaded(
            "paper.pdf".to_string(),
            "A Useful Paper".to_string(),
            vec!["Ada".to_string()],
            "hello 世界".to_string(),
        );

        assert_eq!(paper.status(), PaperStatus::Uploaded);
        assert_eq!(paper.char_count(), 8);
        assert!(paper.completed_at().is_none());
    }

    #[test]
    fn paper_lifecycle_controls_status_transitions() {
        let mut paper = Paper::new_uploaded(
            "paper.pdf".to_string(),
            "A Useful Paper".to_string(),
            Vec::new(),
            "body".to_string(),
        );

        paper.start_processing().expect("can start from uploaded");
        assert_eq!(paper.status(), PaperStatus::Processing);

        let retry_while_processing = paper.queue_for_retry();
        assert!(retry_while_processing.is_err());

        paper.complete().expect("can complete from processing");
        assert_eq!(paper.status(), PaperStatus::Completed);
        assert!(paper.completed_at().is_some());

        paper
            .queue_for_retry()
            .expect("completed papers can be retried");
        assert_eq!(paper.status(), PaperStatus::Uploaded);
        assert!(paper.completed_at().is_none());
    }

    #[test]
    fn status_round_trips_through_storage_string() {
        let status: PaperStatus = "completed".parse().expect("valid status");
        assert_eq!(status, PaperStatus::Completed);
        assert_eq!(status.as_str(), "completed");
        assert!("missing".parse::<PaperStatus>().is_err());
    }
}
