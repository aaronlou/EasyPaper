use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::models::api::ConceptExpansion;

/// Text extracted from an uploaded paper file.
#[derive(Debug, Clone)]
pub struct ExtractedPaperText {
    pub full_text: String,
    pub title: String,
    pub authors: Vec<String>,
}

#[async_trait]
pub trait PdfExtractor: Send + Sync + 'static {
    async fn extract(&self, pdf_bytes: &[u8]) -> crate::error::AppResult<ExtractedPaperText>;
}

pub type SharedPdfExtractor = Arc<dyn PdfExtractor>;

#[async_trait]
pub trait ConceptExpansionCache: Send + Sync + 'static {
    async fn get_concept_expansion(
        &self,
        paper_id: Uuid,
        concept_id: &str,
        cache_version: &str,
        max_age_days: i64,
    ) -> anyhow::Result<Option<ConceptExpansion>>;

    async fn save_concept_expansion(
        &self,
        paper_id: Uuid,
        concept_id: &str,
        cache_version: &str,
        expansion: &ConceptExpansion,
    ) -> anyhow::Result<()>;

    async fn delete_expired_concept_expansions(&self, max_age_days: i64) -> anyhow::Result<u64>;
}

pub type SharedConceptExpansionCache = Arc<dyn ConceptExpansionCache>;
