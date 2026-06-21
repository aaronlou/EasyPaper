import { useEffect, useState } from "react";
import { usePaperStore } from "@/stores/usePaperStore";
import { renderBlock } from "@/renderer/blockRenderer";
import { ReaderProvider } from "@/contexts/ReaderContext";
import ConceptDeepDiveModal from "@/components/reader/ConceptDeepDiveModal";
import ConceptGraphBlock from "@/components/reader/ConceptGraphBlock";
import StudyPackPanel from "@/components/reader/StudyPackPanel";
import {
  ArrowRight,
  BookOpen,
  Compass,
  Layers3,
  Lightbulb,
  Network,
  Route,
  SearchCheck,
  Sparkles,
} from "lucide-react";
import type {
  Block,
  SectionBlock as SectionBlockType,
} from "@/types";

interface Props {
  paperId: string;
}

type WorkspaceMode =
  | "problem"
  | "method"
  | "innovation"
  | "evidence"
  | "concepts"
  | "reading";

const workspaceTabs: {
  id: WorkspaceMode;
  step: string;
  title: string;
  description: string;
  icon: typeof BookOpen;
}[] = [
  {
    id: "problem",
    step: "01",
    title: "问题定位",
    description: "这篇 paper 到底想解决什么问题",
    icon: SearchCheck,
  },
  {
    id: "method",
    step: "02",
    title: "如何做的",
    description: "方法路径、结构逻辑和核心机制",
    icon: Route,
  },
  {
    id: "innovation",
    step: "03",
    title: "创新借鉴",
    description: "关键创新点和可迁移的启发",
    icon: Lightbulb,
  },
  {
    id: "evidence",
    step: "04",
    title: "证据评估",
    description: "论文用什么证据支撑结论",
    icon: BookOpen,
  },
  {
    id: "concepts",
    step: "05",
    title: "概念骨架",
    description: "关键术语、依赖关系和概念实验室",
    icon: Network,
  },
  {
    id: "reading",
    step: "06",
    title: "正文深读",
    description: "回到章节、图表、段落和自测",
    icon: Layers3,
  },
];

export default function ReaderView({ paperId }: Props) {
  const { current, loadPaper, loadingCurrent } = usePaperStore();
  const [activeConceptId, setActiveConceptId] = useState<string | null>(null);
  const [activeMode, setActiveMode] = useState<WorkspaceMode>("problem");
  const [visitedModes, setVisitedModes] = useState<Set<WorkspaceMode>>(
    () => new Set(["problem"]),
  );

  useEffect(() => {
    loadPaper(paperId);
  }, [paperId, loadPaper]);

  const changeMode = (mode: WorkspaceMode) => {
    setActiveMode(mode);
    setVisitedModes((prev) => {
      if (prev.has(mode)) return prev;
      const next = new Set(prev);
      next.add(mode);
      return next;
    });
  };

  if (loadingCurrent || !current || !current.interpretation) {
    return (
      <main className="max-w-3xl mx-auto px-6 py-32 text-center text-gray-400">
        <span className="inline-block w-8 h-8 border-3 border-gray-300 border-t-gray-500 rounded-full animate-spin mb-4" />
        <p>加载中...</p>
      </main>
    );
  }

  const { interpretation, paper } = current;
  const blocks = interpretation.blocks;
  const sections = blocks.filter(
    (block): block is SectionBlockType => block.type === "section",
  );
  const evidenceBlocks = blocks.filter(isEvidenceBlock);
  const methodBlocks = blocks.filter(isMethodBlock);

  const openSection = (sectionId: string) => {
    changeMode("reading");
    window.setTimeout(() => {
      document.getElementById(sectionId)?.scrollIntoView({
        behavior: "smooth",
        block: "start",
      });
    }, 0);
  };

  const openConcept = (conceptId: string) => {
    changeMode("concepts");
    setActiveConceptId(conceptId);
  };

  return (
    <ReaderProvider
      paperId={paperId}
      interpretation={interpretation}
      activeConceptId={activeConceptId}
      setActiveConceptId={setActiveConceptId}
    >
      <main className="min-h-[calc(100vh-64px)] bg-[#f7f8fb]">
        <div className="mx-auto grid max-w-[1440px] gap-8 px-4 py-6 pb-28 lg:px-8 xl:grid-cols-[minmax(0,1fr)_300px]">
          <article className="min-w-0">
            <header className="border-b border-slate-200 pb-6">
              <div className="mb-5 flex flex-wrap items-center gap-3 text-xs font-semibold uppercase text-slate-500">
                <span className="inline-flex items-center gap-1.5 rounded-md border border-slate-200 bg-white px-2.5 py-1 text-sky-700">
                  <BookOpen className="h-3.5 w-3.5" />
                  Paper Reader
                </span>
                <span>{paper.char_count.toLocaleString()} chars</span>
                <span>{sections.length} sections</span>
                <span>{interpretation.concepts.length} concepts</span>
              </div>

              <div className="grid gap-6 lg:grid-cols-[minmax(0,1fr)_260px]">
                <div>
                  <h1 className="max-w-5xl text-3xl font-semibold leading-tight tracking-normal text-slate-950 md:text-4xl">
                    {paper.title}
                  </h1>
                  {paper.authors.length > 0 && (
                    <p className="mt-3 max-w-4xl text-sm leading-6 text-slate-500">
                      {paper.authors.join(", ")}
                    </p>
                  )}
                </div>

                <div className="rounded-lg border border-slate-200 bg-white px-4 py-3">
                  <div className="mb-2 flex items-center gap-2 text-sm font-semibold text-slate-900">
                    <Sparkles className="h-4 w-4 text-sky-600" />
                    推荐读法
                  </div>
                  <p className="text-sm leading-6 text-slate-600">
                    先回答论文问题，再看方法、创新、证据，最后进入概念和正文。
                  </p>
                </div>
              </div>
            </header>

            <WorkspaceTabs activeMode={activeMode} onChange={changeMode} />

            <div className="py-7">
              <WorkspacePane active={activeMode === "problem"} mounted={visitedModes.has("problem")}>
                <ProblemWorkspace
                  summary={interpretation.summary}
                  activeTab={workspaceTabs[0]}
                  onChangeMode={changeMode}
                  paperTitle={paper.title}
                  sectionsCount={sections.length}
                  conceptsCount={interpretation.concepts.length}
                  firstSections={sections.slice(0, 4)}
                />
              </WorkspacePane>

              <WorkspacePane active={activeMode === "method"} mounted={visitedModes.has("method")}>
                <WorkspaceSection
                  activeTab={workspaceTabs[1]}
                  title="它是如何做的"
                  description="这里先看论文的结构和方法链路。目标不是背模块名，而是理解作者如何把问题拆成可操作步骤。"
                  nextTab={workspaceTabs[2]}
                  onChangeMode={changeMode}
                >
                  <StudyPackPanel paperId={paperId} initialTab="structure" />
                  {methodBlocks.length > 0 && (
                    <BlockPreviewStrip
                      title="方法相关图表与对比"
                      blocks={methodBlocks.slice(0, 4)}
                    />
                  )}
                </WorkspaceSection>
              </WorkspacePane>

              <WorkspacePane
                active={activeMode === "innovation"}
                mounted={visitedModes.has("innovation")}
              >
                <WorkspaceSection
                  activeTab={workspaceTabs[2]}
                  title="关键创新点和可借鉴之处"
                  description="读 paper 不只是知道它做了什么，还要提炼哪些设计、论证或工程取舍能迁移到自己的工作。"
                  nextTab={workspaceTabs[3]}
                  onChangeMode={changeMode}
                >
                  <StudyPackPanel paperId={paperId} initialTab="inspiration" />
                </WorkspaceSection>
              </WorkspacePane>

              <WorkspacePane active={activeMode === "evidence"} mounted={visitedModes.has("evidence")}>
                <WorkspaceSection
                  activeTab={workspaceTabs[3]}
                  title="证据是否支撑了论文主张"
                  description="先集中看引用、指标、图表和对比，再回到正文检查每个结论有没有落在证据上。"
                  nextTab={workspaceTabs[4]}
                  onChangeMode={changeMode}
                >
                  <EvidenceWorkspace blocks={evidenceBlocks} onOpenReading={() => changeMode("reading")} />
                </WorkspaceSection>
              </WorkspacePane>

              <WorkspacePane active={activeMode === "concepts"} mounted={visitedModes.has("concepts")}>
                <WorkspaceSection
                  activeTab={workspaceTabs[4]}
                  title="理解关键概念之间的依赖关系"
                  description="先在图上定位关键概念，再进入概念实验室查看解释、机制、互动演示、校准题和证据链。"
                  nextTab={workspaceTabs[5]}
                  onChangeMode={changeMode}
                >
                  <ConceptGraphBlock
                    paperId={paperId}
                    concepts={interpretation.concepts}
                    onSelectConcept={setActiveConceptId}
                  />
                </WorkspaceSection>
              </WorkspacePane>

              <WorkspacePane active={activeMode === "reading"} mounted={visitedModes.has("reading")}>
                <WorkspaceSection
                  activeTab={workspaceTabs[5]}
                  title="回到正文逐段深读"
                  description="这里保留原论文解读块：章节、段落、图表、对比、代码片段和自测题会按正文顺序展开。"
                  onChangeMode={changeMode}
                >
                  <div className="rounded-lg border border-slate-200 bg-white px-5 py-2 sm:px-7 sm:py-4">
                    {blocks.map((block) => (
                      <div
                        key={block.id}
                        id={block.type === "section" ? block.id : undefined}
                        className={block.type === "section" ? "scroll-mt-24" : undefined}
                      >
                        {renderBlock(block)}
                      </div>
                    ))}
                  </div>
                </WorkspaceSection>
              </WorkspacePane>
            </div>
          </article>

          <aside className="hidden xl:block">
            <div className="sticky top-20 max-h-[calc(100vh-6rem)] space-y-4 overflow-y-auto pr-1">
              <nav className="reader-panel p-4">
                <div className="mb-3 flex items-center gap-2 text-sm font-semibold text-slate-900">
                  <Compass className="h-4 w-4 text-sky-600" />
                  阅读问题链
                </div>
                <div className="space-y-1">
                  {workspaceTabs.map((item) => (
                    <button
                      key={item.id}
                      onClick={() => changeMode(item.id)}
                      className={`group grid w-full grid-cols-[28px_minmax(0,1fr)] gap-2 rounded-md px-2 py-2 text-left text-sm transition ${
                        activeMode === item.id
                          ? "bg-slate-950 text-white"
                          : "text-slate-600 hover:bg-sky-50"
                      }`}
                    >
                      <span
                        className={`font-mono text-xs font-semibold ${
                          activeMode === item.id
                            ? "text-white/60"
                            : "text-slate-300 group-hover:text-sky-600"
                        }`}
                      >
                        {item.step}
                      </span>
                      <span>
                        <span
                          className={`block font-medium ${
                            activeMode === item.id
                              ? "text-white"
                              : "text-slate-700 group-hover:text-sky-700"
                          }`}
                        >
                          {item.title}
                        </span>
                        <span
                          className={`mt-0.5 block text-xs leading-5 ${
                            activeMode === item.id ? "text-white/55" : "text-slate-400"
                          }`}
                        >
                          {item.description}
                        </span>
                      </span>
                    </button>
                  ))}
                </div>
              </nav>

            {activeMode === "reading" && sections.length > 0 && (
              <nav className="reader-panel p-4">
                <div className="mb-3 flex items-center gap-2 text-sm font-semibold text-slate-800">
                  <Layers3 className="h-4 w-4 text-sky-600" />
                  章节
                </div>
                <div className="space-y-1">
                  {sections.map((section) => (
                    <button
                      key={section.id}
                      onClick={() => openSection(section.id)}
                      className="block w-full rounded-md px-2 py-1.5 text-left text-sm text-slate-500 transition hover:bg-sky-50 hover:text-sky-700"
                    >
                      <span className="mr-2 font-mono text-xs text-slate-300">
                        {section.num}
                      </span>
                      {section.title}
                    </button>
                  ))}
                </div>
              </nav>
            )}

            {(activeMode === "concepts" || activeMode === "problem") &&
              interpretation.concepts.length > 0 && (
              <div className="reader-panel p-4">
                <div className="mb-3 text-sm font-semibold text-slate-800">
                  概念索引
                </div>
                <div className="flex flex-wrap gap-2">
                  {interpretation.concepts.slice(0, 14).map((concept) => (
                    <button
                      key={concept.id}
                      onClick={() => openConcept(concept.id)}
                      className="rounded-full border border-slate-200 bg-white px-2.5 py-1 text-xs text-slate-600 transition hover:border-sky-300 hover:bg-sky-50 hover:text-sky-700"
                    >
                      {concept.term}
                    </button>
                  ))}
                </div>
              </div>
            )}
            </div>
          </aside>
        </div>
      </main>

      <ConceptDeepDiveModal
        conceptId={activeConceptId}
        onClose={() => setActiveConceptId(null)}
      />
    </ReaderProvider>
  );
}

function WorkspaceTabs({
  activeMode,
  onChange,
}: {
  activeMode: WorkspaceMode;
  onChange: (mode: WorkspaceMode) => void;
}) {
  return (
    <nav
      aria-label="阅读问题链"
      className="sticky top-[57px] z-30 -mx-4 border-b border-slate-200 bg-[#f7f8fb]/95 px-4 py-3 backdrop-blur lg:-mx-8 lg:px-8"
    >
      <div className="flex gap-2 overflow-x-auto">
        {workspaceTabs.map((item) => {
          const Icon = item.icon;
          const active = item.id === activeMode;
          return (
            <button
              key={item.id}
              onClick={() => onChange(item.id)}
              className={`grid min-w-[188px] shrink-0 grid-cols-[28px_minmax(0,1fr)] items-start gap-2 rounded-lg border px-3 py-2.5 text-left transition md:min-w-[160px] md:flex-1 ${
                active
                  ? "border-slate-950 bg-slate-950 text-white shadow-sm"
                  : "border-slate-200 bg-white text-slate-600 hover:border-sky-300 hover:bg-sky-50"
              }`}
            >
              <span
                className={`mt-0.5 flex h-6 w-6 items-center justify-center rounded-md border ${
                  active
                    ? "border-white/15 bg-white/10 text-white"
                    : "border-slate-200 bg-slate-50 text-slate-400"
                }`}
              >
                <Icon className="h-3.5 w-3.5" />
              </span>
              <span className="min-w-0">
                <span className="flex items-center gap-2">
                  <span
                    className={`font-mono text-[11px] font-semibold ${
                      active ? "text-white/55" : "text-slate-400"
                    }`}
                  >
                    {item.step}
                  </span>
                  <span className="text-sm font-semibold">{item.title}</span>
                </span>
                <span
                  className={`mt-0.5 block truncate text-xs ${
                    active ? "text-white/60" : "text-slate-400"
                  }`}
                >
                  {item.description}
                </span>
              </span>
            </button>
          );
        })}
      </div>
    </nav>
  );
}

function WorkspacePane({
  active,
  mounted,
  children,
}: {
  active: boolean;
  mounted: boolean;
  children: React.ReactNode;
}) {
  if (!mounted) return null;
  return (
    <div className={active ? "block" : "hidden"} aria-hidden={!active}>
      {children}
    </div>
  );
}

function ProblemWorkspace({
  summary,
  activeTab,
  onChangeMode,
  paperTitle,
  sectionsCount,
  conceptsCount,
  firstSections,
}: {
  summary?: string;
  activeTab: (typeof workspaceTabs)[number];
  onChangeMode: (mode: WorkspaceMode) => void;
  paperTitle: string;
  sectionsCount: number;
  conceptsCount: number;
  firstSections: SectionBlockType[];
}) {
  return (
    <WorkspaceSection
      activeTab={activeTab}
      title="这篇 paper 要解决什么问题"
      description="读论文的第一步不是看模型细节，而是先判断：作者面对的困难是什么，为什么旧方法不够，核心主张是什么。"
      nextTab={workspaceTabs[1]}
      onChangeMode={onChangeMode}
    >
      <div className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_280px]">
        <div className="rounded-lg border border-slate-200 bg-white px-5 py-4">
          <div className="mb-2 text-sm font-semibold text-slate-500">论文主张</div>
          <h3 className="text-xl font-semibold leading-tight text-slate-950">
            {summary || paperTitle}
          </h3>
          <p className="mt-3 text-sm leading-7 text-slate-600">
            接下来建议按右侧问题链阅读：先看方法如何回应这个问题，再判断创新、证据和可借鉴点。
          </p>
        </div>

        <div className="rounded-lg border border-slate-200 bg-white px-5 py-4">
          <div className="mb-4 text-sm font-semibold text-slate-900">阅读资产</div>
          <div className="space-y-3 text-sm">
            <div className="flex items-center justify-between border-b border-slate-100 pb-3">
              <span className="text-slate-500">章节</span>
              <span className="font-mono font-semibold text-slate-950">
                {sectionsCount}
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-slate-500">概念</span>
              <span className="font-mono font-semibold text-slate-950">
                {conceptsCount}
              </span>
            </div>
          </div>
        </div>
      </div>

      {firstSections.length > 0 && (
        <div className="mt-4 rounded-lg border border-slate-200 bg-white px-5 py-4">
          <div className="mb-3 text-sm font-semibold text-slate-900">
            先扫这些章节标题
          </div>
          <div className="grid gap-2 md:grid-cols-2">
            {firstSections.map((section) => (
              <div
                key={section.id}
                className="rounded-md bg-slate-50 px-3 py-2 text-sm text-slate-600"
              >
                <span className="mr-2 font-mono text-xs text-slate-400">
                  {section.num}
                </span>
                {section.title}
              </div>
            ))}
          </div>
        </div>
      )}

      <div className="mt-4 grid gap-3 md:grid-cols-3">
        {workspaceTabs.slice(1, 4).map((item) => {
          const Icon = item.icon;
          return (
            <button
              key={item.id}
              onClick={() => onChangeMode(item.id)}
              className="group rounded-lg border border-slate-200 bg-white px-4 py-4 text-left transition hover:border-sky-300 hover:bg-sky-50"
            >
              <div className="mb-4 flex items-center justify-between gap-3">
                <span className="font-mono text-xs font-semibold text-slate-400">
                  {item.step}
                </span>
                <Icon className="h-4 w-4 text-slate-400 group-hover:text-sky-600" />
              </div>
              <div className="text-sm font-semibold text-slate-950">
                {item.title}
              </div>
              <p className="mt-1 text-xs leading-5 text-slate-500">
                {item.description}
              </p>
            </button>
          );
        })}
      </div>
    </WorkspaceSection>
  );
}

function EvidenceWorkspace({
  blocks,
  onOpenReading,
}: {
  blocks: Block[];
  onOpenReading: () => void;
}) {
  if (blocks.length === 0) {
    return (
      <div className="rounded-lg border border-slate-200 bg-white px-5 py-6">
        <div className="text-sm font-semibold text-slate-950">暂未提取到独立证据块</div>
        <p className="mt-2 text-sm leading-6 text-slate-600">
          可以进入正文深读查看原文引用、实验段落和图表解释。
        </p>
        <button
          onClick={onOpenReading}
          className="mt-4 inline-flex items-center gap-2 rounded-md bg-slate-950 px-3 py-2 text-sm font-semibold text-white"
        >
          去正文深读
          <ArrowRight className="h-4 w-4" />
        </button>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="rounded-lg border border-slate-200 bg-white px-5 py-4">
        <div className="text-sm font-semibold text-slate-950">证据线索</div>
        <p className="mt-2 text-sm leading-6 text-slate-600">
          这里集中展示引用、指标、图表和方案对比。读的时候重点判断：这些证据是否真的支持论文主张。
        </p>
      </div>
      <div className="space-y-4">
        {blocks.slice(0, 8).map((block) => (
          <div key={block.id}>{renderBlock(block)}</div>
        ))}
      </div>
    </div>
  );
}

function BlockPreviewStrip({ title, blocks }: { title: string; blocks: Block[] }) {
  if (blocks.length === 0) return null;

  return (
    <section className="mt-5 rounded-lg border border-slate-200 bg-white px-5 py-4">
      <div className="mb-3 text-sm font-semibold text-slate-950">{title}</div>
      <div className="space-y-4">
        {blocks.map((block) => (
          <div key={block.id}>{renderBlock(block)}</div>
        ))}
      </div>
    </section>
  );
}

function isEvidenceBlock(block: Block) {
  return (
    block.type === "quote" ||
    block.type === "stat_row" ||
    block.type === "chart" ||
    block.type === "comparison" ||
    block.type === "figure"
  );
}

function isMethodBlock(block: Block) {
  if (block.type === "diagram") return true;
  if (block.type === "comparison") return true;
  if (block.type !== "figure") return false;
  const caption = block.caption?.toLowerCase() ?? "";
  return /method|architecture|pipeline|framework|机制|方法|架构|流程/.test(caption);
}

function WorkspaceSection({
  activeTab,
  title,
  description,
  nextTab,
  onChangeMode,
  children,
}: {
  activeTab: (typeof workspaceTabs)[number];
  title: string;
  description: string;
  nextTab?: (typeof workspaceTabs)[number];
  onChangeMode: (mode: WorkspaceMode) => void;
  children: React.ReactNode;
}) {
  const Icon = activeTab.icon;

  return (
    <section className="min-h-[560px]">
      <div className="mb-5 flex flex-col gap-4 border-b border-slate-200 pb-5 md:flex-row md:items-end md:justify-between">
        <div>
          <div className="mb-3 flex items-center gap-2 text-xs font-semibold uppercase text-sky-700">
            <span className="flex h-7 w-7 items-center justify-center rounded-md border border-sky-100 bg-white">
              <Icon className="h-4 w-4" />
            </span>
            <span>{activeTab.step}</span>
            <span>{activeTab.title}</span>
          </div>
          <h2 className="text-2xl font-semibold leading-tight text-slate-950">
            {title}
          </h2>
          <p className="mt-2 max-w-3xl text-sm leading-6 text-slate-600">
            {description}
          </p>
        </div>

        {nextTab && (
          <button
            onClick={() => onChangeMode(nextTab.id)}
            className="inline-flex shrink-0 items-center justify-center gap-2 rounded-md border border-slate-200 bg-white px-3 py-2 text-sm font-semibold text-slate-700 transition hover:border-sky-300 hover:bg-sky-50 hover:text-sky-700"
          >
            下一步：{nextTab.title}
            <ArrowRight className="h-4 w-4" />
          </button>
        )}
      </div>
      {children}
    </section>
  );
}
