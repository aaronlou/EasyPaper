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
  | CustomHtmlBlock
  | FigureBlock
  | ChartBlock
  | DiagramBlock;

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

export interface FigureBlock {
  type: "figure";
  id: string;
  svg: string;
  caption?: string;
}

export interface ChartDataPoint {
  label: string;
  value: number;
}

export interface ChartBlock {
  type: "chart";
  id: string;
  chart_type: "bar" | "line" | "pie";
  title?: string;
  data: ChartDataPoint[];
  x_label?: string;
  y_label?: string;
}

export interface DiagramBlock {
  type: "diagram";
  id: string;
  svg: string;
  caption?: string;
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

// ── Progress ───────────────────────

export interface ProgressInfo {
  phase:
    | "uploaded"
    | "interpreting"
    | "reading"
    | "parsing"
    | "saving"
    | "completed"
    | "failed";
  stage: string;
  message: string;
  percent: number;
  updated_at: string;
}

// ── Concept Expansion ──────────────

export interface ConceptExpansion {
  term: string;
  expanded_definition: string;
  in_this_paper: string;
  analogy: string;
  example: string;
  common_misconceptions: string;
  intuition: string;
  mechanism_steps: MechanismStep[];
  interactive_demo?: InteractiveDemo | null;
  contrast_cases: ContrastCase[];
  check_questions: CheckQuestion[];
  key_takeaways: string[];
  prerequisites: string[];
  paper_evidence: ConceptEvidence[];
  research_trail: ResearchStep[];
  reference_links: ReferenceLink[];
  external_queries: string[];
  related_concepts: string[];
  follow_up_questions: string[];
}

export interface ConceptEvidence {
  claim: string;
  quote: string;
  cite?: string;
}

export interface MechanismStep {
  title: string;
  input: string;
  process: string;
  output: string;
  why_it_matters: string;
}

export interface InteractiveDemo {
  title: string;
  prompt: string;
  knobs: DemoKnob[];
  scenarios: DemoScenario[];
}

export interface DemoKnob {
  name: string;
  low_label: string;
  high_label: string;
  default_value: number;
  effect: string;
}

export interface DemoScenario {
  label: string;
  observation: string;
  explanation: string;
}

export interface ContrastCase {
  label: string;
  without_concept: string;
  with_concept: string;
  lesson: string;
}

export interface CheckQuestion {
  question: string;
  options: CheckOption[];
  explanation: string;
}

export interface CheckOption {
  text: string;
  correct: boolean;
}

export interface ResearchStep {
  question: string;
  action: string;
  finding: string;
  confidence: "high" | "medium" | "low" | string;
}

export interface ReferenceLink {
  title: string;
  authors: string[];
  venue?: string;
  year?: string;
  url?: string;
  relevance: string;
  source_type: "paper" | "web" | "paper_reference" | "inferred" | string;
}
