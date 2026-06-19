use std::collections::HashMap;
use std::future::{self, Future};

use serde::{Deserialize, Deserializer, Serialize};
use tokio::task::JoinSet;
use uuid::Uuid;

use crate::domain::interpretation::{
    Block, ChartDataPoint, ComparisonRow, Concept, Interpretation, QuizOption, Stat,
};
use crate::error::{AppError, AppResult};
use crate::llm::{LlmClient, LlmRole};
use crate::prompt;

const SLICE_CHARS: usize = 6_500;
const MAX_PARALLEL_READERS: usize = 4;
const SPECIALIST_AGENT_COUNT: usize = 3;
const AGENT_NOTE_CHARS: usize = 900;

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
        self.interpret_with_progress(paper_id, title, text, |_| future::ready(()))
            .await
    }

    /// 多 Agent 解读：先把论文拆给多个短上下文 reader 并发分析，再把 reader
    /// artifacts 封装成 A2A-style task envelope 交给 specialist agents 做交叉审稿，
    /// 最后由本地 reducer 稳定组装成前端 Block。
    pub async fn interpret_with_progress<F, Fut>(
        &self,
        paper_id: Uuid,
        title: &str,
        text: &str,
        mut on_agent_event: F,
    ) -> AppResult<Interpretation>
    where
        F: FnMut(AgentProgressEvent) -> Fut,
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
                    .call_json_with_role(LlmRole::Reader, prompt::SYSTEM_ANALYZE_SLICE, &user_msg)
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
                    let percent = 35
                        + ((completed as f32 / total.max(1) as f32) * 30.0)
                            .round()
                            .clamp(0.0, 30.0) as u8;
                    on_agent_event(AgentProgressEvent::new(
                        "reading",
                        "Reader Agent",
                        &format!(
                            "已完成 {completed}/{total} 个 reader agent：{}。正在继续合并其余片段的理解。",
                            output.label
                        ),
                        percent,
                    ))
                    .await;
                    outputs.push(output);
                }
                Ok(Err(err)) => {
                    tracing::warn!(paper_id = %paper_id, "reader agent 失败: {err}");
                    on_agent_event(AgentProgressEvent::new(
                        "reading",
                        "Reader Agent",
                        "一个 reader agent 返回失败，继续汇总其余结果。",
                        35,
                    ))
                    .await;
                    failures.push(err.to_string());
                }
                Err(err) => {
                    tracing::warn!(paper_id = %paper_id, "reader agent 任务中断: {err}");
                    on_agent_event(AgentProgressEvent::new(
                        "reading",
                        "Reader Agent",
                        "一个 reader agent 任务中断，继续汇总其余结果。",
                        35,
                    ))
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

        on_agent_event(AgentProgressEvent::new(
            "synthesizing",
            "A2A 协同审稿",
            "reader notes 已封装为 A2A-style task，正在分发给方法、证据和教学 specialist agents。",
            68,
        ))
        .await;
        let review = self
            .run_specialist_agents(paper_id, title, &outputs, &mut on_agent_event)
            .await;

        Ok(build_interpretation_from_notes(
            paper_id, title, outputs, review,
        ))
    }

    async fn run_specialist_agents<F, Fut>(
        &self,
        paper_id: Uuid,
        title: &str,
        outputs: &[ReaderOutput],
        on_agent_event: &mut F,
    ) -> DeepReviewArtifacts
    where
        F: FnMut(AgentProgressEvent) -> Fut,
        Fut: Future<Output = ()>,
    {
        let mut specialists = JoinSet::new();
        for agent in SpecialistAgent::all() {
            let llm = self.llm.clone();
            let paper_title = title.to_string();
            let envelope = build_a2a_task_envelope(paper_id, title, agent, outputs);
            specialists.spawn(async move {
                let user_msg =
                    prompt::user_a2a_agent_task(&paper_title, agent.display_name(), &envelope);
                let value = llm
                    .call_json_with_role(LlmRole::Specialist, agent.system_prompt(), &user_msg)
                    .await?;
                Ok::<(SpecialistAgent, serde_json::Value), AppError>((agent, value))
            });
        }

        let mut completed = 0;
        let mut artifacts = DeepReviewArtifacts::default();
        while let Some(joined) = specialists.join_next().await {
            completed += 1;
            let percent = 68
                + ((completed as f32 / SPECIALIST_AGENT_COUNT as f32) * 12.0)
                    .round()
                    .clamp(0.0, 12.0) as u8;

            match joined {
                Ok(Ok((agent, value))) => match artifacts.insert(agent, value) {
                    Ok(()) => {
                        tracing::info!(
                            paper_id = %paper_id,
                            agent = agent.slug(),
                            "specialist agent 完成"
                        );
                        on_agent_event(AgentProgressEvent::new(
                            "synthesizing",
                            agent.stage_label(),
                            &format!(
                                "{} 已返回 artifact，正在等待其他 specialist agents。",
                                agent.display_name()
                            ),
                            percent,
                        ))
                        .await;
                    }
                    Err(err) => {
                        tracing::warn!(
                            paper_id = %paper_id,
                            agent = agent.slug(),
                            "specialist agent artifact 解析失败: {err}"
                        );
                        artifacts
                            .failed_agents
                            .push(format!("{}: {err}", agent.display_name()));
                        on_agent_event(AgentProgressEvent::new(
                            "synthesizing",
                            agent.stage_label(),
                            &format!(
                                "{} 返回的 artifact 无法解析，继续使用其余 agent 的结果。",
                                agent.display_name()
                            ),
                            percent,
                        ))
                        .await;
                    }
                },
                Ok(Err(err)) => {
                    tracing::warn!(paper_id = %paper_id, "specialist agent 失败: {err}");
                    artifacts.failed_agents.push(err.to_string());
                    on_agent_event(AgentProgressEvent::new(
                        "synthesizing",
                        "Specialist Agent",
                        "一个 specialist agent 调用失败，继续使用其余 agent 的结果。",
                        percent,
                    ))
                    .await;
                }
                Err(err) => {
                    tracing::warn!(paper_id = %paper_id, "specialist agent 任务中断: {err}");
                    artifacts.failed_agents.push(err.to_string());
                    on_agent_event(AgentProgressEvent::new(
                        "synthesizing",
                        "Specialist Agent",
                        "一个 specialist agent 任务中断，继续使用其余 agent 的结果。",
                        percent,
                    ))
                    .await;
                }
            }
        }

        artifacts
    }
}

#[derive(Debug, Clone)]
pub struct AgentProgressEvent {
    pub phase: String,
    pub stage: String,
    pub message: String,
    pub percent: u8,
}

impl AgentProgressEvent {
    fn new(phase: &str, stage: &str, message: &str, percent: u8) -> Self {
        Self {
            phase: phase.to_string(),
            stage: stage.to_string(),
            message: message.to_string(),
            percent,
        }
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

#[derive(Debug, Clone, Copy)]
enum SpecialistAgent {
    Method,
    Evidence,
    Teaching,
}

impl SpecialistAgent {
    fn all() -> [Self; SPECIALIST_AGENT_COUNT] {
        [Self::Method, Self::Evidence, Self::Teaching]
    }

    fn slug(self) -> &'static str {
        match self {
            Self::Method => "method-mechanism",
            Self::Evidence => "evidence-audit",
            Self::Teaching => "teaching-synthesis",
        }
    }

    fn display_name(self) -> &'static str {
        match self {
            Self::Method => "Method & Mechanism Agent",
            Self::Evidence => "Evidence Audit Agent",
            Self::Teaching => "Teaching Synthesis Agent",
        }
    }

    fn stage_label(self) -> &'static str {
        match self {
            Self::Method => "机制审稿",
            Self::Evidence => "证据审计",
            Self::Teaching => "教学综合",
        }
    }

    fn system_prompt(self) -> &'static str {
        match self {
            Self::Method => prompt::SYSTEM_A2A_METHOD_AGENT,
            Self::Evidence => prompt::SYSTEM_A2A_EVIDENCE_AGENT,
            Self::Teaching => prompt::SYSTEM_A2A_TEACHING_AGENT,
        }
    }
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

// ── specialist agents 的深度审稿 artifact ──────────────

#[derive(Debug, Clone, Default)]
struct DeepReviewArtifacts {
    method: Option<MethodReview>,
    evidence: Option<EvidenceReview>,
    teaching: Option<TeachingReview>,
    failed_agents: Vec<String>,
}

impl DeepReviewArtifacts {
    fn insert(&mut self, agent: SpecialistAgent, value: serde_json::Value) -> AppResult<()> {
        match agent {
            SpecialistAgent::Method => {
                self.method = Some(serde_json::from_value(value).map_err(|e| {
                    AppError::InvalidLlmOutput(format!("method agent artifact 解析失败: {e}"))
                })?);
            }
            SpecialistAgent::Evidence => {
                self.evidence = Some(serde_json::from_value(value).map_err(|e| {
                    AppError::InvalidLlmOutput(format!("evidence agent artifact 解析失败: {e}"))
                })?);
            }
            SpecialistAgent::Teaching => {
                self.teaching = Some(serde_json::from_value(value).map_err(|e| {
                    AppError::InvalidLlmOutput(format!("teaching agent artifact 解析失败: {e}"))
                })?);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct MethodReview {
    #[serde(default)]
    problem_statement: String,
    #[serde(default)]
    why_hard: String,
    #[serde(default)]
    prior_gap: String,
    #[serde(default)]
    contribution_thesis: String,
    #[serde(default)]
    mechanism_chain: Vec<ReviewMechanismStep>,
    #[serde(default)]
    assumptions: Vec<String>,
    #[serde(default)]
    limitations: Vec<ReviewLimitation>,
    #[serde(default)]
    open_questions: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct ReviewMechanismStep {
    #[serde(default)]
    title: String,
    #[serde(default)]
    input: String,
    #[serde(default)]
    process: String,
    #[serde(default)]
    output: String,
    #[serde(default)]
    why_it_matters: String,
    #[serde(default)]
    evidence_anchor: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct ReviewLimitation {
    #[serde(default)]
    point: String,
    #[serde(default)]
    why_it_matters: String,
    #[serde(default)]
    how_to_check: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct EvidenceReview {
    #[serde(default)]
    evidence_map: Vec<EvidenceAuditItem>,
    #[serde(default)]
    metric_insights: Vec<MetricInsight>,
    #[serde(default)]
    weak_claims: Vec<WeakClaim>,
    #[serde(default)]
    counterfactual_checks: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct EvidenceAuditItem {
    #[serde(default)]
    claim: String,
    #[serde(default)]
    support: String,
    #[serde(default)]
    quote: String,
    #[serde(default)]
    cite: Option<String>,
    #[serde(default)]
    confidence: String,
    #[serde(default)]
    caveat: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct MetricInsight {
    #[serde(default)]
    metric: String,
    #[serde(default)]
    interpretation: String,
    #[serde(default)]
    risk: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct WeakClaim {
    #[serde(default)]
    claim: String,
    #[serde(default)]
    missing_evidence: String,
    #[serde(default)]
    suggested_check: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct TeachingReview {
    #[serde(default)]
    reader_model: String,
    #[serde(default)]
    learning_path: Vec<LearningStep>,
    #[serde(default)]
    analogies: Vec<Analogy>,
    #[serde(default)]
    feynman_questions: Vec<FeynmanQuestion>,
    #[serde(default)]
    final_takeaway: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct LearningStep {
    #[serde(default)]
    question: String,
    #[serde(default)]
    answer: String,
    #[serde(default)]
    why_it_matters: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct Analogy {
    #[serde(default)]
    concept: String,
    #[serde(default)]
    analogy: String,
    #[serde(default)]
    boundary: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct FeynmanQuestion {
    #[serde(default)]
    question: String,
    #[serde(default)]
    ideal_answer: String,
    #[serde(default)]
    common_wrong_answer: String,
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
    review: DeepReviewArtifacts,
) -> Interpretation {
    let notes: Vec<SliceNotes> = outputs.into_iter().map(|output| output.notes).collect();
    let concepts = collect_concepts(&notes);
    let stats = collect_stats(&notes);
    let mechanisms = collect_mechanisms(&notes);
    let comparisons = collect_comparisons(&notes, &mechanisms);
    let evidence = collect_evidence(&notes);
    let summary = synthesize_summary(title, &notes, &review);

    let mut blocks = Vec::new();
    blocks.push(Block::Section {
        id: "sec-1".to_string(),
        num: "01".to_string(),
        title: "先抓住问题、缺口和贡献主张".to_string(),
    });
    blocks.push(Block::Paragraph {
        id: "p-1".to_string(),
        text: opening_paragraph(title, &summary, &review),
    });

    if let Some(rows) = problem_frame_rows(&review) {
        blocks.push(Block::Comparison {
            id: "cmp-problem-frame".to_string(),
            columns: vec![
                "层次".to_string(),
                "当前解读".to_string(),
                "阅读时要验证什么".to_string(),
            ],
            rows,
        });
    }

    if let Some(item) = best_quote(&review, &evidence) {
        blocks.push(Block::Quote {
            id: "quote-1".to_string(),
            text: truncate_chars(&item.quote, 280),
            cite: item.cite.filter(|cite| !cite.trim().is_empty()),
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
        title: "机制链路：从输入到可验证输出".to_string(),
    });
    blocks.push(Block::Paragraph {
        id: "p-2".to_string(),
        text: mechanism_paragraph(&mechanisms, &notes, &review),
    });
    if let Some(method) = &review.method
        && !method.mechanism_chain.is_empty()
    {
        blocks.push(Block::Diagram {
            id: "diagram-1".to_string(),
            svg: build_review_mechanism_svg(&method.mechanism_chain),
            caption: Some("specialist agent 汇总出的论文机制链路".to_string()),
        });
        blocks.push(Block::Comparison {
            id: "cmp-mechanism-chain".to_string(),
            columns: vec![
                "步骤".to_string(),
                "输入".to_string(),
                "处理".to_string(),
                "输出".to_string(),
                "作用".to_string(),
            ],
            rows: method
                .mechanism_chain
                .iter()
                .filter(|step| !step.title.trim().is_empty())
                .take(5)
                .map(|step| ComparisonRow {
                    label: truncate_chars(&step.title, 32),
                    cells: vec![
                        truncate_chars(&step.input, 80),
                        truncate_chars(&step.process, 90),
                        truncate_chars(&step.output, 80),
                        truncate_chars(&step.why_it_matters, 90),
                    ],
                })
                .collect(),
        });
    } else if !mechanisms.is_empty() {
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
        title: "证据审计：哪些结论真的被支持".to_string(),
    });
    blocks.push(Block::Paragraph {
        id: "p-evidence".to_string(),
        text: evidence_audit_paragraph(&review, &evidence),
    });
    if let Some(evidence_rows) = evidence_audit_rows(&review) {
        blocks.push(Block::Comparison {
            id: "cmp-evidence-audit".to_string(),
            columns: vec![
                "主张".to_string(),
                "支持证据".to_string(),
                "可信度".to_string(),
                "边界/小心点".to_string(),
            ],
            rows: evidence_rows,
        });
    }
    if let Some(metric_rows) = metric_insight_rows(&review, &stats) {
        blocks.push(Block::Comparison {
            id: "cmp-metrics".to_string(),
            columns: vec![
                "指标".to_string(),
                "它真正说明什么".to_string(),
                "常见误读风险".to_string(),
            ],
            rows: metric_rows,
        });
    }
    if let Some(boundary_rows) = boundary_rows(&review) {
        blocks.push(Block::Comparison {
            id: "cmp-boundaries".to_string(),
            columns: vec![
                "边界/弱证据".to_string(),
                "为什么重要".to_string(),
                "如何回论文检查".to_string(),
            ],
            rows: boundary_rows,
        });
    }

    blocks.push(Block::Section {
        id: "sec-4".to_string(),
        num: "04".to_string(),
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
        id: "sec-5".to_string(),
        num: "05".to_string(),
        title: "用学习路径和反问检验理解深度".to_string(),
    });
    if let Some(teaching_rows) = learning_path_rows(&review) {
        blocks.push(Block::Comparison {
            id: "cmp-learning-path".to_string(),
            columns: vec![
                "追问".to_string(),
                "基于论文的回答".to_string(),
                "为什么能加深理解".to_string(),
            ],
            rows: teaching_rows,
        });
    }
    if let Some(analogy_rows) = analogy_rows(&review) {
        blocks.push(Block::Comparison {
            id: "cmp-analogies".to_string(),
            columns: vec![
                "概念/机制".to_string(),
                "类比".to_string(),
                "类比边界".to_string(),
            ],
            rows: analogy_rows,
        });
    }
    blocks.push(build_quiz_block(&notes, concepts.first(), &review));
    blocks.push(Block::Paragraph {
        id: "p-4".to_string(),
        text: closing_paragraph(&review),
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

fn synthesize_summary(title: &str, notes: &[SliceNotes], review: &DeepReviewArtifacts) -> String {
    if let Some(method) = &review.method {
        let thesis = method.contribution_thesis.trim();
        if !thesis.is_empty() {
            return truncate_chars(thesis, 140);
        }
        let problem = method.problem_statement.trim();
        if !problem.is_empty() {
            return truncate_chars(problem, 140);
        }
    }

    if let Some(teaching) = &review.teaching {
        let takeaway = teaching.final_takeaway.trim();
        if !takeaway.is_empty() {
            return truncate_chars(takeaway, 140);
        }
    }

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

fn opening_paragraph(title: &str, summary: &str, review: &DeepReviewArtifacts) -> String {
    if let Some(method) = &review.method {
        let mut parts = Vec::new();
        if !method.problem_statement.trim().is_empty() {
            parts.push(format!(
                "问题：{}",
                truncate_chars(&method.problem_statement, 160)
            ));
        }
        if !method.prior_gap.trim().is_empty() {
            parts.push(format!("缺口：{}", truncate_chars(&method.prior_gap, 140)));
        }
        if !method.why_hard.trim().is_empty() {
            parts.push(format!("难点：{}", truncate_chars(&method.why_hard, 140)));
        }
        if !parts.is_empty() {
            return format!(
                "这次解读先不把《{}》压扁成摘要，而是用多 agent 审稿把它拆成问题、缺口、机制和证据四层。{} 最终要验证的主张是：{}",
                truncate_chars(title, 80),
                parts.join(" "),
                summary
            );
        }
    }

    format!(
        "用费曼式自学的第一步看这篇论文：先不要急着背术语，而是把它压缩成一句可转述、可质疑、可回到证据检查的话。{}",
        summary
    )
}

fn problem_frame_rows(review: &DeepReviewArtifacts) -> Option<Vec<ComparisonRow>> {
    let method = review.method.as_ref()?;
    let mut rows = Vec::new();
    push_row_if_present(
        &mut rows,
        "问题",
        &method.problem_statement,
        "它是否具体到任务、对象和约束，而不是泛泛说“提升效果”。",
    );
    push_row_if_present(
        &mut rows,
        "为什么难",
        &method.why_hard,
        "旧方法为什么会失败，失败是成本、准确性、扩展性还是假设问题。",
    );
    push_row_if_present(
        &mut rows,
        "既有缺口",
        &method.prior_gap,
        "作者是否证明了这个 gap 真实存在，而不是为了引出方法而设定。",
    );
    push_row_if_present(
        &mut rows,
        "贡献主张",
        &method.contribution_thesis,
        "后面的机制和实验是否真的支持这句主张。",
    );

    if rows.is_empty() { None } else { Some(rows) }
}

fn push_row_if_present(rows: &mut Vec<ComparisonRow>, label: &str, value: &str, check: &str) {
    if value.trim().is_empty() {
        return;
    }
    rows.push(ComparisonRow {
        label: label.to_string(),
        cells: vec![truncate_chars(value, 120), check.to_string()],
    });
}

#[derive(Debug, Clone)]
struct QuoteCandidate {
    quote: String,
    cite: Option<String>,
}

fn best_quote(review: &DeepReviewArtifacts, evidence: &[SliceEvidence]) -> Option<QuoteCandidate> {
    if let Some(item) = review.evidence.as_ref().and_then(|audit| {
        audit
            .evidence_map
            .iter()
            .find(|item| !item.quote.trim().is_empty())
    }) {
        return Some(QuoteCandidate {
            quote: item.quote.clone(),
            cite: item.cite.clone(),
        });
    }

    evidence.first().map(|item| QuoteCandidate {
        quote: item.quote.clone(),
        cite: item.cite.clone(),
    })
}

fn mechanism_paragraph(
    mechanisms: &[SliceMechanism],
    notes: &[SliceNotes],
    review: &DeepReviewArtifacts,
) -> String {
    if let Some(method) = &review.method
        && !method.mechanism_chain.is_empty()
    {
        let names = method
            .mechanism_chain
            .iter()
            .take(4)
            .map(|step| step.title.trim())
            .filter(|title| !title.is_empty())
            .collect::<Vec<_>>()
            .join(" -> ");
        if !names.is_empty() {
            return format!(
                "Method agent 把论文机制拆成一条可追问的链路：{}。读的时候不要只看模块名，而要逐步检查每一步的输入、处理、输出，以及它是否真的支撑贡献主张。",
                names
            );
        }
    }

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

fn evidence_audit_paragraph(
    review: &DeepReviewArtifacts,
    fallback_evidence: &[SliceEvidence],
) -> String {
    if let Some(evidence) = &review.evidence {
        let supported = evidence
            .evidence_map
            .iter()
            .filter(|item| !item.claim.trim().is_empty())
            .count();
        let weak = evidence
            .weak_claims
            .iter()
            .filter(|item| !item.claim.trim().is_empty())
            .count();
        if supported > 0 || weak > 0 {
            return format!(
                "Evidence agent 没有把论文当成一串结论来复述，而是把主张和证据逐一配对：目前整理出 {supported} 条可追溯主张，以及 {weak} 条需要回原文继续确认的弱证据点。"
            );
        }
    }

    if fallback_evidence.is_empty() {
        "当前 reader notes 中可追溯证据不足，所以这部分先提醒读者：把任何听起来顺滑的解释都当成待验证假设，回到原文找实验、定义、消融或理论说明。"
            .to_string()
    } else {
        format!(
            "reader agents 已提取 {} 条原文证据。下面的关键是把每条引用和它支持的主张配对，而不是把引用当作装饰。",
            fallback_evidence.len()
        )
    }
}

fn evidence_audit_rows(review: &DeepReviewArtifacts) -> Option<Vec<ComparisonRow>> {
    let evidence = review.evidence.as_ref()?;
    let rows = evidence
        .evidence_map
        .iter()
        .filter(|item| !item.claim.trim().is_empty())
        .take(6)
        .map(|item| ComparisonRow {
            label: truncate_chars(&item.claim, 42),
            cells: vec![
                truncate_chars(&item.support, 100),
                sanitize_confidence(&item.confidence),
                truncate_chars(&item.caveat, 100),
            ],
        })
        .collect::<Vec<_>>();

    if rows.is_empty() { None } else { Some(rows) }
}

fn metric_insight_rows(
    review: &DeepReviewArtifacts,
    fallback_stats: &[SliceStat],
) -> Option<Vec<ComparisonRow>> {
    if let Some(evidence) = &review.evidence {
        let rows = evidence
            .metric_insights
            .iter()
            .filter(|item| !item.metric.trim().is_empty())
            .take(4)
            .map(|item| ComparisonRow {
                label: truncate_chars(&item.metric, 36),
                cells: vec![
                    truncate_chars(&item.interpretation, 100),
                    truncate_chars(&item.risk, 100),
                ],
            })
            .collect::<Vec<_>>();
        if !rows.is_empty() {
            return Some(rows);
        }
    }

    let rows = fallback_stats
        .iter()
        .filter(|stat| !stat.value.trim().is_empty() && !stat.label.trim().is_empty())
        .take(4)
        .map(|stat| ComparisonRow {
            label: truncate_chars(&stat.value, 32),
            cells: vec![
                truncate_chars(&stat.label, 100),
                "只看数字大小容易误读；需要检查指标定义、基线、数据集和实验条件。".to_string(),
            ],
        })
        .collect::<Vec<_>>();

    if rows.is_empty() { None } else { Some(rows) }
}

fn boundary_rows(review: &DeepReviewArtifacts) -> Option<Vec<ComparisonRow>> {
    let mut rows = Vec::new();
    if let Some(method) = &review.method {
        rows.extend(
            method
                .limitations
                .iter()
                .filter(|item| !item.point.trim().is_empty())
                .take(3)
                .map(|item| ComparisonRow {
                    label: truncate_chars(&item.point, 40),
                    cells: vec![
                        truncate_chars(&item.why_it_matters, 100),
                        truncate_chars(&item.how_to_check, 100),
                    ],
                }),
        );
    }

    if let Some(evidence) = &review.evidence {
        rows.extend(
            evidence
                .weak_claims
                .iter()
                .filter(|item| !item.claim.trim().is_empty())
                .take(3)
                .map(|item| ComparisonRow {
                    label: truncate_chars(&item.claim, 40),
                    cells: vec![
                        truncate_chars(&item.missing_evidence, 100),
                        truncate_chars(&item.suggested_check, 100),
                    ],
                }),
        );
    }

    rows.truncate(5);
    if rows.is_empty() { None } else { Some(rows) }
}

fn learning_path_rows(review: &DeepReviewArtifacts) -> Option<Vec<ComparisonRow>> {
    let teaching = review.teaching.as_ref()?;
    let rows = teaching
        .learning_path
        .iter()
        .filter(|step| !step.question.trim().is_empty())
        .take(5)
        .map(|step| ComparisonRow {
            label: truncate_chars(&step.question, 48),
            cells: vec![
                truncate_chars(&step.answer, 120),
                truncate_chars(&step.why_it_matters, 100),
            ],
        })
        .collect::<Vec<_>>();

    if rows.is_empty() { None } else { Some(rows) }
}

fn analogy_rows(review: &DeepReviewArtifacts) -> Option<Vec<ComparisonRow>> {
    let teaching = review.teaching.as_ref()?;
    let rows = teaching
        .analogies
        .iter()
        .filter(|item| !item.concept.trim().is_empty() || !item.analogy.trim().is_empty())
        .take(3)
        .map(|item| ComparisonRow {
            label: truncate_chars(&item.concept, 36),
            cells: vec![
                truncate_chars(&item.analogy, 110),
                truncate_chars(&item.boundary, 100),
            ],
        })
        .collect::<Vec<_>>();

    if rows.is_empty() { None } else { Some(rows) }
}

fn closing_paragraph(review: &DeepReviewArtifacts) -> String {
    if let Some(teaching) = &review.teaching {
        let takeaway = teaching.final_takeaway.trim();
        if !takeaway.is_empty() {
            return format!(
                "最后检验自己是否真正读懂：能否不用论文原句，把问题、机制、证据和边界串成一个闭环。Teaching agent 给出的最终 takeaway 是：{}",
                truncate_chars(takeaway, 160)
            );
        }
    }

    "如果你能不用论文原句、只用自己的话解释上面的机制、证据和边界，就已经进入费曼学习法最关键的一步。接下来可以点开概念实验室，把薄弱概念继续拆到例子和反例层面。".to_string()
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

fn build_quiz_block(
    notes: &[SliceNotes],
    first_concept: Option<&Concept>,
    review: &DeepReviewArtifacts,
) -> Block {
    if let Some(question) = review.teaching.as_ref().and_then(|teaching| {
        teaching.feynman_questions.iter().find(|question| {
            !question.question.trim().is_empty()
                && !question.ideal_answer.trim().is_empty()
                && !question.common_wrong_answer.trim().is_empty()
        })
    }) {
        return Block::Quiz {
            id: "quiz-1".to_string(),
            question: truncate_chars(&question.question, 140),
            options: vec![
                QuizOption {
                    text: truncate_chars(&question.ideal_answer, 120),
                    correct: true,
                },
                QuizOption {
                    text: truncate_chars(&question.common_wrong_answer, 120),
                    correct: false,
                },
                QuizOption {
                    text: "只复述术语定义，不说明证据和边界。".to_string(),
                    correct: false,
                },
            ],
            explain: truncate_chars(&question.explanation, 220),
        };
    }

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

fn build_review_mechanism_svg(steps: &[ReviewMechanismStep]) -> String {
    let width = 1040;
    let height = 330;
    let count = steps.len().clamp(1, 5);
    let card_width = 176;
    let gap = 22;
    let start_x = ((width - (count * card_width + (count - 1) * gap)) / 2) as i32;

    let mut cards = String::new();
    for (idx, step) in steps.iter().take(count).enumerate() {
        let x = start_x + (idx * (card_width + gap)) as i32;
        let title = escape_xml(&truncate_chars(&step.title, 22));
        let input = escape_xml(&truncate_chars(&step.input, 28));
        let process = escape_xml(&truncate_chars(&step.process, 34));
        let output = escape_xml(&truncate_chars(&step.output, 28));
        cards.push_str(&format!(
            r##"<g>
  <rect x="{x}" y="74" width="{card_width}" height="176" rx="16" fill="#ffffff" stroke="#a7f3d0" stroke-width="2"/>
  <circle cx="{cx}" cy="74" r="18" fill="#059669"/>
  <text x="{cx}" y="80" text-anchor="middle" font-size="15" font-weight="700" fill="#ffffff">{step_no}</text>
  <text x="{tx}" y="112" font-size="14" font-weight="700" fill="#0f172a">{title}</text>
  <text x="{tx}" y="142" font-size="11" fill="#475569">输入: {input}</text>
  <text x="{tx}" y="166" font-size="11" fill="#475569">处理: {process}</text>
  <text x="{tx}" y="204" font-size="11" fill="#475569">输出: {output}</text>
</g>"##,
            x = x,
            cx = x + card_width as i32 / 2,
            tx = x + 16,
            step_no = idx + 1,
            title = title,
            input = input,
            process = process,
            output = output,
        ));

        if idx + 1 < count {
            let arrow_x = x + card_width as i32 + 6;
            cards.push_str(&format!(
                r##"<path d="M {arrow_x} 160 H {end_x}" stroke="#14b8a6" stroke-width="3" stroke-linecap="round"/>
<path d="M {end_x} 160 l -8 -6 v12 z" fill="#14b8a6"/>"##,
                arrow_x = arrow_x,
                end_x = arrow_x + gap as i32 - 10,
            ));
        }
    }

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" role="img" aria-label="多 agent 汇总的论文机制链路">
  <rect width="{width}" height="{height}" rx="24" fill="#f8fafc"/>
  <text x="44" y="42" font-size="19" font-weight="700" fill="#0f172a">A2A Specialist Review: mechanism chain</text>
  <text x="44" y="276" font-size="13" fill="#475569">读法：每个节点都要能回答“输入是什么、处理如何发生、输出为什么支持论文主张”。</text>
  {cards}
</svg>"##
    )
}

fn build_a2a_task_envelope(
    paper_id: Uuid,
    title: &str,
    agent: SpecialistAgent,
    outputs: &[ReaderOutput],
) -> String {
    let reader_artifacts = outputs
        .iter()
        .map(|output| {
            serde_json::json!({
                "artifactId": format!("reader-note-{}", output.index),
                "name": format!("Reader Agent {} Notes", output.index),
                "description": output.label,
                "parts": [
                    {
                        "kind": "data",
                        "data": compact_slice_notes(&output.notes)
                    }
                ]
            })
        })
        .collect::<Vec<_>>();

    let envelope = serde_json::json!({
        "protocol": "a2a-inspired-internal",
        "protocol_note": "Local task/message/artifact envelope shaped after Agent2Agent concepts; transport can later be replaced by remote A2A.",
        "task": {
            "id": format!("paper-{paper_id}-{}", agent.slug()),
            "kind": "task",
            "skill": agent.slug(),
            "status": "submitted",
            "metadata": {
                "paper_id": paper_id,
                "paper_title": title,
                "target_agent": agent.display_name(),
                "source_agents": ["reader-agent"],
                "expected_artifact": format!("{} JSON", agent.display_name())
            }
        },
        "message": {
            "role": "user",
            "parts": [
                {
                    "kind": "text",
                    "text": "请基于 reader artifacts 做跨片段综合审稿。不要重写整页内容，只返回你的结构化 JSON artifact。"
                }
            ]
        },
        "artifacts": reader_artifacts
    });

    serde_json::to_string(&envelope).unwrap_or_else(|_| "{}".to_string())
}

fn compact_slice_notes(notes: &SliceNotes) -> serde_json::Value {
    serde_json::json!({
        "slice_focus": truncate_chars(&notes.slice_focus, 90),
        "summary": truncate_chars(&notes.summary, 160),
        "core_ideas": compact_strings(&notes.core_ideas, 5, 160),
        "mechanisms": notes.mechanisms.iter().take(5).map(|item| serde_json::json!({
            "name": truncate_chars(&item.name, 80),
            "input": truncate_chars(&item.input, AGENT_NOTE_CHARS / 6),
            "process": truncate_chars(&item.process, AGENT_NOTE_CHARS / 4),
            "output": truncate_chars(&item.output, AGENT_NOTE_CHARS / 6),
            "why": truncate_chars(&item.why, AGENT_NOTE_CHARS / 5)
        })).collect::<Vec<_>>(),
        "concepts": notes.concepts.iter().take(8).map(|item| serde_json::json!({
            "term": truncate_chars(&item.term, 70),
            "definition": truncate_chars(&item.definition, 140),
            "difficulty": item.difficulty,
            "related": compact_strings(&item.related, 4, 60)
        })).collect::<Vec<_>>(),
        "evidence": notes.evidence.iter().take(6).map(|item| serde_json::json!({
            "claim": truncate_chars(&item.claim, 140),
            "quote": truncate_chars(&item.quote, 180),
            "cite": item.cite
        })).collect::<Vec<_>>(),
        "stats": notes.stats.iter().take(6).map(|item| serde_json::json!({
            "value": truncate_chars(&item.value, 48),
            "label": truncate_chars(&item.label, 120),
            "numeric_value": item.numeric_value
        })).collect::<Vec<_>>(),
        "comparisons": notes.comparisons.iter().take(5).map(|item| serde_json::json!({
            "dimension": truncate_chars(&item.dimension, 80),
            "baseline": truncate_chars(&item.baseline, 140),
            "paper_approach": truncate_chars(&item.paper_approach, 140),
            "lesson": truncate_chars(&item.lesson, 140)
        })).collect::<Vec<_>>()
    })
}

fn compact_strings(values: &[String], limit: usize, max_chars: usize) -> Vec<String> {
    values
        .iter()
        .map(|value| truncate_chars(value, max_chars))
        .filter(|value| !value.trim().is_empty())
        .take(limit)
        .collect()
}

fn sanitize_confidence(value: &str) -> String {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.contains("high") {
        "high".to_string()
    } else if normalized.contains("low") {
        "low".to_string()
    } else {
        "medium".to_string()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_deep_interpretation_from_specialist_artifacts() {
        let paper_id = Uuid::new_v4();
        let outputs = vec![ReaderOutput {
            index: 1,
            label: "摘要/方法".to_string(),
            notes: SliceNotes {
                summary: "论文提出一个更稳的检索增强生成评估框架。".to_string(),
                core_ideas: vec!["把检索、生成和验证拆开评估。".to_string()],
                mechanisms: vec![SliceMechanism {
                    name: "分阶段评估".to_string(),
                    input: "查询和候选证据".to_string(),
                    process: "先筛证据，再约束回答".to_string(),
                    output: "可追溯答案".to_string(),
                    why: "避免把生成质量和检索质量混在一起".to_string(),
                }],
                concepts: vec![SliceConcept {
                    term: "Evidence Grounding".to_string(),
                    definition: "把模型回答绑定到可检查证据的约束。".to_string(),
                    difficulty: "intermediate".to_string(),
                    related: vec![],
                }],
                evidence: vec![SliceEvidence {
                    claim: "框架强调可追溯证据。".to_string(),
                    quote: "answers should be grounded in retrieved evidence".to_string(),
                    cite: Some("Section 3".to_string()),
                }],
                stats: vec![SliceStat {
                    value: "12.5%".to_string(),
                    label: "faithfulness improvement".to_string(),
                    numeric_value: Some(12.5),
                }],
                ..SliceNotes::default()
            },
        }];

        let review = DeepReviewArtifacts {
            method: Some(MethodReview {
                problem_statement: "如何判断 RAG 系统的回答是否真的由证据支持。".to_string(),
                why_hard: "检索错误和生成幻觉会互相掩盖。".to_string(),
                prior_gap: "旧评估常把端到端分数当成唯一质量信号。".to_string(),
                contribution_thesis:
                    "论文把 RAG 评估拆成证据、生成和验证三层，使错误来源更可诊断。".to_string(),
                mechanism_chain: vec![ReviewMechanismStep {
                    title: "检索证据".to_string(),
                    input: "用户查询".to_string(),
                    process: "召回候选段落并过滤".to_string(),
                    output: "证据集合".to_string(),
                    why_it_matters: "后续回答只能基于这些材料检查".to_string(),
                    evidence_anchor: "Section 3".to_string(),
                }],
                limitations: vec![ReviewLimitation {
                    point: "依赖证据标注质量".to_string(),
                    why_it_matters: "低质量标注会误导评估结论。".to_string(),
                    how_to_check: "查看实验数据构建部分。".to_string(),
                }],
                ..MethodReview::default()
            }),
            evidence: Some(EvidenceReview {
                evidence_map: vec![EvidenceAuditItem {
                    claim: "回答必须绑定证据。".to_string(),
                    support: "方法章节要求 evidence grounding。".to_string(),
                    quote: "answers should be grounded in retrieved evidence".to_string(),
                    cite: Some("Section 3".to_string()),
                    confidence: "high".to_string(),
                    caveat: "需要检查证据是否覆盖所有答案片段。".to_string(),
                }],
                metric_insights: vec![MetricInsight {
                    metric: "12.5% faithfulness improvement".to_string(),
                    interpretation: "说明忠实性指标改善。".to_string(),
                    risk: "不能直接等同于用户满意度提升。".to_string(),
                }],
                ..EvidenceReview::default()
            }),
            teaching: Some(TeachingReview {
                learning_path: vec![LearningStep {
                    question: "先问它要解决什么评估问题？".to_string(),
                    answer: "区分检索失败和生成失败。".to_string(),
                    why_it_matters: "这样才能定位系统瓶颈。".to_string(),
                }],
                feynman_questions: vec![FeynmanQuestion {
                    question: "为什么端到端分数不足以说明 RAG 可靠？".to_string(),
                    ideal_answer: "因为同一分数可能来自检索错误、生成幻觉或证据验证失败。"
                        .to_string(),
                    common_wrong_answer: "因为分数越高就一定越可靠。".to_string(),
                    explanation: "理想回答能拆出错误来源，而不是只看总分。".to_string(),
                }],
                final_takeaway: "读懂这篇论文的关键是把质量分数还原成可诊断的证据链。".to_string(),
                ..TeachingReview::default()
            }),
            failed_agents: vec![],
        };

        let interp = build_interpretation_from_notes(paper_id, "RAG Eval", outputs, review);

        assert_eq!(interp.paper_id, paper_id);
        assert!(interp.summary.as_deref().unwrap_or("").contains("三层"));
        assert!(interp.blocks.iter().any(|block| matches!(
            block,
            Block::Comparison { id, .. } if id == "cmp-problem-frame"
        )));
        assert!(interp.blocks.iter().any(|block| matches!(
            block,
            Block::Comparison { id, .. } if id == "cmp-evidence-audit"
        )));
        assert!(interp.blocks.iter().any(|block| matches!(
            block,
            Block::Comparison { id, .. } if id == "cmp-learning-path"
        )));
        assert!(interp.blocks.iter().any(|block| matches!(
            block,
            Block::Quiz { question, .. } if question.contains("端到端分数")
        )));
    }

    #[test]
    fn a2a_task_envelope_carries_reader_artifacts() {
        let paper_id = Uuid::new_v4();
        let outputs = vec![ReaderOutput {
            index: 2,
            label: "实验附近".to_string(),
            notes: SliceNotes {
                summary: "实验报告了关键指标。".to_string(),
                evidence: vec![SliceEvidence {
                    claim: "方法提升了指标。".to_string(),
                    quote: "improves the score".to_string(),
                    cite: Some("Table 2".to_string()),
                }],
                ..SliceNotes::default()
            },
        }];

        let raw =
            build_a2a_task_envelope(paper_id, "Test Paper", SpecialistAgent::Evidence, &outputs);
        let value: serde_json::Value = serde_json::from_str(&raw).expect("valid JSON envelope");

        assert_eq!(value["protocol"], "a2a-inspired-internal");
        assert_eq!(value["task"]["skill"], "evidence-audit");
        assert_eq!(
            value["artifacts"][0]["artifactId"],
            serde_json::Value::String("reader-note-2".to_string())
        );
        assert_eq!(
            value["artifacts"][0]["parts"][0]["data"]["evidence"][0]["cite"],
            serde_json::Value::String("Table 2".to_string())
        );
    }
}
