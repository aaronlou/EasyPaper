use uuid::Uuid;

use crate::application::entitlements::AiBillingMode;
use crate::application::paper_workflow::PaperWorkflow;
use crate::domain::interpretation::{Block, Interpretation};
use crate::domain::research::WebSearchResult;
use crate::domain::study_pack::{StudyPack, StudyPackDraft};
use crate::error::{AppError, AppResult};
use crate::llm::LlmRole;
use crate::models::api::ClientLlmProfile;
use crate::prompt;

const STUDY_PACK_CACHE_VERSION: &str = "study-pack-v2-bilingual-translation";

impl PaperWorkflow {
    pub async fn get_or_generate_study_pack(
        &self,
        owner_id: &str,
        paper_id: Uuid,
        llm_profile: Option<ClientLlmProfile>,
    ) -> AppResult<StudyPack> {
        let profile_cache_key = llm_profile
            .as_ref()
            .map(ClientLlmProfile::cache_key)
            .unwrap_or_else(|| "managed".to_string());
        self.entitlements.record_workflow_start(
            AiBillingMode::from_profile(llm_profile.as_ref()),
            "study_pack",
        );
        let workflow = self.with_client_llm_profile(llm_profile);
        let cache_version = workflow.study_pack_cache_version();

        let paper = workflow
            .papers
            .get_paper_for_owner(owner_id, paper_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound(format!("论文 {paper_id} 不存在")))?;

        if let Some(pack) = workflow
            .papers
            .get_study_pack(paper_id, &cache_version)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            return Ok(pack);
        }

        if !workflow.llm.is_configured() {
            return Err(AppError::LlmNotConfigured);
        }

        let in_flight_key = format!("{paper_id}:{cache_version}:{profile_cache_key}");
        let _guard = workflow.acquire_study_pack_slot(in_flight_key).await;

        if let Some(pack) = workflow
            .papers
            .get_study_pack(paper_id, &cache_version)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            return Ok(pack);
        }

        let interpretation = workflow
            .papers
            .get_interpretation(paper_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound(format!("论文 {paper_id} 尚未解读完成")))?;

        let interpretation_context = summarize_interpretation(&interpretation);
        let research_context = workflow
            .build_study_pack_research_context(paper.title(), &interpretation)
            .await;
        let user_msg = prompt::user_study_pack(
            paper.title(),
            paper.full_text(),
            &interpretation_context,
            &research_context,
        );
        let value = workflow
            .llm
            .call_json_with_role(LlmRole::Study, prompt::SYSTEM_STUDY_PACK, &user_msg)
            .await
            .map_err(|e| AppError::LlmCall(format!("研究地图生成失败: {e}")))?;
        let draft: StudyPackDraft = serde_json::from_value(value)
            .map_err(|e| AppError::InvalidLlmOutput(format!("研究地图解析失败: {e}")))?;
        let now = chrono::Utc::now().to_rfc3339();
        let pack = draft.into_study_pack(paper_id, now);

        workflow
            .papers
            .save_study_pack(&pack, &cache_version)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(pack)
    }

    async fn acquire_study_pack_slot(&self, key: String) -> StudyPackSlotGuard {
        loop {
            let mut in_flight = self.study_pack_in_flight.lock().await;
            if in_flight.insert(key.clone()) {
                return StudyPackSlotGuard {
                    key,
                    in_flight: self.study_pack_in_flight.clone(),
                };
            }
            drop(in_flight);
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }
    }

    fn study_pack_cache_version(&self) -> String {
        format!("{STUDY_PACK_CACHE_VERSION}:{}", self.llm.cache_namespace())
    }

    async fn build_study_pack_research_context(
        &self,
        title: &str,
        interpretation: &Interpretation,
    ) -> String {
        let mut queries = vec![
            format!("{title} related work literature review"),
            format!("{title} follow up citation"),
        ];
        queries.extend(
            interpretation
                .concepts
                .iter()
                .take(3)
                .map(|concept| format!("{} survey prerequisite paper", concept.term)),
        );

        let mut lines = Vec::new();
        for query in queries.into_iter().take(5) {
            let results = self.research.search(&query).await;
            lines.push(format!("Query: {query}"));
            lines.extend(format_search_results(&results));
        }

        if lines.is_empty() {
            "未配置外部检索，研究地图只能基于论文正文和已有解读生成。".to_string()
        } else {
            limit_chars(&lines.join("\n"), 6_000)
        }
    }
}

struct StudyPackSlotGuard {
    key: String,
    in_flight: crate::application::paper_workflow::StudyPackInFlight,
}

impl Drop for StudyPackSlotGuard {
    fn drop(&mut self) {
        let key = self.key.clone();
        let in_flight = self.in_flight.clone();
        tokio::spawn(async move {
            in_flight.lock().await.remove(&key);
        });
    }
}

fn summarize_interpretation(interpretation: &Interpretation) -> String {
    let mut lines = Vec::new();
    if let Some(summary) = &interpretation.summary {
        lines.push(format!("Summary: {summary}"));
    }
    if !interpretation.concepts.is_empty() {
        lines.push("Concepts:".to_string());
        lines.extend(
            interpretation
                .concepts
                .iter()
                .take(12)
                .map(|concept| format!("- {}: {}", concept.term, concept.definition)),
        );
    }
    lines.push("Important blocks:".to_string());
    lines.extend(interpretation.blocks.iter().take(20).filter_map(block_line));
    limit_chars(&lines.join("\n"), 6_000)
}

fn block_line(block: &Block) -> Option<String> {
    match block {
        Block::Section { num, title, .. } => Some(format!("Section {num}: {title}")),
        Block::Paragraph { text, .. } => Some(format!("Paragraph: {}", limit_chars(text, 220))),
        Block::Quote { text, cite, .. } => Some(format!(
            "Quote{}: {}",
            cite.as_ref()
                .map(|cite| format!(" ({cite})"))
                .unwrap_or_default(),
            limit_chars(text, 180)
        )),
        Block::Comparison { columns, rows, .. } => Some(format!(
            "Comparison [{}]: {} rows",
            columns.join(" / "),
            rows.len()
        )),
        Block::ConceptCard {
            term, definition, ..
        } => Some(format!("ConceptCard: {term} - {definition}")),
        _ => None,
    }
}

fn format_search_results(results: &[WebSearchResult]) -> Vec<String> {
    results
        .iter()
        .take(4)
        .map(|result| {
            format!(
                "- {}{} — {}",
                result.title,
                result
                    .url
                    .as_ref()
                    .map(|url| format!(" {url}"))
                    .unwrap_or_default(),
                result.snippet
            )
        })
        .collect()
}

fn limit_chars(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        let truncated = text.chars().take(max_chars).collect::<String>();
        format!("{truncated}...")
    }
}
