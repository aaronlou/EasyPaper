use async_trait::async_trait;
use sqlx::Row;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::str::FromStr;
use uuid::Uuid;

use crate::domain::interpretation::Interpretation;
use crate::domain::paper::{Paper, PaperStatus, PaperSummary};
use crate::domain::repositories::PaperRepository;
use crate::error::AppError;

/// SQLite 实现。用 sqlx 连接池
#[derive(Clone)]
pub struct SqliteStore {
    pool: SqlitePool,
}

impl SqliteStore {
    pub async fn new(db_path: &std::path::Path) -> anyhow::Result<Self> {
        let url = format!("sqlite://{}?mode=rwc", db_path.display());
        let opts = SqliteConnectOptions::from_str(&url)?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

        let pool = SqlitePool::connect_with(opts).await?;

        // 建表（idempotent）
        sqlx::query(SCHEMA).execute(&pool).await?;

        tracing::info!("SQLite 初始化完成：{}", db_path.display());
        Ok(Self { pool })
    }
}

#[async_trait]
impl PaperRepository for SqliteStore {
    async fn insert_paper(&self, paper: &Paper) -> anyhow::Result<()> {
        let authors_json = serde_json::to_string(&paper.authors)?;
        let status_str = format!("{:?}", paper.status).to_lowercase();

        sqlx::query(
            "INSERT INTO papers (id, filename, title, authors, full_text, char_count, status, created_at, completed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )
        .bind(paper.id.to_string())
        .bind(&paper.filename)
        .bind(&paper.title)
        .bind(&authors_json)
        .bind(&paper.full_text)
        .bind(paper.char_count as i64)
        .bind(&status_str)
        .bind(&paper.created_at)
        .bind(&paper.completed_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_paper(&self, id: Uuid) -> anyhow::Result<Option<Paper>> {
        let row = sqlx::query("SELECT filename, title, authors, full_text, status, char_count, created_at, completed_at FROM papers WHERE id = ?1")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        let row = match row {
            Some(r) => r,
            None => return Ok(None),
        };

        let filename: String = row.get(0);
        let title: String = row.get(1);
        let authors_json: String = row.get(2);
        let full_text: String = row.get(3);
        let status_str: String = row.get(4);
        let char_count: i64 = row.get(5);
        let created_at: String = row.get(6);
        let completed_at: Option<String> = row.get(7);

        let authors: Vec<String> = serde_json::from_str(&authors_json).unwrap_or_default();
        let status = match status_str.as_str() {
            "uploaded" => PaperStatus::Uploaded,
            "processing" => PaperStatus::Processing,
            "completed" => PaperStatus::Completed,
            "failed" => PaperStatus::Failed,
            _ => PaperStatus::Uploaded,
        };

        Ok(Some(Paper {
            id,
            filename,
            title,
            authors,
            full_text,
            char_count: char_count as usize,
            status,
            created_at,
            completed_at,
        }))
    }

    async fn list_papers(&self) -> anyhow::Result<Vec<PaperSummary>> {
        let rows = sqlx::query(
            "SELECT id, filename, title, authors, char_count, status, created_at, completed_at
             FROM papers ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut out = Vec::new();
        for row in rows {
            let id_str: String = row.get(0);
            let id = Uuid::parse_str(&id_str).unwrap_or_else(|_| Uuid::new_v4());
            let authors_json: String = row.get(3);
            let authors: Vec<String> = serde_json::from_str(&authors_json).unwrap_or_default();
            let status_str: String = row.get(5);
            let status = match status_str.as_str() {
                "uploaded" => PaperStatus::Uploaded,
                "processing" => PaperStatus::Processing,
                "completed" => PaperStatus::Completed,
                "failed" => PaperStatus::Failed,
                _ => PaperStatus::Uploaded,
            };
            out.push(PaperSummary {
                id,
                filename: row.get(1),
                title: row.get(2),
                authors,
                char_count: row.get::<i64, _>(4) as usize,
                status,
                created_at: row.get(6),
                completed_at: row.get(7),
            });
        }
        Ok(out)
    }

    async fn update_status(&self, id: Uuid, status: PaperStatus) -> anyhow::Result<()> {
        let status_str = format!("{:?}", status).to_lowercase();
        let completed_at = if matches!(status, PaperStatus::Completed) {
            Some(chrono::Utc::now().to_rfc3339())
        } else {
            None
        };
        sqlx::query("UPDATE papers SET status = ?1, completed_at = COALESCE(?2, completed_at) WHERE id = ?3")
            .bind(&status_str)
            .bind(&completed_at)
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn mark_interrupted_processing_as_failed(&self) -> anyhow::Result<u64> {
        let result = sqlx::query(
            "UPDATE papers
             SET status = 'failed'
             WHERE status = 'processing'
               AND id NOT IN (SELECT paper_id FROM interpretations)",
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    async fn save_interpretation(&self, interp: &Interpretation) -> anyhow::Result<()> {
        let json = serde_json::to_string(interp)?;
        let concepts_json = serde_json::to_string(&interp.concepts)?;

        sqlx::query(
            "INSERT INTO interpretations (paper_id, summary, blocks_json, concepts_json)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(paper_id) DO UPDATE SET
                summary = excluded.summary,
                blocks_json = excluded.blocks_json,
                concepts_json = excluded.concepts_json",
        )
        .bind(interp.paper_id.to_string())
        .bind(&interp.summary)
        .bind(&json)
        .bind(&concepts_json)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_interpretation(&self, paper_id: Uuid) -> anyhow::Result<Option<Interpretation>> {
        let row = sqlx::query("SELECT blocks_json FROM interpretations WHERE paper_id = ?1")
            .bind(paper_id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        let row = match row {
            Some(r) => r,
            None => return Ok(None),
        };

        let json: String = row.get(0);
        let interp: Interpretation =
            serde_json::from_str(&json).map_err(|e| AppError::Internal(e.into()))?;
        Ok(Some(interp))
    }
}

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS papers (
    id           TEXT PRIMARY KEY,
    filename     TEXT NOT NULL,
    title        TEXT NOT NULL,
    authors      TEXT NOT NULL,  -- JSON array
    full_text    TEXT NOT NULL,
    char_count   INTEGER NOT NULL,
    status       TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    completed_at TEXT
);

CREATE TABLE IF NOT EXISTS interpretations (
    paper_id      TEXT PRIMARY KEY REFERENCES papers(id),
    summary       TEXT,
    blocks_json   TEXT NOT NULL,   -- 完整 Interpretation 序列化
    concepts_json TEXT NOT NULL,
    created_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_papers_created ON papers(created_at DESC);
"#;
