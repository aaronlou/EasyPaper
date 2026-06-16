use crate::error::{AppError, AppResult};

/// PDF 文本提取结果
#[derive(Debug, Clone)]
pub struct ExtractResult {
    pub full_text: String,
    pub title: String,
    pub authors: Vec<String>,
}

/// 从 PDF 字节流提取文本 + 启发式推断标题/作者
///
/// pdf-extract 0.7 只接受文件路径（AsRef<Path>），所以先把 bytes
/// 写入临时文件，提取完再删除。
pub fn extract_text(pdf_bytes: &[u8]) -> AppResult<ExtractResult> {
    // 写到临时文件
    let tmp_dir = std::env::temp_dir();
    let tmp_path = tmp_dir.join(format!("easypaper_{}.pdf", uuid::Uuid::new_v4()));
    std::fs::write(&tmp_path, pdf_bytes)?;

    let full_text = pdf_extract::extract_text(&tmp_path)
        .map_err(|e| AppError::PdfExtract(e.to_string()))?;

    // 清理临时文件
    let _ = std::fs::remove_file(&tmp_path);

    // 启发式：从文本头部推断标题和作者
    let (title, authors) = infer_metadata(&full_text);

    Ok(ExtractResult {
        full_text,
        title,
        authors,
    })
}

/// 极简启发式：取第一页前若干非空行作为标题候选，过滤掉常见的页眉/版权行
fn infer_metadata(text: &str) -> (String, Vec<String>) {
    let head: Vec<&str> = text
        .lines()
        .take(40)
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    // 找标题：第一个长度合理、不像编号/页眉的行
    let title = head
        .iter()
        .find(|l| {
            let len = l.chars().count();
            len >= 8 && len <= 200
                && !l.starts_with("http")
                && !l.contains("©")
                && !l.starts_with("Abstract")
                && !l.chars().next().is_some_and(|c| c.is_ascii_digit() && len < 30)
        })
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Untitled Paper".to_string());

    // 找作者：标题之后、Abstract 之前的行
    let title_idx = head.iter().position(|l| *l == title).unwrap_or(0);
    let authors: Vec<String> = head
        .iter()
        .skip(title_idx + 1)
        .take(8)
        .take_while(|l| {
            !l.starts_with("Abstract")
                && !l.starts_with("ABSTRACT")
                && !l.contains("Abstract")
        })
        .filter(|l| {
            // 作者行特征：包含逗号分隔的人名，或单个人名
            let len = l.chars().count();
            len < 200
                && (l.contains(',') || l.split_whitespace().count() <= 6)
                && !l.contains("://")
                && !l.contains('@')
        })
        .flat_map(|l| {
            l.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty() && s.chars().count() < 60)
        })
        .take(10)
        .collect();

    (title, authors)
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
}
