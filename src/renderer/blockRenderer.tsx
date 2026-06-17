// ── Block 渲染引擎：把 Block JSON 映射到 React 组件

import type { Block } from "@/types";
import SectionBlock from "@/components/reader/SectionBlock";
import ParagraphBlock from "@/components/reader/ParagraphBlock";
import QuoteBlock from "@/components/reader/QuoteBlock";
import StatRowBlock from "@/components/reader/StatRowBlock";
import ConceptCardBlock from "@/components/reader/ConceptCardBlock";
import TimelineBlock from "@/components/reader/TimelineBlock";
import ComparisonBlock from "@/components/reader/ComparisonBlock";
import QuizBlock from "@/components/reader/QuizBlock";
import CodeFragmentBlock from "@/components/reader/CodeFragmentBlock";
import CustomHtmlBlock from "@/components/reader/CustomHtmlBlock";
import FigureBlock from "@/components/reader/FigureBlock";
import ChartBlock from "@/components/reader/ChartBlock";
import DiagramBlock from "@/components/reader/DiagramBlock";

export function renderBlock(block: Block): React.ReactNode {
  switch (block.type) {
    case "section":
      return <SectionBlock block={block} />;
    case "paragraph":
      return <ParagraphBlock block={block} />;
    case "quote":
      return <QuoteBlock block={block} />;
    case "stat_row":
      return <StatRowBlock block={block} />;
    case "concept_card":
      return <ConceptCardBlock block={block} />;
    case "timeline":
      return <TimelineBlock block={block} />;
    case "comparison":
      return <ComparisonBlock block={block} />;
    case "quiz":
      return <QuizBlock block={block} />;
    case "code_fragment":
      return <CodeFragmentBlock block={block} />;
    case "custom_html":
      return <CustomHtmlBlock block={block} />;
    case "figure":
      return <FigureBlock block={block} />;
    case "chart":
      return <ChartBlock block={block} />;
    case "diagram":
      return <DiagramBlock block={block} />;
    default:
      return null;
  }
}
