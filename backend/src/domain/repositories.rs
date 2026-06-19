use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::interpretation::Interpretation;
use crate::domain::paper::{Paper, PaperSummary};
use crate::domain::study_pack::StudyPack;

/// 论文聚合仓储端口。
///
/// DDD 边界：domain/application 只依赖这个 trait，具体 SQLite 实现放在 infrastructure。
#[async_trait]
pub trait PaperRepository: Send + Sync + 'static {
    async fn insert_paper(&self, paper: &Paper) -> anyhow::Result<()>;
    async fn save_paper(&self, paper: &Paper) -> anyhow::Result<()>;
    async fn get_paper(&self, id: uuid::Uuid) -> anyhow::Result<Option<Paper>>;
    async fn list_papers(&self) -> anyhow::Result<Vec<PaperSummary>>;
    async fn list_interrupted_processing_papers(&self) -> anyhow::Result<Vec<Paper>>;
    async fn save_interpretation(&self, interp: &Interpretation) -> anyhow::Result<()>;
    async fn get_interpretation(
        &self,
        paper_id: uuid::Uuid,
    ) -> anyhow::Result<Option<Interpretation>>;
    async fn save_study_pack(&self, pack: &StudyPack, cache_version: &str) -> anyhow::Result<()>;
    async fn get_study_pack(
        &self,
        paper_id: uuid::Uuid,
        cache_version: &str,
    ) -> anyhow::Result<Option<StudyPack>>;
}

pub type SharedPaperRepository = Arc<dyn PaperRepository>;
