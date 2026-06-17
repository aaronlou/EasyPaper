use std::collections::HashMap;
use std::future::{self, Future};

use serde::{Deserialize, Deserializer, Serialize};
use tokio::task::JoinSet;
use uuid::Uuid;

use crate::domain::interpretation::{
    Block, ChartDataPoint, ComparisonRow, Concept, Interpretation, QuizOption, Stat,
};
use crate::error::{AppError, AppResult};
use crate::llm::LlmClient;
use crate::prompt;

const SLICE_CHARS: usize = 6_500;
const MAX_PARALLEL_READERS: usize = 4;

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
        self.interpret_with_progress(paper_id, title, text, |_, _, _| future::ready(()))
            .await
    }

    /// 多 Agent 解读：把论文拆给多个短上下文 reader 并发分析，再由本地 reducer
    /// 稳定组装成前端 Block。回调用于向应用服务报告 reader 完成进度。
    pub async fn interpret_with_progress<F, Fut>(
        &self,
        paper_id: Uuid,
        title: &str,
        text: &str,
        mut on_reader_done: F,
    ) -> AppResult<Interpretation>
    where
        F: FnMut(usize, usize, String) -> Fut,
        Fut: Future<Output = ()>,
    {
        if !self.llm.is_configured() {
            return Err(AppError::LlmNotConfigured);
        }

        let slices = select_analysis_slices(text);
        let total = slices.len();
        tracing::info!(paper_id = %paper_id, readers = total, "启动并行论文 reader agents");

        let mut readers = JoinSet::new();
        for slice in slices {
            let llm = self.llm.clone();
            let paper_title = title.to_string();
            readers.spawn(async move {
                let user_msg = prompt::user_analyze_slice(
                    &paper_title,
                    slice.index,
                    slice.total,
                    &slice.label,
                    &slice.text,
                );
                let value = llm
                    .call_json(prompt::SYSTEM_ANALYZE_SLICE, &user_msg)
                    .await?;
                let notes: SliceNotes = serde_json::from_value(value).map_err(|e| {
                    AppError::InvalidLlmOutput(format!(
                        "reader agent {} 笔记结构解析失败: {e}",
                        slice.index
                    ))
                })?;

                Ok::<ReaderOutput, AppError>(ReaderOutput {
                    index: slice.index,
                    label: slice.label,
                    notes,
                })
            });
        }

        let mut completed = 0;
        let mut outputs = Vec::new();
        let mut failures = Vec::new();
        while let Some(joined) = readers.join_next().await {
            completed += 1;
            match joined {
                Ok(Ok(output)) => {
                    tracing::info!(
                        paper_id = %paper_id,
                        reader = output.index,
                        label = %output.label,
                        "reader agent 完成"
                    );
                    on_reader_done(completed, total, output.label.clone()).await;
                    outputs.push(output);
                }
                Ok(Err(err)) => {
                    tracing::warn!(paper_id = %paper_id, "reader agent 失败: {err}");
                    on_reader_done(
                        completed,
                        total,
                        "一个 reader agent 返回失败，继续汇总其余结果".to_string(),
                    )
                    .await;
                    failures.push(err.to_string());
                }
                Err(err) => {
                    tracing::warn!(paper_id = %paper_id, "reader agent 任务中断: {err}");
                    on_reader_done(
                        completed,
                        total,
                        "一个 reader agent 任务中断，继续汇总其余结果".to_string(),
                    )
                    .await;
                    failures.push(err.to_string());
                }
            }
        }

        if outputs.is_empty() {
            return Err(AppError::LlmCall(format!(
                "所有并行 reader agents 都失败，无法生成解读: {}",
                failures.join(" | ")
            )));
        }

        outputs.sort_by_key(|output| output.index);
        Ok(build_interpretation_from_notes(paper_id, title, outputs))
    }
}

#[derive(Debug)]
struct AnalysisSlice {
    index: usize,
    total: usize,
    label: String,
    text: String,
}

#[derive(Debug)]
struct ReaderOutput {
    index: usize,
    label: String,
    notes: SliceNotes,
}

// ── 并行 reader agent 的短笔记结构 ──────────────

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct SliceNotes {
    #[serde(default)]
    slice_focus: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    core_ideas: Vec<String>,
    #[serde(default)]
    mechanisms: Vec<SliceMechanism>,
    #[serde(default)]
    concepts: Vec<SliceConcept>,
    #[serde(default)]
    evidence: Vec<SliceEvidence>,
    #[serde(default)]
    stats: Vec<SliceStat>,
    #[serde(default)]
    comparisons: Vec<SliceComparison>,
    #[serde(default)]
    quiz_questions: Vec<SliceQuiz>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct SliceMechanism {
    #[serde(default)]
    name: String,
    #[serde(default)]
    input: String,
    #[serde(default)]
    process: String,
    #[serde(default)]
    output: String,
    #[serde(default)]
    why: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct SliceConcept {
    #[serde(default)]
    term: String,
    #[serde(default)]
    definition: String,
    #[serde(default)]
    difficulty: String,
    #[serde(default)]
    related: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct SliceEvidence {
    #[serde(default)]
    claim: String,
    #[serde(default)]
    quote: String,
    #[serde(default)]
    cite: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct SliceStat {
    #[serde(default)]
    value: String,
    #[serde(default)]
    label: String,
    #[serde(default, deserialize_with = "optional_f64_from_any")]
    numeric_value: Option<f64>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct SliceComparison {
    #[serde(default)]
    dimension: String,
    #[serde(default)]
    baseline: String,
    #[serde(default)]
    paper_approach: String,
    #[serde(default)]
    lesson: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct SliceQuiz {
    #[serde(default)]
    question: String,
    #[serde(default)]
    correct_answer: String,
    #[serde(default)]
    distractors: Vec<String>,
    #[serde(default)]
    explanation: String,
}

fn select_analysis_slices(text: &str) -> Vec<AnalysisSlice> {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    if len == 0 {
        return vec![AnalysisSlice {
            index: 1,
            total: 1,
            label: "空文本".to_string(),
            text: String::new(),
        }];
    }

    let ranges = if len <= SLICE_CHARS {
        vec![(0, len, "全文")]
    } else if len <= SLICE_CHARS * MAX_PARALLEL_READERS {
        let mut ranges = Vec::new();
        let mut start = 0;
        while start < len {
            let end = (start + SLICE_CHARS).min(len);
            ranges.push((start, end, "连续片段"));
            start = end;
        }
        ranges
    } else {
        let window = SLICE_CHARS;
        let anchors = [
            0,
            (len / 3).saturating_sub(window / 2),
            (len * 2 / 3).saturating_sub(window / 2),
            len.saturating_sub(window),
        ];
        let labels = [
            "摘要/引言附近",
            "方法/系统设计附近",
            "实验/分析附近",
            "结论/参考附近",
        ];
        anchors
            .into_iter()
            .zip(labels)
            .map(|(start, label)| {
                let bounded_start = start.min(len.saturating_sub(1));
                (bounded_start, (bounded_start + window).min(len), label)
            })
            .collect()
    };

    let total = ranges.len();
    ranges
        .into_iter()
        .enumerate()
        .map(|(idx, (start, end, label))| AnalysisSlice {
            index: idx + 1,
            total,
            label: format!("{label} chars {start}-{end}"),
            text: chars[start..end].iter().collect(),
        })
        .collect()
}

fn build_interpretation_from_notes(
    paper_id: Uuid,
    title: &str,
    outputs: Vec<ReaderOutput>,
) -> Interpretation {
    let notes: Vec<SliceNotes> = outputs.into_iter().map(|output| output.notes).collect();
    let concepts = collect_concepts(&notes);
    let stats = collect_stats(&notes);
    let mechanisms = collect_mechanisms(&notes);
    let comparisons = collect_comparisons(&notes, &mechanisms);
    let evidence = collect_evidence(&notes);
    let summary = synthesize_summary(title, &notes);

    let mut blocks = Vec::new();
    blocks.push(Block::Section {
        id: "sec-1".to_string(),
        num: "01".to_string(),
        title: "先抓住这篇论文想解决的问题".to_string(),
    });
    blocks.push(Block::Paragraph {
        id: "p-1".to_string(),
        text: format!(
            "用费曼式自学的第一步看这篇论文：先不要急着背术语，而是把它压缩成一句可转述的话。{}",
            summary
        ),
    });

    if let Some(item) = evidence.first() {
        blocks.push(Block::Quote {
            id: "quote-1".to_string(),
            text: truncate_chars(&item.quote, 280),
            cite: item.cite.clone().filter(|cite| !cite.trim().is_empty()),
        });
    }

    if !stats.is_empty() {
        blocks.push(Block::StatRow {
            id: "stats-1".to_string(),
            stats: stats
                .iter()
                .take(4)
                .map(|stat| Stat {
                    value: truncate_chars(&stat.value, 24),
                    label: truncate_chars(&stat.label, 42),
                })
                .collect(),
        });
    }

    blocks.push(Block::Section {
        id: "sec-2".to_string(),
        num: "02".to_string(),
        title: "把机制拆成输入、处理、输出".to_string(),
    });
    blocks.push(Block::Paragraph {
        id: "p-2".to_string(),
        text: mechanism_paragraph(&mechanisms, &notes),
    });
    if !mechanisms.is_empty() {
        blocks.push(Block::Diagram {
            id: "diagram-1".to_string(),
            svg: build_mechanism_svg(&mechanisms),
            caption: Some("把论文机制拆成可讲给别人听的路径".to_string()),
        });
    }

    if !comparisons.is_empty() {
        blocks.push(Block::Comparison {
            id: "cmp-1".to_string(),
            columns: vec![
                "维度".to_string(),
                "常见理解/旧做法".to_string(),
                "论文做法".to_string(),
                "学习 takeaway".to_string(),
            ],
            rows: comparisons,
        });
    }

    blocks.push(Block::Section {
        id: "sec-3".to_string(),
        num: "03".to_string(),
        title: "关键概念不是名词表，而是思维工具".to_string(),
    });
    blocks.push(Block::Paragraph {
        id: "p-3".to_string(),
        text: "下面这些概念是理解论文的支点。点击任意卡片可以进入概念实验室，继续展开直觉、机制、例子、证据和自测。".to_string(),
    });
    for (idx, concept) in concepts.iter().take(6).enumerate() {
        blocks.push(Block::ConceptCard {
            id: format!("concept-card-{}", idx + 1),
            term: concept.term.clone(),
            definition: concept.definition.clone(),
            icon: None,
        });
    }

    let chart_data = chart_data_from_stats(&stats);
    if chart_data.len() >= 2 {
        blocks.push(Block::Chart {
            id: "chart-1".to_string(),
            chart_type: "bar".to_string(),
            title: Some("论文片段中出现的关键量化信号".to_string()),
            data: chart_data,
            x_label: Some("指标".to_string()),
            y_label: Some("数值".to_string()),
        });
    }

    blocks.push(Block::Section {
        id: "sec-4".to_string(),
        num: "04".to_string(),
        title: "最后用问题检验自己是否真的懂了".to_string(),
    });
    blocks.push(build_quiz_block(&notes, concepts.first()));
    blocks.push(Block::Paragraph {
        id: "p-4".to_string(),
        text: "如果你能不用论文原句、只用自己的话解释上面的机制和取舍，就已经进入费曼学习法最关键的一步。接下来可以点开概念实验室，把薄弱概念继续拆到例子和反例层面。".to_string(),
    });

    Interpretation {
        paper_id,
        blocks,
        concepts,
        summary: Some(summary),
    }
}

fn collect_concepts(notes: &[SliceNotes]) -> Vec<Concept> {
    let mut raw_concepts = Vec::new();
    let mut seen = HashMap::new();

    for note in notes {
        for concept in &note.concepts {
            let term = concept.term.trim();
            if term.is_empty() {
                continue;
            }
            let key = normalize_key(term);
            if seen.contains_key(&key) {
                continue;
            }
            seen.insert(key, raw_concepts.len());
            raw_concepts.push(concept.clone());
        }
    }

    let mut term_to_id = HashMap::new();
    let mut concepts = Vec::new();
    for (idx, concept) in raw_concepts.into_iter().take(10).enumerate() {
        let id = format!("concept-{}", idx + 1);
        term_to_id.insert(normalize_key(&concept.term), id.clone());
        concepts.push(Concept {
            id,
            term: truncate_chars(concept.term.trim(), 60),
            definition: truncate_chars(concept.definition.trim(), 110),
            difficulty: sanitize_difficulty(&concept.difficulty),
            related: Vec::new(),
            source_blocks: Vec::new(),
        });
    }

    for idx in 0..concepts.len() {
        let mut related = Vec::new();
        if let Some(raw_index) = seen.get(&normalize_key(&concepts[idx].term)) {
            let note_related = notes
                .iter()
                .flat_map(|note| &note.concepts)
                .find(|concept| normalize_key(&concept.term) == normalize_key(&concepts[idx].term))
                .map(|concept| concept.related.clone())
                .unwrap_or_default();
            let _ = raw_index;
            for term in note_related {
                if let Some(id) = term_to_id.get(&normalize_key(&term))
                    && id != &concepts[idx].id
                    && !related.contains(id)
                {
                    related.push(id.clone());
                }
            }
        }
        if related.is_empty() && concepts.len() > 1 {
            related.push(concepts[(idx + 1) % concepts.len()].id.clone());
        }
        concepts[idx].related = related.into_iter().take(4).collect();
    }

    concepts
}

fn collect_stats(notes: &[SliceNotes]) -> Vec<SliceStat> {
    let mut stats = Vec::new();
    let mut seen = HashMap::new();
    for note in notes {
        for stat in &note.stats {
            if stat.value.trim().is_empty() || stat.label.trim().is_empty() {
                continue;
            }
            let key = normalize_key(&format!("{} {}", stat.value, stat.label));
            if seen.insert(key, ()).is_none() {
                stats.push(stat.clone());
            }
        }
    }
    stats.truncate(8);
    stats
}

fn collect_mechanisms(notes: &[SliceNotes]) -> Vec<SliceMechanism> {
    let mut mechanisms = Vec::new();
    let mut seen = HashMap::new();
    for note in notes {
        for mechanism in &note.mechanisms {
            if mechanism.name.trim().is_empty() {
                continue;
            }
            let key = normalize_key(&mechanism.name);
            if seen.insert(key, ()).is_none() {
                mechanisms.push(mechanism.clone());
            }
        }
    }
    mechanisms.truncate(5);
    mechanisms
}

fn collect_comparisons(notes: &[SliceNotes], mechanisms: &[SliceMechanism]) -> Vec<ComparisonRow> {
    let mut rows = Vec::new();
    for note in notes {
        for comparison in &note.comparisons {
            if comparison.dimension.trim().is_empty() {
                continue;
            }
            rows.push(ComparisonRow {
                label: truncate_chars(&comparison.dimension, 32),
                cells: vec![
                    truncate_chars(&comparison.baseline, 80),
                    truncate_chars(&comparison.paper_approach, 80),
                    truncate_chars(&comparison.lesson, 80),
                ],
            });
        }
    }

    if rows.is_empty() {
        for mechanism in mechanisms.iter().take(3) {
            rows.push(ComparisonRow {
                label: truncate_chars(&mechanism.name, 32),
                cells: vec![
                    truncate_chars(&mechanism.input, 80),
                    truncate_chars(&mechanism.process, 80),
                    truncate_chars(&mechanism.why, 80),
                ],
            });
        }
    }

    rows.truncate(4);
    rows
}

fn collect_evidence(notes: &[SliceNotes]) -> Vec<SliceEvidence> {
    let mut evidence = Vec::new();
    for note in notes {
        for item in &note.evidence {
            if !item.quote.trim().is_empty() {
                evidence.push(item.clone());
            }
        }
    }
    evidence.truncate(4);
    evidence
}

fn synthesize_summary(title: &str, notes: &[SliceNotes]) -> String {
    if let Some(summary) = notes
        .iter()
        .map(|note| note.summary.trim())
        .find(|summary| !summary.is_empty())
    {
        return truncate_chars(summary, 120);
    }

    if let Some(idea) = notes
        .iter()
        .flat_map(|note| &note.core_ideas)
        .map(|idea| idea.trim())
        .find(|idea| !idea.is_empty())
    {
        return truncate_chars(idea, 120);
    }

    format!(
        "这篇论文《{}》需要从问题、机制、证据和取舍四层逐步拆解。",
        truncate_chars(title, 80)
    )
}

fn mechanism_paragraph(mechanisms: &[SliceMechanism], notes: &[SliceNotes]) -> String {
    if !mechanisms.is_empty() {
        let names = mechanisms
            .iter()
            .take(3)
            .map(|mechanism| mechanism.name.trim())
            .filter(|name| !name.is_empty())
            .collect::<Vec<_>>()
            .join("、");
        return format!(
            "从 reader agents 的笔记看，论文的核心不是孤立概念，而是一组可以串起来的机制：{}。理解它们时，可以反复追问：输入是什么，系统如何处理，输出改变了什么，为什么这比直觉做法更可靠。",
            names
        );
    }

    let ideas = notes
        .iter()
        .flat_map(|note| &note.core_ideas)
        .map(|idea| idea.trim())
        .filter(|idea| !idea.is_empty())
        .take(3)
        .collect::<Vec<_>>()
        .join("；");
    if ideas.is_empty() {
        "目前可用笔记还不足以还原完整机制，但可以先从论文提出的问题、关键概念和证据开始建立理解。"
            .to_string()
    } else {
        format!(
            "可以先把论文压缩成三个可复述的判断：{}。再回到原文检查每个判断的证据。",
            ideas
        )
    }
}

fn chart_data_from_stats(stats: &[SliceStat]) -> Vec<ChartDataPoint> {
    stats
        .iter()
        .filter_map(|stat| {
            let value = stat.numeric_value.or_else(|| parse_number(&stat.value))?;
            if !value.is_finite() {
                return None;
            }
            Some(ChartDataPoint {
                label: truncate_chars(&stat.label, 18),
                value,
            })
        })
        .take(6)
        .collect()
}

fn build_quiz_block(notes: &[SliceNotes], first_concept: Option<&Concept>) -> Block {
    if let Some(quiz) = notes
        .iter()
        .flat_map(|note| &note.quiz_questions)
        .find(|quiz| !quiz.question.trim().is_empty() && !quiz.correct_answer.trim().is_empty())
    {
        let mut options = vec![QuizOption {
            text: truncate_chars(&quiz.correct_answer, 90),
            correct: true,
        }];
        options.extend(quiz.distractors.iter().take(3).map(|text| QuizOption {
            text: truncate_chars(text, 90),
            correct: false,
        }));
        if options.len() < 2 {
            options.push(QuizOption {
                text: "只记住论文术语，不解释机制".to_string(),
                correct: false,
            });
        }
        return Block::Quiz {
            id: "quiz-1".to_string(),
            question: truncate_chars(&quiz.question, 120),
            options,
            explain: truncate_chars(&quiz.explanation, 180),
        };
    }

    let concept_name = first_concept
        .map(|concept| concept.term.as_str())
        .unwrap_or("核心机制");
    Block::Quiz {
        id: "quiz-1".to_string(),
        question: format!("如果用费曼方法解释 {concept_name}，最关键要讲清哪件事？"),
        options: vec![
            QuizOption {
                text: "它接收什么输入、做了什么处理、为什么让问题更容易解决".to_string(),
                correct: true,
            },
            QuizOption {
                text: "把论文里的英文术语按出现顺序背出来".to_string(),
                correct: false,
            },
            QuizOption {
                text: "只记住实验数字，不解释它们支持了什么判断".to_string(),
                correct: false,
            },
        ],
        explain: "费曼学习法强调能用自己的话讲清机制和因果，而不是复述术语。".to_string(),
    }
}

fn build_mechanism_svg(mechanisms: &[SliceMechanism]) -> String {
    let width = 920;
    let height = 260;
    let count = mechanisms.len().clamp(1, 4);
    let card_width = 190;
    let gap = 30;
    let start_x = ((width - (count * card_width + (count - 1) * gap)) / 2) as i32;

    let mut cards = String::new();
    for (idx, mechanism) in mechanisms.iter().take(count).enumerate() {
        let x = start_x + (idx * (card_width + gap)) as i32;
        let name = escape_xml(&truncate_chars(&mechanism.name, 24));
        let input = escape_xml(&truncate_chars(&mechanism.input, 30));
        let output = escape_xml(&truncate_chars(&mechanism.output, 30));
        cards.push_str(&format!(
            r##"<g>
  <rect x="{x}" y="62" width="{card_width}" height="132" rx="14" fill="#ffffff" stroke="#bfdbfe" stroke-width="2"/>
  <circle cx="{cx}" cy="62" r="18" fill="#2563eb"/>
  <text x="{cx}" y="68" text-anchor="middle" font-size="15" font-weight="700" fill="#ffffff">{step}</text>
  <text x="{tx}" y="100" font-size="15" font-weight="700" fill="#0f172a">{name}</text>
  <text x="{tx}" y="130" font-size="12" fill="#475569">输入：{input}</text>
  <text x="{tx}" y="154" font-size="12" fill="#475569">输出：{output}</text>
</g>"##,
            x = x,
            cx = x + card_width as i32 / 2,
            tx = x + 18,
            step = idx + 1,
            name = name,
            input = input,
            output = output,
        ));

        if idx + 1 < count {
            let arrow_x = x + card_width as i32 + 8;
            cards.push_str(&format!(
                r##"<path d="M {arrow_x} 128 H {end_x}" stroke="#38bdf8" stroke-width="3" stroke-linecap="round"/>
<path d="M {end_x} 128 l -8 -6 v12 z" fill="#38bdf8"/>"##,
                arrow_x = arrow_x,
                end_x = arrow_x + gap as i32 - 12,
            ));
        }
    }

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" role="img" aria-label="论文机制流程图">
  <rect width="{width}" height="{height}" rx="22" fill="#f8fafc"/>
  <text x="40" y="38" font-size="18" font-weight="700" fill="#0f172a">Feynman Path: 输入 -> 处理 -> 输出 -> 取舍</text>
  {cards}
</svg>"##
    )
}

fn sanitize_difficulty(value: &str) -> String {
    let value = value.trim().to_ascii_lowercase();
    if value.contains("basic") {
        "basic".to_string()
    } else if value.contains("advanced") {
        "advanced".to_string()
    } else {
        "intermediate".to_string()
    }
}

fn normalize_key(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_number(value: &str) -> Option<f64> {
    let mut buf = String::new();
    let mut started = false;
    for ch in value.chars() {
        if ch.is_ascii_digit() || (started && ch == '.') || (!started && ch == '-') {
            started = true;
            buf.push(ch);
        } else if started && ch == ',' {
            continue;
        } else if started {
            break;
        }
    }
    if buf.is_empty() || buf == "-" {
        None
    } else {
        buf.parse::<f64>().ok()
    }
}

fn optional_f64_from_any<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    let Some(value) = value else {
        return Ok(None);
    };

    Ok(match value {
        serde_json::Value::Number(number) => number.as_f64(),
        serde_json::Value::String(text) => parse_number(&text),
        _ => None,
    })
}

fn truncate_chars(value: &str, max: usize) -> String {
    let trimmed = value.trim();
    let mut result = String::new();
    for (idx, ch) in trimmed.chars().enumerate() {
        if idx >= max {
            result.push('…');
            return result;
        }
        result.push(ch);
    }
    result
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
