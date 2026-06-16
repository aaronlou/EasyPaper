// ════════════════════════════════════════════════════════
//  Block 协议 —— 前后端共享的类型契约
//
//  与 backend/src/models/interpretation.rs 对应
// ════════════════════════════════════════════════════════

export type Block =
  | SectionBlock
  | ParagraphBlock
  | QuoteBlock
  | StatRowBlock
  | ConceptCardBlock
  | TimelineBlock
  | ComparisonBlock
  | QuizBlock
  | CodeFragmentBlock
  | CustomHtmlBlock;

export interface SectionBlock {
  type: "section";
  id: string;
  num: string;
  title: string;
}

export interface ParagraphBlock {
  type: "paragraph";
  id: string;
  text: string;
}

export interface QuoteBlock {
  type: "quote";
  id: string;
  text: string;
  cite?: string;
}

export interface StatRowBlock {
  type: "stat_row";
  id: string;
  stats: { value: string; label: string }[];
}

export interface ConceptCardBlock {
  type: "concept_card";
  id: string;
  term: string;
  definition: string;
  icon?: string;
}

export interface TimelineBlock {
  type: "timeline";
  id: string;
  items: TimelineItem[];
}

export interface TimelineItem {
  year: string;
  title: string;
  body: string;
}

export interface ComparisonBlock {
  type: "comparison";
  id: string;
  columns: string[];
  rows: { label: string; cells: string[] }[];
}

export interface QuizBlock {
  type: "quiz";
  id: string;
  question: string;
  options: { text: string; correct: boolean }[];
  explain: string;
}

export interface CodeFragmentBlock {
  type: "code_fragment";
  id: string;
  lang: string;
  code: string;
}

export interface CustomHtmlBlock {
  type: "custom_html";
  id: string;
  html: string;
}

// ── Concept ────────────────────────

export interface Concept {
  id: string;
  term: string;
  definition: string;
  difficulty: "basic" | "intermediate" | "advanced";
  related: string[];
}

// ── Interpretation ─────────────────

export interface Interpretation {
  paper_id: string;
  blocks: Block[];
  concepts: Concept[];
  summary?: string;
}

// ── Paper ──────────────────────────

export interface PaperSummary {
  id: string;
  filename: string;
  title: string;
  authors: string[];
  char_count: number;
  status: "uploaded" | "processing" | "completed" | "failed";
  created_at: string;
  completed_at?: string;
}

export interface PaperDetail {
  paper: PaperSummary;
  interpretation?: Interpretation;
}

// ── API ────────────────────────────

export interface HealthResponse {
  status: string;
  service: string;
  version: string;
  llm_configured: boolean;
}

export interface UploadResponse {
  paper: PaperSummary;
}
