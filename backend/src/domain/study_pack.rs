use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyPack {
    pub paper_id: Uuid,
    #[serde(default)]
    pub inspiration: Vec<InsightItem>,
    #[serde(default)]
    pub structure_logic: Vec<StructureMove>,
    #[serde(default)]
    pub prerequisites: Vec<Prerequisite>,
    #[serde(default)]
    pub research_directions: Vec<ResearchDirection>,
    #[serde(default)]
    pub literature_review: Vec<LineageItem>,
    #[serde(default)]
    pub lineage: ResearchLineage,
    #[serde(default)]
    pub translation: Option<TranslationSummary>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct StudyPackDraft {
    #[serde(default)]
    pub inspiration: Vec<InsightItem>,
    #[serde(default)]
    pub structure_logic: Vec<StructureMove>,
    #[serde(default)]
    pub prerequisites: Vec<Prerequisite>,
    #[serde(default)]
    pub research_directions: Vec<ResearchDirection>,
    #[serde(default)]
    pub literature_review: Vec<LineageItem>,
    #[serde(default)]
    pub lineage: ResearchLineage,
    #[serde(default)]
    pub translation: Option<TranslationSummary>,
}

impl StudyPackDraft {
    pub fn into_study_pack(self, paper_id: Uuid, now: String) -> StudyPack {
        StudyPack {
            paper_id,
            inspiration: self.inspiration,
            structure_logic: self.structure_logic,
            prerequisites: self.prerequisites,
            research_directions: self.research_directions,
            literature_review: self.literature_review,
            lineage: self.lineage,
            translation: self.translation,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InsightItem {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub explanation: String,
    #[serde(default)]
    pub how_to_apply: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StructureMove {
    #[serde(default)]
    pub step: String,
    #[serde(default)]
    pub purpose: String,
    #[serde(default)]
    pub why_it_works: String,
    #[serde(default)]
    pub writing_takeaway: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Prerequisite {
    #[serde(default)]
    pub topic: String,
    #[serde(default)]
    pub why_needed: String,
    #[serde(default)]
    pub minimum_goal: String,
    #[serde(default)]
    pub references: Vec<StudyReference>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResearchDirection {
    #[serde(default)]
    pub question: String,
    #[serde(default)]
    pub motivation: String,
    #[serde(default)]
    pub possible_method: String,
    #[serde(default)]
    pub first_step: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LineageItem {
    #[serde(default)]
    pub stage: String,
    #[serde(default)]
    pub idea: String,
    #[serde(default)]
    pub representative_work: Vec<StudyReference>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResearchLineage {
    #[serde(default)]
    pub builds_on: Vec<StudyReference>,
    #[serde(default)]
    pub follow_ups: Vec<StudyReference>,
    #[serde(default)]
    pub search_queries: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StudyReference {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub year: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub relevance: String,
    #[serde(default)]
    pub source_type: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TranslationSummary {
    #[serde(default)]
    pub source_language: String,
    #[serde(default)]
    pub target_language: String,
    #[serde(default)]
    pub glossary: Vec<TranslationGlossaryItem>,
    #[serde(default)]
    pub sections: Vec<TranslatedSection>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TranslatedSection {
    #[serde(default)]
    pub heading: String,
    #[serde(default)]
    pub original_excerpt: String,
    #[serde(default)]
    pub translated_text: String,
    #[serde(default)]
    pub expression_notes: Vec<ExpressionNote>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TranslationGlossaryItem {
    #[serde(default)]
    pub term: String,
    #[serde(default)]
    pub translation: String,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExpressionNote {
    #[serde(default)]
    pub english: String,
    #[serde(default)]
    pub chinese: String,
    #[serde(default)]
    pub usage: String,
}
