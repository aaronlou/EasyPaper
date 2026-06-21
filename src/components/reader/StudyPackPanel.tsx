import { useEffect, useState } from "react";
import {
  BookMarked,
  Compass,
  FileText,
  GitBranch,
  Languages,
  Lightbulb,
  Loader2,
  Network,
  Sparkles,
} from "lucide-react";
import * as api from "@/lib/api";
import type {
  InsightItem,
  LineageItem,
  Prerequisite,
  ResearchDirection,
  StudyPack,
  StudyReference,
  StructureMove,
} from "@/types";
import { cn } from "@/lib/cn";

interface Props {
  paperId: string;
  initialTab?: TabKey;
}

type TabKey =
  | "inspiration"
  | "structure"
  | "prerequisites"
  | "directions"
  | "review"
  | "lineage"
  | "translation";

const tabs: {
  key: TabKey;
  label: string;
  description: string;
  icon: React.ReactNode;
}[] = [
  {
    key: "inspiration",
    label: "论文启发",
    description: "哪些思想值得迁移",
    icon: <Lightbulb className="h-4 w-4" />,
  },
  {
    key: "structure",
    label: "结构逻辑",
    description: "作者如何组织论证",
    icon: <FileText className="h-4 w-4" />,
  },
  {
    key: "prerequisites",
    label: "前提知识",
    description: "先补哪些基础",
    icon: <BookMarked className="h-4 w-4" />,
  },
  {
    key: "review",
    label: "文献脉络",
    description: "思想如何演进",
    icon: <Network className="h-4 w-4" />,
  },
  {
    key: "lineage",
    label: "前后继",
    description: "基于谁，又影响谁",
    icon: <GitBranch className="h-4 w-4" />,
  },
  {
    key: "directions",
    label: "继续研究",
    description: "还能挖哪些问题",
    icon: <Compass className="h-4 w-4" />,
  },
  {
    key: "translation",
    label: "中文翻译",
    description: "英文论文辅助理解",
    icon: <Languages className="h-4 w-4" />,
  },
];

const tabCopy: Record<TabKey, { eyebrow: string; title: string; description: string }> = {
  inspiration: {
    eyebrow: "Step 1",
    title: "本论文给你的研究启发",
    description: "先看最值得带走的思想，再判断它能否迁移到自己的问题。",
  },
  structure: {
    eyebrow: "Step 2",
    title: "论文结构与论证方法",
    description: "拆解作者如何提出问题、铺垫方法、组织证据和收束贡献。",
  },
  prerequisites: {
    eyebrow: "Step 3",
    title: "充分理解前需要补齐的知识",
    description: "把陌生领域拆成可学习的前置概念，并尽量给出参考资料线索。",
  },
  review: {
    eyebrow: "Step 4",
    title: "相关思想的发展脉络",
    description: "像文献综述一样梳理这条研究线的关键阶段和代表性工作。",
  },
  lineage: {
    eyebrow: "Step 5",
    title: "本论文的前序基础与后续继承",
    description: "理解它从哪里来、可能通向哪里，以及继续检索时应该用哪些 query。",
  },
  directions: {
    eyebrow: "Step 6",
    title: "可以继续挖掘的研究问题",
    description: "把阅读后的直觉转成更具体的问题、方法和第一步行动。",
  },
  translation: {
    eyebrow: "Reference",
    title: "中文翻译辅助理解",
    description: "对于英文论文，提供更顺畅的中文摘要式翻译，帮助快速回看。",
  },
};

export default function StudyPackPanel({ paperId, initialTab }: Props) {
  const [pack, setPack] = useState<StudyPack | null>(null);
  const [activeTab, setActiveTab] = useState<TabKey>(initialTab ?? "inspiration");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (initialTab) setActiveTab(initialTab);
  }, [initialTab]);

  const load = async () => {
    setLoading(true);
    setError(null);
    try {
      setPack(await api.getStudyPack(paperId));
    } catch (err) {
      setError(err instanceof Error ? err.message : "研究地图生成失败");
    } finally {
      setLoading(false);
    }
  };

  return (
    <section className="overflow-hidden rounded-lg border border-slate-200 bg-white">
      <div className="border-b border-slate-200 px-5 py-4">
        <div className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
          <div>
            <div className="mb-1 flex items-center gap-2 text-sm font-semibold text-sky-700">
              <Sparkles className="h-4 w-4" />
              AI Study Pack
            </div>
            <p className="max-w-2xl text-sm leading-6 text-slate-600">
              生成后会缓存研究地图；再次打开相同论文时优先读取缓存，避免重复消耗模型资源。
            </p>
          </div>
          <button
            onClick={load}
            disabled={loading}
            className="inline-flex items-center justify-center gap-2 rounded-md bg-slate-950 px-4 py-2 text-sm font-semibold text-white transition hover:bg-sky-700 disabled:opacity-60"
          >
            {loading ? <Loader2 className="h-4 w-4 animate-spin" /> : <Sparkles className="h-4 w-4" />}
            {pack ? "刷新研究地图" : "生成研究地图"}
          </button>
        </div>
        {error && <p className="mt-3 text-sm text-red-600">{error}</p>}
      </div>

      {pack ? (
        <div className="grid min-h-[520px] lg:grid-cols-[240px_minmax(0,1fr)]">
          <nav
            className="flex gap-2 overflow-x-auto border-b border-slate-200 bg-slate-50/70 px-3 py-3 lg:block lg:space-y-1 lg:overflow-visible lg:border-b-0 lg:border-r lg:px-3"
            aria-label="研究地图导航"
          >
            {tabs.map((tab, index) => (
              <button
                key={tab.key}
                onClick={() => setActiveTab(tab.key)}
                className={cn(
                  "grid shrink-0 grid-cols-[24px_minmax(0,1fr)] items-start gap-2 rounded-md px-3 py-2.5 text-left text-sm transition lg:w-full",
                  activeTab === tab.key
                    ? "bg-white text-slate-950 shadow-sm ring-1 ring-slate-200"
                    : "text-slate-500 hover:bg-white hover:text-slate-900",
                )}
              >
                <span
                  className={cn(
                    "mt-0.5 flex h-5 w-5 items-center justify-center rounded border text-[10px] font-semibold",
                    activeTab === tab.key
                      ? "border-sky-200 bg-sky-50 text-sky-700"
                      : "border-slate-200 bg-white text-slate-400",
                  )}
                >
                  {index + 1}
                </span>
                <span className="min-w-0">
                  <span className="flex items-center gap-1.5 font-semibold">
                    {tab.icon}
                    {tab.label}
                  </span>
                  <span className="mt-0.5 hidden text-xs leading-5 text-slate-400 lg:block">
                    {tab.description}
                  </span>
                </span>
              </button>
            ))}
          </nav>

          <div className="min-w-0 p-5 lg:p-6">
            <TabIntro tab={activeTab} />
            {renderTab(activeTab, pack)}
          </div>
        </div>
      ) : (
        <div className="grid gap-5 px-5 py-6 md:grid-cols-[minmax(0,1fr)_220px]">
          <div>
            <h3 className="text-lg font-semibold text-slate-950">
              先生成一份可复用的研究地图
            </h3>
            <p className="mt-2 max-w-2xl text-sm leading-6 text-slate-600">
              它会把论文拆成启发、结构、前提知识、文献脉络、前后继研究和中文翻译，让后续阅读更有顺序。
            </p>
            <div className="mt-4 grid gap-2 text-sm text-slate-600 sm:grid-cols-2">
              {tabs.slice(0, 6).map((tab) => (
                <div key={tab.key} className="flex items-center gap-2">
                  <span className="text-sky-600">{tab.icon}</span>
                  <span>{tab.label}</span>
                </div>
              ))}
            </div>
          </div>
          <div className="rounded-lg border border-slate-200 bg-slate-50 px-4 py-3 text-sm leading-6 text-slate-600">
            首次生成需要等待 AI 分析；之后同一论文会优先使用缓存结果。
          </div>
        </div>
      )}
    </section>
  );
}

function TabIntro({ tab }: { tab: TabKey }) {
  const copy = tabCopy[tab];
  return (
    <header className="mb-5 border-b border-slate-200 pb-4">
      <p className="mb-1 text-xs font-semibold uppercase text-sky-700">
        {copy.eyebrow}
      </p>
      <h3 className="text-xl font-semibold leading-snug text-slate-950">
        {copy.title}
      </h3>
      <p className="mt-2 max-w-3xl text-sm leading-6 text-slate-600">
        {copy.description}
      </p>
    </header>
  );
}

function renderTab(tab: TabKey, pack: StudyPack) {
  switch (tab) {
    case "inspiration":
      return <InsightGrid items={pack.inspiration} />;
    case "structure":
      return <StructureList items={pack.structure_logic} />;
    case "prerequisites":
      return <PrerequisiteList items={pack.prerequisites} />;
    case "directions":
      return <DirectionList items={pack.research_directions} />;
    case "review":
      return <ReviewTimeline items={pack.literature_review} />;
    case "lineage":
      return <Lineage pack={pack} />;
    case "translation":
      return <Translation pack={pack} />;
  }
}

function InsightGrid({ items }: { items: InsightItem[] }) {
  return (
    <div className="grid gap-3 md:grid-cols-2">
      {items.map((item, index) => (
        <InfoCard key={index} index={index + 1} title={item.title}>
          <p>{item.explanation}</p>
          <p className="mt-3 border-l-2 border-sky-300 pl-3 text-sky-700">
            {item.how_to_apply}
          </p>
        </InfoCard>
      ))}
    </div>
  );
}

function StructureList({ items }: { items: StructureMove[] }) {
  return (
    <div className="space-y-3">
      {items.map((item, index) => (
        <InfoCard key={index} index={index + 1} title={item.step}>
          <p>{item.purpose}</p>
          <p className="mt-2">{item.why_it_works}</p>
          <p className="mt-3 border-l-2 border-sky-300 pl-3 text-sky-700">
            {item.writing_takeaway}
          </p>
        </InfoCard>
      ))}
    </div>
  );
}

function PrerequisiteList({ items }: { items: Prerequisite[] }) {
  return (
    <div className="space-y-3">
      {items.map((item, index) => (
        <InfoCard key={index} index={index + 1} title={item.topic}>
          <p>{item.why_needed}</p>
          <p className="mt-2 text-slate-500">最低目标：{item.minimum_goal}</p>
          <ReferenceList references={item.references} />
        </InfoCard>
      ))}
    </div>
  );
}

function DirectionList({ items }: { items: ResearchDirection[] }) {
  return (
    <div className="grid gap-3 md:grid-cols-2">
      {items.map((item, index) => (
        <InfoCard key={index} index={index + 1} title={item.question}>
          <p>{item.motivation}</p>
          <p className="mt-2">方法：{item.possible_method}</p>
          <p className="mt-3 border-l-2 border-sky-300 pl-3 text-sky-700">
            第一步：{item.first_step}
          </p>
        </InfoCard>
      ))}
    </div>
  );
}

function ReviewTimeline({ items }: { items: LineageItem[] }) {
  return (
    <div className="space-y-4">
      {items.map((item, index) => (
        <div key={index} className="grid gap-3 md:grid-cols-[120px_minmax(0,1fr)]">
          <div className="font-mono text-sm font-semibold text-sky-700">{item.stage}</div>
          <InfoCard index={index + 1} title={item.idea}>
            <ReferenceList references={item.representative_work} />
          </InfoCard>
        </div>
      ))}
    </div>
  );
}

function Lineage({ pack }: { pack: StudyPack }) {
  return (
    <div className="grid gap-4 md:grid-cols-2">
      <InfoCard title="本论文基于哪些研究">
        <ReferenceList references={pack.lineage.builds_on} />
      </InfoCard>
      <InfoCard title="后续继承性研究">
        <ReferenceList references={pack.lineage.follow_ups} />
      </InfoCard>
      <InfoCard title="继续检索 Query">
        <ul className="space-y-2">
          {pack.lineage.search_queries.map((query, index) => (
            <li key={index} className="rounded-md bg-slate-50 px-3 py-2 font-mono text-xs text-slate-600">
              {query}
            </li>
          ))}
        </ul>
      </InfoCard>
    </div>
  );
}

function Translation({ pack }: { pack: StudyPack }) {
  const translation = pack.translation;
  if (!translation || translation.sections.length === 0) {
    return <p className="text-sm text-slate-500">暂无翻译摘要。</p>;
  }
  const glossary = translation.glossary?.filter((item) => item.term || item.translation) ?? [];
  return (
    <div className="space-y-5">
      <div className="flex flex-col gap-3 rounded-lg border border-slate-200 bg-white px-4 py-3 md:flex-row md:items-center md:justify-between">
        <div>
          <p className="text-sm font-semibold text-slate-950">中英对照阅读</p>
          <p className="mt-1 text-sm leading-6 text-slate-500">
            {translation.source_language} → {translation.target_language}，保留关键英文表达，便于边读边学习论文写法。
          </p>
        </div>
        <span className="rounded-md bg-slate-100 px-2.5 py-1 font-mono text-xs font-semibold text-slate-500">
          bilingual
        </span>
      </div>

      {translation.sections.map((section, index) => (
        <article
          key={index}
          className="rounded-lg border border-slate-200 bg-white p-4 text-sm leading-6 text-slate-700"
        >
          <div className="mb-4 flex items-start gap-3">
            <span className="mt-0.5 flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-slate-100 font-mono text-xs font-semibold text-slate-500">
              {index + 1}
            </span>
            <div>
              <h3 className="text-base font-semibold leading-6 text-slate-950">
                {section.heading || `Section ${index + 1}`}
              </h3>
              <p className="mt-1 text-xs text-slate-400">原文摘录 + 中文理解</p>
            </div>
          </div>

          <div className="grid gap-3 lg:grid-cols-2">
            <div className="rounded-lg border border-slate-200 bg-slate-50 px-4 py-3">
              <div className="mb-2 text-xs font-semibold uppercase text-slate-400">
                English Expression
              </div>
              <p className="whitespace-pre-line text-sm leading-7 text-slate-700">
                {section.original_excerpt || "旧版缓存暂无英文摘录。刷新研究地图后会生成英文对照。"}
              </p>
            </div>
            <div className="rounded-lg border border-sky-100 bg-sky-50/60 px-4 py-3">
              <div className="mb-2 text-xs font-semibold uppercase text-sky-700">
                中文理解
              </div>
              <p className="whitespace-pre-line text-sm leading-7 text-slate-700">
                {section.translated_text}
              </p>
            </div>
          </div>

          {section.expression_notes && section.expression_notes.length > 0 && (
            <div className="mt-4">
              <div className="mb-2 text-xs font-semibold uppercase text-slate-400">
                专业表达学习
              </div>
              <div className="space-y-2">
                {section.expression_notes.map((note, noteIndex) => (
                  <div
                    key={`${note.english}-${noteIndex}`}
                    className="rounded-md border border-slate-200 px-3 py-2"
                  >
                    <div className="grid gap-2 md:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
                      <p className="font-medium text-slate-900">{note.english}</p>
                      <p className="text-sky-700">{note.chinese}</p>
                    </div>
                    <p className="mt-1 text-xs leading-5 text-slate-500">
                      {note.usage}
                    </p>
                  </div>
                ))}
              </div>
            </div>
          )}
        </article>
      ))}

      {glossary.length > 0 && (
        <section className="rounded-lg border border-slate-200 bg-white p-4">
          <div className="mb-3 text-sm font-semibold text-slate-950">
            术语与表达对照
          </div>
          <div className="grid gap-2 md:grid-cols-2">
            {glossary.map((item, index) => (
              <div
                key={`${item.term}-${index}`}
                className="rounded-md bg-slate-50 px-3 py-2 text-sm"
              >
                <div className="flex flex-wrap items-baseline gap-2">
                  <span className="font-semibold text-slate-900">{item.term}</span>
                  <span className="text-sky-700">{item.translation}</span>
                </div>
                {item.note && (
                  <p className="mt-1 text-xs leading-5 text-slate-500">{item.note}</p>
                )}
              </div>
            ))}
          </div>
        </section>
      )}
    </div>
  );
}

function ReferenceList({ references }: { references: StudyReference[] }) {
  if (!references || references.length === 0) {
    return <p className="mt-2 text-sm text-slate-400">暂无可靠参考线索。</p>;
  }
  return (
    <ul className="mt-3 space-y-2">
      {references.map((reference, index) => (
        <li key={index} className="rounded-md bg-slate-50 px-3 py-2 text-sm">
          {reference.url ? (
            <a
              href={reference.url}
              target="_blank"
              rel="noreferrer"
              className="font-medium text-sky-700 hover:underline"
            >
              {reference.title || "Untitled"}
            </a>
          ) : (
            <span className="font-medium text-slate-800">
              {reference.title || "Untitled"}
            </span>
          )}
          <p className="mt-1 text-xs leading-5 text-slate-500">
            {reference.year ? `${reference.year} · ` : ""}
            {reference.relevance}
          </p>
        </li>
      ))}
    </ul>
  );
}

function InfoCard({
  title,
  index,
  children,
}: {
  title: string;
  index?: number;
  children: React.ReactNode;
}) {
  return (
    <article className="rounded-lg border border-slate-200 bg-white p-4 text-sm leading-6 text-slate-700">
      <div className="mb-2 flex items-start gap-3">
        {index !== undefined && (
          <span className="mt-0.5 flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-slate-100 font-mono text-xs font-semibold text-slate-500">
            {index}
          </span>
        )}
        <h3 className="text-base font-semibold leading-6 text-slate-950">
          {title || "未命名条目"}
        </h3>
      </div>
      <div className={index !== undefined ? "sm:pl-9" : undefined}>{children}</div>
    </article>
  );
}
