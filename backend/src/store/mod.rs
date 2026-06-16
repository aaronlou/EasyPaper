pub mod sqlite;

pub use sqlite::SqliteStore;

use async_trait::async_trait;
use std::sync::Arc;

use crate::models::interpretation::Interpretation;
use crate::models::paper::{Paper, PaperStatus, PaperSummary};

/// 存储抽象。当前只有 SQLite 实现，后续可换 Postgres 等
#[async_trait]
pub trait PaperStore: Send + Sync + 'static {
    async fn insert_paper(&self, paper: &Paper) -> anyhow::Result<()>;
    async fn get_paper(&self, id: uuid::Uuid) -> anyhow::Result<Option<Paper>>;
    async fn list_papers(&self) -> anyhow::Result<Vec<PaperSummary>>;
    async fn update_status(
        &self,
        id: uuid::Uuid,
        status: PaperStatus,
    ) -> anyhow::Result<()>;
    async fn save_interpretation(&self, interp: &Interpretation) -> anyhow::Result<()>;
    async fn get_interpretation(
        &self,
        paper_id: uuid::Uuid,
    ) -> anyhow::Result<Option<Interpretation>>;
}

/// 类型别名，便于在 AppState 里用
pub type SharedStore = Arc<dyn PaperStore>;
