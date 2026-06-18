use uuid::Uuid;

use crate::application::paper_workflow::PaperWorkflow;
use crate::domain::interpretation::Block;
use crate::domain::research::WebSearchResult;
use crate::error::{AppError, AppResult};
use crate::models::api::{ConceptExpansion, ReferenceLink, ResearchStep};
use crate::prompt;

const MAX_PAPER_CONTEXT_CHARS: usize = 9_000;
const MAX_REFERENCE_CONTEXT_CHARS: usize = 4_000;
const MAX_WEB_CONTEXT_CHARS: usize = 3_500;

impl PaperWorkflow {
    /// 对单个概念执行 Feynman 式深潜：回到原文证据、补足参考线索、再生成可讲解内容。
    pub async fn expand_concept(
        &self,
        paper_id: Uuid,
        concept_id: String,
    ) -> AppResult<ConceptExpansion> {
        if let Some(cached) = self
            .concept_expansions
            .get_concept_expansion(paper_id, &concept_id, self.concept_cache_ttl_days)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            tracing::info!(
                paper_id = %paper_id,
                concept_id = %concept_id,
                "命中概念深潜缓存"
            );
            return Ok(cached);
        }

        let paper = self
            .papers
            .get_paper(paper_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound(format!("论文 {paper_id} 不存在")))?;

        let interpretation = self
            .papers
            .get_interpretation(paper_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound(format!("论文 {paper_id} 尚未解读完成")))?;

        let concept = interpretation
            .concepts
            .iter()
            .find(|concept| concept.id == concept_id)
            .cloned();

        let (term, definition) = match concept {
            Some(concept) => (concept.term, concept.definition),
            None => {
                let mut found = None;
                for block in &interpretation.blocks {
                    if let Block::ConceptCard {
                        id,
                        term,
                        definition,
                        ..
                    } = block
                        && id == &concept_id
                    {
                        found = Some((term.clone(), definition.clone()));
                        break;
                    }
                }
                found.ok_or_else(|| AppError::NotFound(format!("概念 {concept_id} 不存在")))?
            }
        };

        if !self.llm.is_configured() {
            return Err(AppError::LlmNotConfigured);
        }

        let paper_context = build_concept_context(paper.full_text(), &term, &definition);
        let reference_candidates =
            extract_reference_candidates(paper.full_text(), &term, &definition);
        let reference_context = format_reference_context(&reference_candidates);
        let search_query = build_search_query(paper.title(), &term);
        let web_results = self.research.search(&search_query).await;
        let web_context = format_web_context(&web_results);

        let user_msg = prompt::user_expand_concept(
            paper.title(),
            &term,
            &definition,
            &paper_context,
            &reference_context,
            &web_context,
        );
        let value = self
            .llm
            .call_json(prompt::SYSTEM_EXPAND_CONCEPT, &user_msg)
            .await
            .map_err(|e| AppError::LlmCall(format!("概念深潜失败: {e}")))?;

        let mut expansion: ConceptExpansion = serde_json::from_value(value)
            .map_err(|e| AppError::InvalidLlmOutput(format!("概念深潜解析失败: {e}")))?;

        enrich_expansion_with_research_context(
            &mut expansion,
            &term,
            &reference_candidates,
            &web_results,
            &search_query,
        );

        self.concept_expansions
            .save_concept_expansion(paper_id, &concept_id, &expansion)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(expansion)
    }

    pub(super) fn spawn_concept_prewarm(&self, paper_id: Uuid) {
        if self.concept_prewarm_limit == 0 || !self.llm.is_configured() {
            return;
        }

        let workflow = self.clone();
        tokio::spawn(async move {
            let interpretation = match workflow.papers.get_interpretation(paper_id).await {
                Ok(Some(interpretation)) => interpretation,
                Ok(None) => return,
                Err(err) => {
                    tracing::warn!(paper_id = %paper_id, "概念预热读取解读失败: {err}");
                    return;
                }
            };

            let concept_ids = interpretation
                .concepts
                .iter()
                .take(workflow.concept_prewarm_limit)
                .map(|concept| concept.id.clone())
                .collect::<Vec<_>>();

            if concept_ids.is_empty() {
                return;
            }

            tracing::info!(
                paper_id = %paper_id,
                count = concept_ids.len(),
                "开始后台预热概念深潜缓存"
            );

            for concept_id in concept_ids {
                if let Err(err) = workflow.expand_concept(paper_id, concept_id.clone()).await {
                    tracing::warn!(
                        paper_id = %paper_id,
                        concept_id = %concept_id,
                        "概念深潜预热失败: {err}"
                    );
                }
            }

            tracing::info!(paper_id = %paper_id, "后台概念深潜缓存预热完成");
        });
    }
}

fn build_concept_context(full_text: &str, term: &str, definition: &str) -> String {
    let mut chunks = Vec::new();
    chunks.push(format!("概念定义：{definition}"));

    for snippet in snippets_around_terms(full_text, &[term], 5, 900) {
        chunks.push(snippet);
    }

    if chunks.len() == 1 {
        chunks.push(take_chars(full_text, MAX_PAPER_CONTEXT_CHARS.min(4_000)));
    }

    limit_chars(&chunks.join("\n\n---\n\n"), MAX_PAPER_CONTEXT_CHARS)
}

fn snippets_around_terms(
    text: &str,
    terms: &[&str],
    max_snippets: usize,
    radius_chars: usize,
) -> Vec<String> {
    let text_lower = text.to_lowercase();
    let needles: Vec<String> = terms
        .iter()
        .flat_map(|term| term_keywords(term))
        .filter(|term| term.chars().count() >= 3)
        .map(|term| term.to_lowercase())
        .collect();

    let mut positions = Vec::new();
    for needle in needles {
        for (pos, _) in text_lower.match_indices(&needle) {
            positions.push(pos);
        }
    }
    positions.sort_unstable();
    positions.dedup_by(|a, b| a.abs_diff(*b) < radius_chars / 2);

    positions
        .into_iter()
        .take(max_snippets)
        .map(|pos| {
            let start = byte_floor_for_char_radius(text, pos, radius_chars, false);
            let end = byte_floor_for_char_radius(text, pos, radius_chars, true);
            let snippet = text[start..end].replace('\n', " ");
            collapse_spaces(&snippet)
        })
        .filter(|snippet| !snippet.is_empty())
        .collect()
}

fn term_keywords(term: &str) -> Vec<String> {
    let mut keywords = Vec::new();
    let clean = term
        .trim()
        .trim_matches(|c: char| c == '(' || c == ')' || c == '（' || c == '）');
    if !clean.is_empty() {
        keywords.push(clean.to_string());
    }

    for segment in clean.split(|c: char| {
        c == '/'
            || c == '|'
            || c == ','
            || c == ';'
            || c == '，'
            || c == '；'
            || c == '('
            || c == ')'
            || c == '（'
            || c == '）'
    }) {
        let segment = segment.trim();
        if segment.chars().count() >= 3 {
            keywords.push(segment.to_string());
        }
    }

    keywords.sort();
    keywords.dedup();
    keywords
}

fn byte_floor_for_char_radius(
    text: &str,
    byte_pos: usize,
    radius_chars: usize,
    forward: bool,
) -> usize {
    let pos = byte_pos.min(text.len());
    if forward {
        text[pos..]
            .char_indices()
            .nth(radius_chars)
            .map(|(offset, _)| pos + offset)
            .unwrap_or(text.len())
    } else {
        text[..pos]
            .char_indices()
            .rev()
            .nth(radius_chars)
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }
}

fn extract_reference_candidates(
    full_text: &str,
    term: &str,
    definition: &str,
) -> Vec<ReferenceLink> {
    let references = reference_section(full_text);
    let mut candidates = parse_reference_lines(&references);
    let keywords = term_keywords(term)
        .into_iter()
        .chain(term_keywords(definition))
        .map(|keyword| keyword.to_lowercase())
        .collect::<Vec<_>>();

    candidates.sort_by_key(|item| {
        let haystack = format!(
            "{} {} {}",
            item.title,
            item.authors.join(" "),
            item.relevance
        )
        .to_lowercase();
        let hits = keywords
            .iter()
            .filter(|keyword| keyword.chars().count() >= 4 && haystack.contains(keyword.as_str()))
            .count();
        std::cmp::Reverse(hits)
    });

    candidates.into_iter().take(6).collect()
}

fn reference_section(full_text: &str) -> String {
    let lower = full_text.to_lowercase();
    let markers = [
        "\nreferences",
        "\nreference",
        "\nbibliography",
        "\n参考文献",
    ];
    let start = markers
        .iter()
        .filter_map(|marker| lower.rfind(marker))
        .min()
        .unwrap_or_else(|| {
            let total = full_text.chars().count();
            char_to_byte(full_text, total.saturating_sub(8_000))
        });

    limit_chars(&full_text[start..], MAX_REFERENCE_CONTEXT_CHARS * 2)
}

fn parse_reference_lines(references: &str) -> Vec<ReferenceLink> {
    let mut entries = Vec::new();
    let mut current = String::new();

    for line in references.lines() {
        let line = collapse_spaces(line);
        if line.is_empty() {
            continue;
        }

        let starts_entry = looks_like_reference_start(&line);
        if starts_entry && !current.is_empty() {
            if let Some(entry) = reference_link_from_text(&current) {
                entries.push(entry);
            }
            current.clear();
        }

        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(&line);
    }

    if !current.is_empty()
        && let Some(entry) = reference_link_from_text(&current)
    {
        entries.push(entry);
    }

    entries
}

fn looks_like_reference_start(line: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.starts_with('[') {
        return true;
    }
    let mut chars = trimmed.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_digit())
        && matches!(chars.next(), Some('.') | Some(']') | Some(')'))
}

fn reference_link_from_text(raw: &str) -> Option<ReferenceLink> {
    let raw = collapse_spaces(raw);
    if raw.chars().count() < 24 {
        return None;
    }

    let year = find_year(&raw);
    let url = find_url(&raw);
    let title = infer_reference_title(&raw);
    let authors = infer_reference_authors(&raw);

    Some(ReferenceLink {
        title,
        authors,
        venue: None,
        year,
        url,
        relevance: limit_chars(&raw, 260),
        source_type: "paper_reference".to_string(),
    })
}

fn infer_reference_title(raw: &str) -> String {
    let stripped = raw
        .trim_start_matches(|c: char| {
            c == '[' || c == ']' || c == '.' || c == ')' || c.is_ascii_digit()
        })
        .trim();
    let pieces: Vec<&str> = stripped.split(". ").collect();
    if pieces.len() >= 2 {
        limit_chars(pieces[1].trim_matches('"'), 140)
    } else {
        limit_chars(stripped, 140)
    }
}

fn infer_reference_authors(raw: &str) -> Vec<String> {
    let stripped = raw
        .trim_start_matches(|c: char| {
            c == '[' || c == ']' || c == '.' || c == ')' || c.is_ascii_digit()
        })
        .trim();
    let first = stripped.split(". ").next().unwrap_or("").trim();
    if first.is_empty() || first.chars().count() > 180 {
        Vec::new()
    } else {
        first
            .split(',')
            .take(4)
            .map(collapse_spaces)
            .filter(|author| !author.is_empty())
            .collect()
    }
}

fn find_year(raw: &str) -> Option<String> {
    for token in raw.split(|c: char| !c.is_ascii_alphanumeric()) {
        if token.len() == 4
            && let Ok(year) = token.parse::<u16>()
            && (1900..=2100).contains(&year)
        {
            return Some(token.to_string());
        }
    }
    None
}

fn find_url(raw: &str) -> Option<String> {
    raw.split_whitespace()
        .find(|part| part.starts_with("http://") || part.starts_with("https://"))
        .map(|part| part.trim_end_matches(['.', ',', ')', ']']))
        .map(str::to_string)
}

fn format_reference_context(references: &[ReferenceLink]) -> String {
    if references.is_empty() {
        return "未在 PDF 文本中稳定识别到参考文献条目。".to_string();
    }

    limit_chars(
        &references
            .iter()
            .enumerate()
            .map(|(idx, reference)| {
                format!(
                    "{}. {}{}{} — {}",
                    idx + 1,
                    reference.title,
                    reference
                        .year
                        .as_ref()
                        .map(|year| format!(" ({year})"))
                        .unwrap_or_default(),
                    reference
                        .url
                        .as_ref()
                        .map(|url| format!(" {url}"))
                        .unwrap_or_default(),
                    reference.relevance
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
        MAX_REFERENCE_CONTEXT_CHARS,
    )
}

fn build_search_query(paper_title: &str, term: &str) -> String {
    format!("{term} academic paper explanation {paper_title}")
}

fn format_web_context(results: &[WebSearchResult]) -> String {
    if results.is_empty() {
        return "未配置外部检索，或检索暂无可用结果。".to_string();
    }

    limit_chars(
        &results
            .iter()
            .enumerate()
            .map(|(idx, result)| {
                format!(
                    "{}. {}{} — {}",
                    idx + 1,
                    result.title,
                    result
                        .url
                        .as_ref()
                        .map(|url| format!(" {url}"))
                        .unwrap_or_default(),
                    collapse_spaces(&result.snippet)
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
        MAX_WEB_CONTEXT_CHARS,
    )
}

fn enrich_expansion_with_research_context(
    expansion: &mut ConceptExpansion,
    term: &str,
    references: &[ReferenceLink],
    web_results: &[WebSearchResult],
    search_query: &str,
) {
    if expansion.term.trim().is_empty() {
        expansion.term = term.to_string();
    }

    if expansion.reference_links.is_empty() {
        expansion
            .reference_links
            .extend(references.iter().take(4).cloned());
    }

    let existing_urls = expansion
        .reference_links
        .iter()
        .filter_map(|link| link.url.clone())
        .collect::<std::collections::HashSet<_>>();

    for result in web_results.iter().take(3) {
        if let Some(url) = &result.url
            && existing_urls.contains(url)
        {
            continue;
        }
        expansion.reference_links.push(ReferenceLink {
            title: if result.title.is_empty() {
                "外部检索结果".to_string()
            } else {
                result.title.clone()
            },
            authors: Vec::new(),
            venue: None,
            year: None,
            url: result.url.clone(),
            relevance: limit_chars(&result.snippet, 180),
            source_type: "web".to_string(),
        });
    }

    if expansion.research_trail.is_empty() {
        expansion.research_trail = vec![
            ResearchStep {
                question: "这个概念在论文中解决了什么问题？".to_string(),
                action: "检查包含该术语的论文上下文摘录。".to_string(),
                finding: "将概念解释限定在论文的任务、方法或实验语境内。".to_string(),
                confidence: "medium".to_string(),
            },
            ResearchStep {
                question: "它和已有研究有什么关系？".to_string(),
                action: "抽取 References 区域中的候选条目。".to_string(),
                finding: if references.is_empty() {
                    "当前 PDF 文本未稳定识别到可用参考文献条目。".to_string()
                } else {
                    format!("找到 {} 条可作为延伸阅读的参考文献线索。", references.len())
                },
                confidence: if references.is_empty() {
                    "low"
                } else {
                    "medium"
                }
                .to_string(),
            },
            ResearchStep {
                question: "是否需要跳出本文补充背景？".to_string(),
                action: "检查可选外部检索摘要。".to_string(),
                finding: if web_results.is_empty() {
                    "未配置外部检索或暂无结果，因此仅基于论文材料解释。".to_string()
                } else {
                    format!(
                        "检索到 {} 条外部摘要，用于补充背景而非替代论文证据。",
                        web_results.len()
                    )
                },
                confidence: if web_results.is_empty() {
                    "low"
                } else {
                    "medium"
                }
                .to_string(),
            },
        ];
    }

    if expansion.external_queries.is_empty() {
        expansion.external_queries = vec![
            search_query.to_string(),
            format!("{term} survey paper"),
            format!("{term} tutorial"),
        ];
    }
}

fn take_chars(text: &str, max_chars: usize) -> String {
    text.chars().take(max_chars).collect()
}

fn limit_chars(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        let mut truncated = text.chars().take(max_chars).collect::<String>();
        truncated.push_str("\n[... truncated ...]");
        truncated
    }
}

fn char_to_byte(text: &str, char_idx: usize) -> usize {
    text.char_indices()
        .nth(char_idx)
        .map(|(idx, _)| idx)
        .unwrap_or(text.len())
}

fn collapse_spaces(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concept_context_prefers_local_snippets() {
        let text = "Intro text.\n\nAttention is a mechanism for routing information between tokens. It lets the model weigh context.\n\nConclusion.";

        let context = build_concept_context(text, "Attention", "A routing mechanism");

        assert!(context.contains("概念定义：A routing mechanism"));
        assert!(context.contains("routing information"));
        assert!(!context.contains("[... truncated ...]"));
    }

    #[test]
    fn reference_candidates_keep_relevant_entries_first() {
        let text = r#"
Main body mentioning contrastive learning.

References
[1] Smith, A. Contrastive Learning for Representations. ICML. 2020.
[2] Doe, B. Unrelated Systems Paper. 2018.
"#;

        let references =
            extract_reference_candidates(text, "contrastive learning", "representation learning");

        assert_eq!(references.len(), 2);
        assert!(references[0].title.contains("Contrastive Learning"));
        assert_eq!(references[0].year.as_deref(), Some("2020"));
    }
}
