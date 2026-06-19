use async_trait::async_trait;
use sqlx::Row;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::str::FromStr;
use uuid::Uuid;

use crate::domain::interpretation::Interpretation;
use crate::domain::paper::{Paper, PaperRecord, PaperStatus, PaperSummary};
use crate::domain::repositories::PaperRepository;
use crate::domain::study_pack::StudyPack;
use crate::error::AppError;
use crate::{application::ports::ConceptExpansionCache, models::api::ConceptExpansion};

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
        run_lightweight_migrations(&pool).await?;

        tracing::info!("SQLite 初始化完成：{}", db_path.display());
        Ok(Self { pool })
    }
}

#[async_trait]
impl PaperRepository for SqliteStore {
    async fn insert_paper(&self, paper: &Paper) -> anyhow::Result<()> {
        let authors_json = serde_json::to_string(paper.authors())?;

        sqlx::query(
            "INSERT INTO papers (id, filename, title, authors, full_text, char_count, status, created_at, completed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )
        .bind(paper.id().to_string())
        .bind(paper.filename())
        .bind(paper.title())
        .bind(&authors_json)
        .bind(paper.full_text())
        .bind(paper.char_count() as i64)
        .bind(paper.status().as_str())
        .bind(paper.created_at())
        .bind(paper.completed_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn save_paper(&self, paper: &Paper) -> anyhow::Result<()> {
        let authors_json = serde_json::to_string(paper.authors())?;

        sqlx::query(
            "UPDATE papers
             SET filename = ?1,
                 title = ?2,
                 authors = ?3,
                 full_text = ?4,
                 char_count = ?5,
                 status = ?6,
                 created_at = ?7,
                 completed_at = ?8
             WHERE id = ?9",
        )
        .bind(paper.filename())
        .bind(paper.title())
        .bind(&authors_json)
        .bind(paper.full_text())
        .bind(paper.char_count() as i64)
        .bind(paper.status().as_str())
        .bind(paper.created_at())
        .bind(paper.completed_at())
        .bind(paper.id().to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_paper(&self, id: Uuid) -> anyhow::Result<Option<Paper>> {
        let row = sqlx::query("SELECT filename, title, authors, full_text, status, created_at, completed_at FROM papers WHERE id = ?1")
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
        let created_at: String = row.get(5);
        let completed_at: Option<String> = row.get(6);

        let authors: Vec<String> = serde_json::from_str(&authors_json).unwrap_or_default();
        let status = PaperStatus::from_str(&status_str)?;

        Ok(Some(Paper::rehydrate(PaperRecord {
            id,
            filename,
            title,
            authors,
            full_text,
            status,
            created_at,
            completed_at,
        })))
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
            let id = Uuid::parse_str(&id_str)?;
            let authors_json: String = row.get(3);
            let authors: Vec<String> = serde_json::from_str(&authors_json).unwrap_or_default();
            let status_str: String = row.get(5);
            let status = PaperStatus::from_str(&status_str)?;
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

    async fn list_interrupted_processing_papers(&self) -> anyhow::Result<Vec<Paper>> {
        let rows = sqlx::query(
            "SELECT id, filename, title, authors, full_text, status, created_at, completed_at
             FROM papers
             WHERE status = 'processing'
               AND id NOT IN (SELECT paper_id FROM interpretations)",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut out = Vec::new();
        for row in rows {
            let id_str: String = row.get(0);
            let authors_json: String = row.get(3);
            let authors: Vec<String> = serde_json::from_str(&authors_json).unwrap_or_default();
            let status_str: String = row.get(5);
            out.push(Paper::rehydrate(PaperRecord {
                id: Uuid::parse_str(&id_str)?,
                filename: row.get(1),
                title: row.get(2),
                authors,
                full_text: row.get(4),
                status: PaperStatus::from_str(&status_str)?,
                created_at: row.get(6),
                completed_at: row.get(7),
            }));
        }

        Ok(out)
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

    async fn save_study_pack(&self, pack: &StudyPack, cache_version: &str) -> anyhow::Result<()> {
        let json = serde_json::to_string(pack)?;

        sqlx::query(
            "INSERT INTO study_packs (paper_id, cache_version, pack_json)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(paper_id) DO UPDATE SET
                cache_version = excluded.cache_version,
                pack_json = excluded.pack_json,
                updated_at = datetime('now')",
        )
        .bind(pack.paper_id.to_string())
        .bind(cache_version)
        .bind(&json)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_study_pack(
        &self,
        paper_id: Uuid,
        cache_version: &str,
    ) -> anyhow::Result<Option<StudyPack>> {
        let row = sqlx::query(
            "SELECT pack_json
             FROM study_packs
             WHERE paper_id = ?1
               AND cache_version = ?2",
        )
        .bind(paper_id.to_string())
        .bind(cache_version)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let json: String = row.get(0);
        let pack = serde_json::from_str(&json).map_err(|e| AppError::Internal(e.into()))?;
        Ok(Some(pack))
    }
}

#[async_trait]
impl ConceptExpansionCache for SqliteStore {
    async fn get_concept_expansion(
        &self,
        paper_id: Uuid,
        concept_id: &str,
        cache_version: &str,
        max_age_days: i64,
    ) -> anyhow::Result<Option<ConceptExpansion>> {
        let row = sqlx::query(
            "SELECT expansion_json
             FROM concept_expansions
             WHERE paper_id = ?1
               AND concept_id = ?2
               AND cache_version = ?3
               AND updated_at >= datetime('now', ?4)",
        )
        .bind(paper_id.to_string())
        .bind(concept_id)
        .bind(cache_version)
        .bind(format!("-{max_age_days} days"))
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let json: String = row.get(0);
        let expansion = serde_json::from_str(&json).map_err(|e| AppError::Internal(e.into()))?;
        Ok(Some(expansion))
    }

    async fn save_concept_expansion(
        &self,
        paper_id: Uuid,
        concept_id: &str,
        cache_version: &str,
        expansion: &ConceptExpansion,
    ) -> anyhow::Result<()> {
        let json = serde_json::to_string(expansion)?;

        sqlx::query(
            "INSERT INTO concept_expansions (paper_id, concept_id, cache_version, expansion_json)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(paper_id, concept_id) DO UPDATE SET
                cache_version = excluded.cache_version,
                expansion_json = excluded.expansion_json,
                updated_at = datetime('now')",
        )
        .bind(paper_id.to_string())
        .bind(concept_id)
        .bind(cache_version)
        .bind(&json)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_expired_concept_expansions(&self, max_age_days: i64) -> anyhow::Result<u64> {
        let result = sqlx::query(
            "DELETE FROM concept_expansions
             WHERE updated_at < datetime('now', ?1)",
        )
        .bind(format!("-{max_age_days} days"))
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
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

CREATE TABLE IF NOT EXISTS concept_expansions (
    paper_id       TEXT NOT NULL REFERENCES papers(id),
    concept_id     TEXT NOT NULL,
    cache_version  TEXT NOT NULL DEFAULT 'concept-expansion-v1',
    expansion_json TEXT NOT NULL,
    created_at     TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at     TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (paper_id, concept_id)
);

CREATE TABLE IF NOT EXISTS study_packs (
    paper_id      TEXT PRIMARY KEY REFERENCES papers(id),
    cache_version TEXT NOT NULL,
    pack_json     TEXT NOT NULL,
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_papers_created ON papers(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_concept_expansions_updated
    ON concept_expansions(updated_at);
CREATE INDEX IF NOT EXISTS idx_study_packs_updated
    ON study_packs(updated_at);
"#;

async fn run_lightweight_migrations(pool: &SqlitePool) -> anyhow::Result<()> {
    ensure_column(
        pool,
        "concept_expansions",
        "cache_version",
        "TEXT NOT NULL DEFAULT 'concept-expansion-v1'",
    )
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS study_packs (
            paper_id      TEXT PRIMARY KEY REFERENCES papers(id),
            cache_version TEXT NOT NULL,
            pack_json     TEXT NOT NULL,
            created_at    TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at    TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_study_packs_updated
         ON study_packs(updated_at)",
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn ensure_column(
    pool: &SqlitePool,
    table: &str,
    column: &str,
    definition: &str,
) -> anyhow::Result<()> {
    let rows = sqlx::query(&format!("PRAGMA table_info({table})"))
        .fetch_all(pool)
        .await?;
    let exists = rows
        .iter()
        .any(|row| row.get::<String, _>("name") == column);

    if !exists {
        sqlx::query(&format!(
            "ALTER TABLE {table} ADD COLUMN {column} {definition}"
        ))
        .execute(pool)
        .await?;
    }

    Ok(())
}
