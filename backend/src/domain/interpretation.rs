use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ════════════════════════════════════════════════════════
//  Block 协议 —— 前后端共享的渲染契约
//
//  后端 LLM 不直接产 HTML，而是产 Block[] JSON，
//  前端 blockRenderer 把每种 Block 映射到对应 React 组件。
// ════════════════════════════════════════════════════════

/// 一个解读块。用 #[serde(tag = "type")] 实现判别联合，
/// 序列化后形如 {"type": "paragraph", "id": "...", "text": "..."}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Block {
    /// 章节标题
    Section {
        id: String,
        num: String,
        title: String,
    },
    /// 普通段落
    Paragraph { id: String, text: String },
    /// 引用块（带 cite 标注论文出处）
    Quote {
        id: String,
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cite: Option<String>,
    },
    /// 关键数据/统计行
    StatRow { id: String, stats: Vec<Stat> },
    /// 概念卡片
    ConceptCard {
        id: String,
        term: String,
        definition: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        icon: Option<String>,
    },
    /// 时间线
    Timeline {
        id: String,
        items: Vec<TimelineItem>,
    },
    /// 对比表
    Comparison {
        id: String,
        columns: Vec<String>,
        rows: Vec<ComparisonRow>,
    },
    /// 交互测验
    Quiz {
        id: String,
        question: String,
        options: Vec<QuizOption>,
        explain: String,
    },
    /// LLM 生成的代码片段（前端沙箱渲染）
    CodeFragment {
        id: String,
        lang: String,
        code: String,
    },
    /// LLM 生成的自定义 HTML 片段（前端 iframe 沙箱渲染）
    CustomHtml { id: String, html: String },
    /// SVG 示意图/插图，带标题
    Figure {
        id: String,
        svg: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        caption: Option<String>,
    },
    /// 数据图表，前端用内联 SVG 渲染
    Chart {
        id: String,
        chart_type: String, // bar | line | pie
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        data: Vec<ChartDataPoint>,
        #[serde(skip_serializing_if = "Option::is_none")]
        x_label: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        y_label: Option<String>,
    },
    /// 流程图 / 架构图（SVG）
    Diagram {
        id: String,
        svg: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        caption: Option<String>,
    },
    /// 机制链路。用结构化数据交给前端排版，避免长文本塞进 SVG 后重叠。
    MechanismChain {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(default)]
        steps: Vec<MechanismChainStep>,
        #[serde(skip_serializing_if = "Option::is_none")]
        note: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stat {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartDataPoint {
    pub label: String,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MechanismChainStep {
    pub title: String,
    pub input: String,
    pub process: String,
    pub output: String,
    pub why_it_matters: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_anchor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineItem {
    pub year: String,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonRow {
    pub label: String,
    pub cells: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizOption {
    pub text: String,
    pub correct: bool,
}

// ════════════════════════════════════════════════════════
//  概念提取 —— 用于 M2 的概念探索
// ════════════════════════════════════════════════════════

/// 一个关键概念
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    pub id: String,
    pub term: String,
    pub definition: String,
    /// 难度：basic / intermediate / advanced
    pub difficulty: String,
    /// 关联概念 id 列表（构成知识图谱）
    pub related: Vec<String>,
    /// 出现在哪些 block 里
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub source_blocks: Vec<String>,
}

// ════════════════════════════════════════════════════════
//  解读结果聚合
// ════════════════════════════════════════════════════════

/// 一篇论文的完整解读
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interpretation {
    pub paper_id: Uuid,
    /// 按"章节"组织的 Block 序列
    pub blocks: Vec<Block>,
    /// 提取的关键概念（M2 启用）
    #[serde(default)]
    pub concepts: Vec<Concept>,
    /// 一句话总结
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

/// 按章节分组的 Block（前端可选这样消费）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub num: String,
    pub title: String,
    pub blocks: Vec<Block>,
}
