use serde::Deserialize;
use uuid::Uuid;

use crate::domain::interpretation::{
    Block, ChartDataPoint, ComparisonRow, Concept, Interpretation, QuizOption, Stat, TimelineItem,
};
use crate::error::{AppError, AppResult};
use crate::llm::LlmClient;
use crate::prompt;

/// 解读器：编排 LLM 调用，把论文文本转成 Interpretation
#[derive(Clone)]
pub struct Interpreter {
    llm: LlmClient,
}

impl Interpreter {
    pub fn new(llm: LlmClient) -> Self {
        Self { llm }
    }

    /// 对一篇论文执行完整解读
    pub async fn interpret(
        &self,
        paper_id: Uuid,
        title: &str,
        text: &str,
    ) -> AppResult<Interpretation> {
        if !self.llm.is_configured() {
            return Err(AppError::LlmNotConfigured);
        }

        let user_msg = prompt::user_interpret(title, text);
        let value = self
            .llm
            .call_json(prompt::SYSTEM_INTERPRET, &user_msg)
            .await?;

        let raw: RawInterpretation = serde_json::from_value(value)
            .map_err(|e| AppError::InvalidLlmOutput(format!("解读结构解析失败: {e}")))?;

        // 转成 Interpretation
        let blocks: Vec<Block> = raw
            .blocks
            .into_iter()
            .map(block_from_raw)
            .collect::<Result<_, _>>()?;

        Ok(Interpretation {
            paper_id,
            blocks,
            concepts: raw.concepts,
            summary: Some(raw.summary),
        })
    }
}

// ── LLM 输出的中间结构（松散解析，做字段映射）──────────────

#[derive(Deserialize)]
struct RawInterpretation {
    #[serde(default)]
    summary: String,
    blocks: Vec<RawBlock>,
    #[serde(default)]
    concepts: Vec<Concept>,
}

#[derive(Deserialize)]
struct RawBlock {
    #[serde(rename = "type")]
    btype: String,
    #[serde(default)]
    id: String,
    #[serde(default)]
    num: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    text: String,
    #[serde(default)]
    cite: Option<String>,
    #[serde(default)]
    stats: Vec<RawStat>,
    #[serde(default)]
    term: String,
    #[serde(default)]
    definition: String,
    #[serde(default)]
    icon: Option<String>,
    #[serde(default)]
    items: Vec<RawTimelineItem>,
    #[serde(default)]
    columns: Vec<String>,
    #[serde(default)]
    rows: Vec<RawComparisonRow>,
    #[serde(default)]
    question: String,
    #[serde(default)]
    options: Vec<RawQuizOption>,
    #[serde(default)]
    explain: String,
    #[serde(default)]
    lang: String,
    #[serde(default)]
    code: String,
    #[serde(default)]
    html: String,
    #[serde(default)]
    svg: String,
    #[serde(default)]
    caption: String,
    #[serde(default)]
    chart_type: String,
    #[serde(default)]
    data: Vec<RawChartDataPoint>,
    #[serde(default)]
    x_label: String,
    #[serde(default)]
    y_label: String,
}

#[derive(Deserialize)]
struct RawStat {
    value: String,
    label: String,
}

#[derive(Deserialize)]
struct RawChartDataPoint {
    label: String,
    value: f64,
}

#[derive(Deserialize)]
struct RawTimelineItem {
    year: String,
    title: String,
    body: String,
}

#[derive(Deserialize)]
struct RawComparisonRow {
    label: String,
    cells: Vec<String>,
}

#[derive(Deserialize)]
struct RawQuizOption {
    text: String,
    correct: bool,
}

fn block_from_raw(r: RawBlock) -> AppResult<Block> {
    let id = if r.id.is_empty() {
        format!("{}-{}", r.btype, uuid::Uuid::new_v4())[..12].to_string()
    } else {
        r.id
    };
    match r.btype.as_str() {
        "section" => Ok(Block::Section {
            id,
            num: r.num,
            title: r.title,
        }),
        "paragraph" => Ok(Block::Paragraph { id, text: r.text }),
        "quote" => Ok(Block::Quote {
            id,
            text: r.text,
            cite: r.cite,
        }),
        "stat_row" => Ok(Block::StatRow {
            id,
            stats: r
                .stats
                .into_iter()
                .map(|s| Stat {
                    value: s.value,
                    label: s.label,
                })
                .collect(),
        }),
        "concept_card" => Ok(Block::ConceptCard {
            id,
            term: r.term,
            definition: r.definition,
            icon: r.icon,
        }),
        "timeline" => Ok(Block::Timeline {
            id,
            items: r
                .items
                .into_iter()
                .map(|i| TimelineItem {
                    year: i.year,
                    title: i.title,
                    body: i.body,
                })
                .collect(),
        }),
        "comparison" => Ok(Block::Comparison {
            id,
            columns: r.columns,
            rows: r
                .rows
                .into_iter()
                .map(|r| ComparisonRow {
                    label: r.label,
                    cells: r.cells,
                })
                .collect(),
        }),
        "quiz" => Ok(Block::Quiz {
            id,
            question: r.question,
            options: r
                .options
                .into_iter()
                .map(|o| QuizOption {
                    text: o.text,
                    correct: o.correct,
                })
                .collect(),
            explain: r.explain,
        }),
        "code_fragment" => Ok(Block::CodeFragment {
            id,
            lang: r.lang,
            code: r.code,
        }),
        "custom_html" => Ok(Block::CustomHtml { id, html: r.html }),
        "figure" => Ok(Block::Figure {
            id,
            svg: r.svg,
            caption: if r.caption.is_empty() {
                None
            } else {
                Some(r.caption)
            },
        }),
        "chart" => Ok(Block::Chart {
            id,
            chart_type: r.chart_type,
            title: if r.title.is_empty() {
                None
            } else {
                Some(r.title)
            },
            data: r
                .data
                .into_iter()
                .map(|d| ChartDataPoint {
                    label: d.label,
                    value: d.value,
                })
                .collect(),
            x_label: if r.x_label.is_empty() {
                None
            } else {
                Some(r.x_label)
            },
            y_label: if r.y_label.is_empty() {
                None
            } else {
                Some(r.y_label)
            },
        }),
        "diagram" => Ok(Block::Diagram {
            id,
            svg: r.svg,
            caption: if r.caption.is_empty() {
                None
            } else {
                Some(r.caption)
            },
        }),
        other => Err(AppError::InvalidLlmOutput(format!(
            "未知的 block 类型: {other}"
        ))),
    }
}
