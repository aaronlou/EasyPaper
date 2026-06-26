use crate::application::ports::{ExtractedPaperText, PdfExtractor};
use crate::error::{AppError, AppResult};

/// 从 PDF 字节流提取文本 + 启发式推断标题/作者
///
/// pdf-extract 0.7 只接受文件路径（AsRef<Path>），所以先把 bytes
/// 写入临时文件，提取完再删除。
pub fn extract_text(pdf_bytes: &[u8]) -> AppResult<ExtractedPaperText> {
    let mut tmp_file = tempfile::Builder::new()
        .prefix("easypaper_")
        .suffix(".pdf")
        .tempfile()?;
    std::io::Write::write_all(&mut tmp_file, pdf_bytes)?;
    let tmp_path = tmp_file.path().to_path_buf();

    let full_text =
        pdf_extract::extract_text(&tmp_path).map_err(|e| AppError::PdfExtract(e.to_string()))?;

    // 启发式：从文本头部推断标题和作者
    let (title, authors) = infer_metadata(&full_text);

    Ok(ExtractedPaperText {
        full_text,
        title,
        authors,
    })
}

#[derive(Clone)]
pub struct PdfExtractAdapter;

#[async_trait::async_trait]
impl PdfExtractor for PdfExtractAdapter {
    async fn extract(&self, pdf_bytes: &[u8]) -> AppResult<ExtractedPaperText> {
        extract_text(pdf_bytes)
    }
}

const UNTITLED_PAPER_TITLE: &str = "Untitled Paper";

#[derive(Debug, Clone)]
struct InferredTitle {
    text: String,
    end_index: usize,
    score: i32,
}

/// 启发式：取第一页前若干非空行作为标题候选，过滤掉常见页眉、章节标题和正文片段。
fn infer_metadata(text: &str) -> (String, Vec<String>) {
    let head: Vec<String> = text
        .lines()
        .take(60)
        .map(normalize_metadata_line)
        .filter(|l| !l.is_empty())
        .collect();

    let abstract_idx = head
        .iter()
        .position(|line| starts_metadata_section(line, "abstract"))
        .unwrap_or(head.len());

    let inferred_title = infer_title_from_head(&head, abstract_idx);
    let title = inferred_title
        .as_ref()
        .map(|candidate| candidate.text.clone())
        .unwrap_or_else(|| UNTITLED_PAPER_TITLE.to_string());
    let authors = inferred_title
        .as_ref()
        .map(|candidate| infer_authors(&head, candidate.end_index + 1, abstract_idx))
        .unwrap_or_default();

    (title, authors)
}

fn normalize_metadata_line(line: &str) -> String {
    line.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn infer_title_from_head(head: &[String], abstract_idx: usize) -> Option<InferredTitle> {
    let scan_limit = abstract_idx.min(head.len()).min(28);
    let mut best: Option<InferredTitle> = None;

    for index in 0..scan_limit {
        let line = &head[index];
        if !is_title_candidate(line) {
            continue;
        }

        let (candidate, end_index) = collect_title_candidate(head, index, scan_limit);
        if !is_title_candidate(&candidate) {
            continue;
        }

        let score = title_score(&candidate, index);
        if score < 12 {
            continue;
        }

        let should_replace = best.as_ref().is_none_or(|current| score > current.score);
        if should_replace {
            best = Some(InferredTitle {
                text: candidate,
                end_index,
                score,
            });
        }
    }

    best
}

fn collect_title_candidate(
    head: &[String],
    start_index: usize,
    scan_limit: usize,
) -> (String, usize) {
    let mut candidate = head[start_index].clone();
    let mut end_index = start_index;

    while end_index + 1 < scan_limit
        && candidate.chars().count() < 140
        && candidate.split_whitespace().count() < 18
    {
        let next = &head[end_index + 1];
        if !should_join_title_line(&candidate, next) {
            break;
        }
        candidate.push(' ');
        candidate.push_str(next);
        end_index += 1;
    }

    (candidate, end_index)
}

fn infer_authors(head: &[String], start_index: usize, abstract_idx: usize) -> Vec<String> {
    let mut authors = Vec::new();

    for line in head.iter().enumerate().skip(start_index).take(8) {
        if line.0 >= abstract_idx || starts_metadata_section(line.1, "abstract") {
            break;
        }
        if !is_author_candidate(line.1) {
            continue;
        }

        authors.extend(
            line.1
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty() && s.chars().count() < 60),
        );

        if authors.len() >= 10 {
            authors.truncate(10);
            break;
        }
    }

    authors
}

fn starts_metadata_section(line: &str, keyword: &str) -> bool {
    let lower = line.trim_start().to_ascii_lowercase();
    lower == keyword
        || lower.starts_with(&format!("{keyword} "))
        || lower.starts_with(&format!("{keyword}:"))
        || lower.starts_with(&format!("{keyword}."))
}

fn is_title_candidate(line: &str) -> bool {
    let len = line.chars().count();
    if !(8..=180).contains(&len) {
        return false;
    }

    let lower = line.to_ascii_lowercase();
    if lower.starts_with("http")
        || lower.contains("://")
        || lower.starts_with("arxiv:")
        || line.contains('@')
        || line.contains("©")
        || lower.contains("copyright")
    {
        return false;
    }

    if ["abstract", "keywords", "references", "acknowledgments"]
        .iter()
        .any(|keyword| starts_metadata_section(line, keyword))
    {
        return false;
    }

    !looks_like_numbered_section(line)
        && !looks_like_affiliation_line(line)
        && !looks_like_body_fragment(line)
}

fn should_join_title_line(current: &str, next: &str) -> bool {
    let next_len = next.chars().count();
    if !(4..=110).contains(&next_len) {
        return false;
    }

    let current_word_count = current.split_whitespace().count();
    if current_word_count <= 2 && !current.contains(':') && !current.contains('：') {
        return false;
    }

    if starts_metadata_section(next, "abstract")
        || starts_metadata_section(next, "keywords")
        || next.contains('@')
        || next.contains("://")
        || looks_like_numbered_section(next)
        || looks_like_affiliation_line(next)
        || looks_like_author_name_list(next)
        || looks_like_body_fragment(next)
    {
        return false;
    }

    current_word_count < 16 && next.split_whitespace().count() <= 12
}

fn is_author_candidate(line: &str) -> bool {
    let len = line.chars().count();
    len < 200
        && (line.contains(',') || line.split_whitespace().count() <= 6)
        && !line.contains("://")
        && !line.contains('@')
        && !starts_metadata_section(line, "abstract")
        && !looks_like_numbered_section(line)
        && !looks_like_body_fragment(line)
}

fn looks_like_numbered_section(line: &str) -> bool {
    let Some(first_token) = line.split_whitespace().next() else {
        return false;
    };
    let marker = first_token.trim_end_matches(['.', ')', ':']);
    if marker.is_empty() || marker.len() > 2 || !marker.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    let lower = line[first_token.len()..].trim_start().to_ascii_lowercase();
    line.chars().count() < 70
        || [
            "introduction",
            "background",
            "related",
            "method",
            "results",
            "discussion",
        ]
        .iter()
        .any(|section| lower.starts_with(section))
}

fn looks_like_affiliation_line(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    [
        "department",
        "university",
        "institute",
        "laboratory",
        "college",
        "school of",
        "research center",
        "faculty",
    ]
    .iter()
    .any(|keyword| lower.contains(keyword))
}

fn looks_like_author_name_list(line: &str) -> bool {
    if !line.contains(',') {
        return false;
    }

    let parts: Vec<&str> = line
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    parts.len() >= 2
        && parts.iter().all(|part| {
            let words: Vec<&str> = part.split_whitespace().collect();
            !words.is_empty()
                && words.len() <= 4
                && words
                    .iter()
                    .all(|word| word.chars().next().is_some_and(|c| c.is_ascii_uppercase()))
        })
}

fn looks_like_body_fragment(line: &str) -> bool {
    let word_count = line.split_whitespace().count();
    if line.ends_with('.') || line.ends_with(';') || line.ends_with('。') || line.ends_with('；')
    {
        return true;
    }
    if (line.contains(';') || line.contains('；')) && word_count > 8 {
        return true;
    }

    let lower = format!(" {} ", line.to_ascii_lowercase());
    if word_count >= 10
        && [
            " however ",
            " therefore ",
            " this ",
            " that ",
            " aims to ",
            " used in ",
            " using the ",
            " can be ",
            " we ",
            " our ",
        ]
        .iter()
        .any(|phrase| lower.contains(phrase))
    {
        return true;
    }

    let punctuation_count = line
        .chars()
        .filter(|c| matches!(c, '.' | ',' | ';' | '，' | '；'))
        .count();
    if word_count >= 12 && punctuation_count >= 3 {
        return true;
    }

    first_alphabetic_is_lowercase(line) && word_count >= 6 && !line.contains(':')
}

fn first_alphabetic_is_lowercase(line: &str) -> bool {
    line.chars()
        .find(|c| c.is_alphabetic())
        .is_some_and(|c| c.is_lowercase())
}

fn title_score(line: &str, index: usize) -> i32 {
    let word_count = line.split_whitespace().count();
    let mut score = line.chars().count().min(110) as i32 - (index as i32 * 4);
    if line.contains(':') || line.contains('：') {
        score += 40;
    }
    if line.contains(',') {
        score -= 35;
    }
    if word_count >= 4 {
        score += 14;
    }
    if word_count <= 2 {
        score -= 30;
    }
    if word_count > 24 {
        score -= 40;
    }
    if first_alphabetic_is_lowercase(line) && !line.contains(':') {
        score -= 30;
    }
    score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_metadata_basic() {
        let text = "Some Header\n\nBigtable: A Distributed Storage System\n\nFay Chang, Jeffrey Dean, Sanjay Ghemawat\n\nAbstract\nThis paper...";
        let (title, authors) = infer_metadata(text);
        assert!(title.contains("Bigtable"));
        assert!(!authors.is_empty());
    }

    #[test]
    fn infer_metadata_rejects_body_fragment_as_title() {
        let text = "role using the same criteria used in the inner loop. This cyclical process aims to progressively enhance data quality;\n\n1 Introduction, However\n\nAbstract\nThis paper...";
        let (title, authors) = infer_metadata(text);
        assert_eq!(title, UNTITLED_PAPER_TITLE);
        assert!(authors.is_empty());
    }

    #[test]
    fn infer_metadata_joins_multiline_title() {
        let text = "Learning from Human Feedback\nfor Language Model Alignment\n\nAda Lovelace, Alan Turing\n\nAbstract\nThis paper...";
        let (title, authors) = infer_metadata(text);
        assert_eq!(
            title,
            "Learning from Human Feedback for Language Model Alignment"
        );
        assert_eq!(authors, vec!["Ada Lovelace", "Alan Turing"]);
    }

    #[test]
    fn infer_metadata_keeps_lowercase_brand_title_with_colon() {
        let text = "vLLM: Easy, Fast, and Cheap LLM Serving with PagedAttention\n\nWoosuk Kwon, Zhuohan Li\n\nAbstract\nThis paper...";
        let (title, authors) = infer_metadata(text);
        assert_eq!(
            title,
            "vLLM: Easy, Fast, and Cheap LLM Serving with PagedAttention"
        );
        assert_eq!(authors, vec!["Woosuk Kwon", "Zhuohan Li"]);
    }
}
