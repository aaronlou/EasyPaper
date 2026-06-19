import { useEffect, useState } from "react";
import { usePaperStore } from "@/stores/usePaperStore";
import { renderBlock } from "@/renderer/blockRenderer";
import { ReaderProvider } from "@/contexts/ReaderContext";
import ConceptDeepDiveModal from "@/components/reader/ConceptDeepDiveModal";
import ConceptGraphBlock from "@/components/reader/ConceptGraphBlock";
import StudyPackPanel from "@/components/reader/StudyPackPanel";
import { ArrowRight, BookOpen, Layers3, Network, Route, Sparkles } from "lucide-react";
import type { SectionBlock as SectionBlockType } from "@/types";

interface Props {
  paperId: string;
}

type WorkspaceMode = "overview" | "research" | "concepts" | "reading";

const workspaceTabs: {
  id: WorkspaceMode;
  step: string;
  title: string;
  description: string;
  icon: typeof BookOpen;
}[] = [
  {
    id: "overview",
    step: "01",
    title: "建立总览",
    description: "先确认论文问题、核心贡献和阅读目标",
    icon: BookOpen,
  },
  {
    id: "research",
    step: "02",
    title: "研究地图",
    description: "把启发、结构、前置知识和后续方向串起来",
    icon: Route,
  },
  {
    id: "concepts",
    step: "03",
    title: "概念骨架",
    description: "通过关系图定位关键术语和依赖关系",
    icon: Network,
  },
  {
    id: "reading",
    step: "04",
    title: "正文深读",
    description: "回到章节、证据、图表和自测题",
    icon: Layers3,
  },
];

export default function ReaderView({ paperId }: Props) {
  const { current, loadPaper, loadingCurrent } = usePaperStore();
  const [activeConceptId, setActiveConceptId] = useState<string | null>(null);
  const [activeMode, setActiveMode] = useState<WorkspaceMode>("overview");

  useEffect(() => {
    loadPaper(paperId);
  }, [paperId, loadPaper]);

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

  const openSection = (sectionId: string) => {
    setActiveMode("reading");
    window.setTimeout(() => {
      document.getElementById(sectionId)?.scrollIntoView({
        behavior: "smooth",
        block: "start",
      });
    }, 0);
  };

  const openConcept = (conceptId: string) => {
    setActiveMode("concepts");
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
                    建议读法
                  </div>
                  <p className="text-sm leading-6 text-slate-600">
                    先获得整体判断，再用研究地图找学习路径，最后回到章节证据。
                  </p>
                </div>
              </div>
            </header>

            <WorkspaceTabs activeMode={activeMode} onChange={setActiveMode} />

            <div className="py-7">
              <WorkspacePane active={activeMode === "overview"}>
                <OverviewWorkspace
                  summary={interpretation.summary}
                  activeTab={workspaceTabs[0]}
                  onChangeMode={setActiveMode}
                  sectionsCount={sections.length}
                  conceptsCount={interpretation.concepts.length}
                />
              </WorkspacePane>

              <WorkspacePane active={activeMode === "research"}>
                <WorkspaceSection
                  activeTab={workspaceTabs[1]}
                  title="先把论文变成可行动的研究地图"
                  description="这一层回答论文带来的启发、它的写作结构、理解所需前置知识，以及可以继续深挖的研究方向。"
                  nextTab={workspaceTabs[2]}
                  onChangeMode={setActiveMode}
                >
                  <StudyPackPanel paperId={paperId} />
                </WorkspaceSection>
              </WorkspacePane>

              <WorkspacePane active={activeMode === "concepts"}>
                <WorkspaceSection
                  activeTab={workspaceTabs[2]}
                  title="建立概念之间的依赖关系"
                  description="先在图上定位关键概念，再进入概念实验室查看解释、机制、互动演示、校准题和证据链。"
                  nextTab={workspaceTabs[3]}
                  onChangeMode={setActiveMode}
                >
                  <ConceptGraphBlock
                    paperId={paperId}
                    concepts={interpretation.concepts}
                    onSelectConcept={setActiveConceptId}
                  />
                </WorkspaceSection>
              </WorkspacePane>

              <WorkspacePane active={activeMode === "reading"}>
                <WorkspaceSection
                  activeTab={workspaceTabs[3]}
                  title="回到正文逐段深读"
                  description="这里保留原论文解读块：章节、段落、图表、对比、代码片段和自测题会按正文顺序展开。"
                  onChangeMode={setActiveMode}
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
                  <Route className="h-4 w-4 text-sky-600" />
                  工作区
                </div>
                <div className="space-y-1">
                  {workspaceTabs.map((item) => (
                    <button
                      key={item.id}
                      onClick={() => setActiveMode(item.id)}
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

            {(activeMode === "concepts" || activeMode === "overview") &&
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
      aria-label="阅读工作区"
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
              className={`grid min-w-[180px] shrink-0 grid-cols-[28px_minmax(0,1fr)] items-start gap-2 rounded-lg border px-3 py-2.5 text-left transition md:min-w-0 md:flex-1 ${
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
  children,
}: {
  active: boolean;
  children: React.ReactNode;
}) {
  return (
    <div className={active ? "block" : "hidden"} aria-hidden={!active}>
      {children}
    </div>
  );
}

function OverviewWorkspace({
  summary,
  activeTab,
  onChangeMode,
  sectionsCount,
  conceptsCount,
}: {
  summary?: string;
  activeTab: (typeof workspaceTabs)[number];
  onChangeMode: (mode: WorkspaceMode) => void;
  sectionsCount: number;
  conceptsCount: number;
}) {
  return (
    <WorkspaceSection
      activeTab={activeTab}
      title="先建立论文的整体坐标"
      description="这一页只保留最重要的判断和下一步入口，避免一开始被所有内容淹没。"
      nextTab={workspaceTabs[1]}
      onChangeMode={onChangeMode}
    >
      <div className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_260px]">
        <div className="rounded-lg border border-slate-200 bg-white px-5 py-4">
          <div className="mb-3 text-sm font-semibold text-slate-500">核心贡献</div>
          <p className="text-base leading-8 text-slate-700">
            {summary || "当前论文暂未生成核心贡献摘要。"}
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

      <div className="mt-4 grid gap-3 md:grid-cols-3">
        {workspaceTabs.slice(1).map((item) => {
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
