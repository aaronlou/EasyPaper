use serde::{Deserialize, Serialize};

use crate::domain::interpretation::Interpretation;
use crate::domain::paper::PaperSummary;

/// POST /api/papers 上传成功后的响应
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub paper: PaperSummary,
}

/// GET /api/papers/:id 返回的完整论文 + 解读
#[derive(Debug, Serialize)]
pub struct PaperDetail {
    pub paper: PaperSummary,
    /// 解读结果（status != completed 时为 None）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interpretation: Option<Interpretation>,
}

/// GET /api/papers/:id/progress 返回的进度信息
///
/// 前端用 phase / stage / percent 渲染步骤条，message 展示当前细节。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    pub phase: String,
    pub stage: String,
    pub message: String,
    pub percent: u8,
    pub updated_at: String,
}

impl ProgressInfo {
    pub fn new(phase: &str, stage: &str, message: &str, percent: u8) -> Self {
        Self {
            phase: phase.to_string(),
            stage: stage.to_string(),
            message: message.to_string(),
            percent,
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// POST /api/papers/:id/concepts/:concept_id/expand 返回的概念深潜内容
#[derive(Debug, Serialize, Deserialize)]
pub struct ConceptExpansion {
    #[serde(default)]
    pub term: String,
    #[serde(default)]
    pub expanded_definition: String,
    #[serde(default)]
    pub in_this_paper: String,
    #[serde(default)]
    pub analogy: String,
    #[serde(default)]
    pub example: String,
    #[serde(default)]
    pub common_misconceptions: String,
    #[serde(default)]
    pub intuition: String,
    #[serde(default)]
    pub mechanism_steps: Vec<MechanismStep>,
    #[serde(default)]
    pub interactive_demo: Option<InteractiveDemo>,
    #[serde(default)]
    pub contrast_cases: Vec<ContrastCase>,
    #[serde(default)]
    pub check_questions: Vec<CheckQuestion>,
    #[serde(default)]
    pub key_takeaways: Vec<String>,
    #[serde(default)]
    pub prerequisites: Vec<String>,
    #[serde(default)]
    pub paper_evidence: Vec<ConceptEvidence>,
    #[serde(default)]
    pub research_trail: Vec<ResearchStep>,
    #[serde(default)]
    pub reference_links: Vec<ReferenceLink>,
    #[serde(default)]
    pub external_queries: Vec<String>,
    #[serde(default)]
    pub related_concepts: Vec<String>,
    #[serde(default)]
    pub follow_up_questions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptEvidence {
    #[serde(default)]
    pub claim: String,
    #[serde(default)]
    pub quote: String,
    #[serde(default)]
    pub cite: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MechanismStep {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub input: String,
    #[serde(default)]
    pub process: String,
    #[serde(default)]
    pub output: String,
    #[serde(default)]
    pub why_it_matters: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveDemo {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub prompt: String,
    #[serde(default)]
    pub knobs: Vec<DemoKnob>,
    #[serde(default)]
    pub scenarios: Vec<DemoScenario>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoKnob {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub low_label: String,
    #[serde(default)]
    pub high_label: String,
    #[serde(default)]
    pub default_value: u8,
    #[serde(default)]
    pub effect: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoScenario {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub observation: String,
    #[serde(default)]
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContrastCase {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub without_concept: String,
    #[serde(default)]
    pub with_concept: String,
    #[serde(default)]
    pub lesson: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckQuestion {
    #[serde(default)]
    pub question: String,
    #[serde(default)]
    pub options: Vec<CheckOption>,
    #[serde(default)]
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckOption {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub correct: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchStep {
    #[serde(default)]
    pub question: String,
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub finding: String,
    #[serde(default)]
    pub confidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceLink {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub venue: Option<String>,
    #[serde(default)]
    pub year: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub relevance: String,
    #[serde(default)]
    pub source_type: String,
}
