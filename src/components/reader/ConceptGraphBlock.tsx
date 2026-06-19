import { useEffect, useMemo, useState } from "react";
import {
  ArrowRight,
  BrainCircuit,
  Lightbulb,
  Loader2,
  Network,
  Sparkles,
} from "lucide-react";
import * as api from "@/lib/api";
import type { Concept, ConceptExpansion } from "@/types";
import { cn } from "@/lib/cn";

interface Props {
  paperId: string;
  concepts: Concept[];
  onSelectConcept: (conceptId: string) => void;
}

const difficultyStyles: Record<string, string> = {
  basic: "border-emerald-200 bg-emerald-50 text-emerald-700",
  intermediate: "border-amber-200 bg-amber-50 text-amber-700",
  advanced: "border-rose-200 bg-rose-50 text-rose-700",
};

const difficultyLabel: Record<string, string> = {
  basic: "基础",
  intermediate: "进阶",
  advanced: "高阶",
} as const;

export default function ConceptGraphBlock({
  paperId,
  concepts,
  onSelectConcept,
}: Props) {
  const [activeId, setActiveId] = useState(concepts[0]?.id ?? "");
  const [preview, setPreview] = useState<ConceptExpansion | null>(null);
  const [previewLoading, setPreviewLoading] = useState(false);
  const [previewError, setPreviewError] = useState<string | null>(null);
  const active = concepts.find((concept) => concept.id === activeId) ?? concepts[0];

  const nodes = useMemo(() => layoutConceptNodes(concepts), [concepts]);
  const nodeById = useMemo(
    () => new Map(nodes.map((node) => [node.concept.id, node])),
    [nodes],
  );
  const edges = useMemo(() => {
    const lines: { from: string; to: string }[] = [];
    for (const concept of concepts) {
      for (const relatedId of concept.related) {
        if (nodeById.has(relatedId)) {
          lines.push({ from: concept.id, to: relatedId });
        }
      }
    }
    return lines.slice(0, 18);
  }, [concepts, nodeById]);

  useEffect(() => {
    if (!activeId) return;
    let cancelled = false;
    setPreviewLoading(true);
    setPreview(null);
    setPreviewError(null);
    api
      .expandConcept(paperId, activeId)
      .then((result) => {
        if (!cancelled) setPreview(result);
      })
      .catch((error) => {
        if (!cancelled) {
          setPreview(null);
          setPreviewError(error instanceof Error ? error.message : "概念深潜生成失败");
        }
      })
      .finally(() => {
        if (!cancelled) setPreviewLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [paperId, activeId]);

  if (concepts.length === 0 || !active) return null;

  const relatedConcepts = active.related
    .map((id) => concepts.find((concept) => concept.id === id))
    .filter((concept): concept is Concept => Boolean(concept));
  const firstStep = preview?.mechanism_steps?.[0];
  const misconceptions =
    preview?.common_misconceptions ||
    "真正理解这个概念，不是记住定义，而是能说清它在论文里改变了哪个判断、机制或实验结果。";
  const takeaways = preview?.key_takeaways?.filter(Boolean).slice(0, 3) ?? [];

  return (
    <section className="reader-panel overflow-hidden p-0">
      <div className="grid gap-0 xl:grid-cols-[minmax(0,0.95fr)_minmax(360px,0.8fr)]">
        <div className="relative min-h-[360px] border-b border-slate-200 bg-[#f7fbff] lg:border-b-0 lg:border-r">
          <div className="absolute left-5 top-5 z-10 flex items-center gap-2 rounded-full border border-slate-200 bg-white/90 px-3 py-1.5 text-xs font-semibold text-slate-700 shadow-sm backdrop-blur">
            <Network className="h-3.5 w-3.5 text-sky-600" />
            概念地图
          </div>
          <svg
            viewBox="0 0 720 360"
            className="h-full min-h-[360px] w-full"
            role="img"
            aria-label="论文关键概念关系图"
          >
            <defs>
              <pattern
                id="concept-grid"
                width="28"
                height="28"
                patternUnits="userSpaceOnUse"
              >
                <path d="M 28 0 L 0 0 0 28" fill="none" stroke="#e2e8f0" strokeWidth="1" />
              </pattern>
            </defs>
            <rect width="720" height="360" fill="url(#concept-grid)" opacity="0.72" />
            {edges.map((edge, index) => {
              const from = nodeById.get(edge.from);
              const to = nodeById.get(edge.to);
              if (!from || !to) return null;
              const hot = active.id === edge.from || active.id === edge.to;
              return (
                <line
                  key={`${edge.from}-${edge.to}-${index}`}
                  x1={from.x}
                  y1={from.y}
                  x2={to.x}
                  y2={to.y}
                  stroke={hot ? "#0284c7" : "#cbd5e1"}
                  strokeWidth={hot ? 2.5 : 1.5}
                  strokeLinecap="round"
                  opacity={hot ? 0.9 : 0.55}
                />
              );
            })}
            {nodes.map((node, index) => {
              const selected = node.concept.id === active.id;
              const related = active.related.includes(node.concept.id);
              const radius = selected ? 34 : related ? 28 : 24;
              return (
                <g
                  key={node.concept.id}
                  className="cursor-pointer"
                  onClick={() => setActiveId(node.concept.id)}
                >
                  <circle
                    cx={node.x}
                    cy={node.y}
                    r={radius}
                    fill={selected ? "#0f172a" : related ? "#e0f2fe" : "#ffffff"}
                    stroke={selected ? "#0f172a" : related ? "#0284c7" : "#94a3b8"}
                    strokeWidth={selected ? 3 : 1.5}
                    className="transition-all duration-200"
                  />
                  <text
                    x={node.x}
                    y={node.y + 4}
                    textAnchor="middle"
                    fontSize={selected ? 12 : 10}
                    fontWeight={selected ? 700 : 600}
                    fill={selected ? "#ffffff" : "#334155"}
                  >
                    {shortLabel(node.concept.term, index)}
                  </text>
                </g>
              );
            })}
          </svg>
        </div>

        <aside className="flex min-h-[520px] flex-col bg-white">
          <div className="border-b border-slate-200 p-5">
            <div className="mb-3 flex items-center justify-between gap-3">
              <span
                className={cn(
                  "rounded-md border px-2.5 py-1 text-xs font-semibold",
                  difficultyStyles[active.difficulty] ?? "border-slate-200 bg-slate-50 text-slate-600",
                )}
              >
                {difficultyLabel[active.difficulty] ?? active.difficulty}
              </span>
              <span className="text-xs text-slate-400">{concepts.length} 个概念</span>
            </div>

            <h2 className="text-2xl font-semibold leading-snug text-slate-950">
              {active.term}
            </h2>
            <p className="mt-3 text-sm leading-7 text-slate-600">
              {preview?.expanded_definition || active.definition}
            </p>
          </div>

          <div className="flex-1 space-y-4 p-5">
            <section className="rounded-lg border border-sky-100 bg-sky-50/70 p-4">
              <div className="mb-2 flex items-center gap-2 text-sm font-semibold text-sky-800">
                <Lightbulb className="h-4 w-4" />
                费曼式直觉
              </div>
              {previewLoading ? (
                <LoadingLine label="正在生成更深入的直觉解释..." />
              ) : (
                <p className="text-sm leading-7 text-slate-700">
                  {preview?.intuition || buildFallbackIntuition(active)}
                </p>
              )}
            </section>

            <section className="grid gap-3 md:grid-cols-2 xl:grid-cols-1 2xl:grid-cols-2">
              <ConceptBriefPanel title="在本文中作用">
                {preview?.in_this_paper ||
                  "点击后会把概念放回论文语境，说明它参与了哪个方法、实验或论证。"}
              </ConceptBriefPanel>
              <ConceptBriefPanel title="常见误区">
                {misconceptions}
              </ConceptBriefPanel>
            </section>

            {firstStep && (
              <section className="rounded-lg border border-slate-200 p-4">
                <div className="mb-3 flex items-center gap-2 text-sm font-semibold text-slate-900">
                  <BrainCircuit className="h-4 w-4 text-sky-600" />
                  机制第一步：{firstStep.title || "先看输入如何被改变"}
                </div>
                <div className="grid gap-2 text-sm text-slate-600 sm:grid-cols-[1fr_24px_1fr_24px_1fr]">
                  <MechanismMiniCell label="输入" value={firstStep.input} />
                  <ArrowRight className="mx-auto hidden h-4 w-4 self-center text-slate-300 sm:block" />
                  <MechanismMiniCell label="处理" value={firstStep.process} />
                  <ArrowRight className="mx-auto hidden h-4 w-4 self-center text-slate-300 sm:block" />
                  <MechanismMiniCell label="输出" value={firstStep.output} />
                </div>
              </section>
            )}

            {takeaways.length > 0 && (
              <section>
                <div className="mb-2 text-xs font-semibold uppercase text-slate-400">
                  可复述要点
                </div>
                <ul className="space-y-2">
                  {takeaways.map((item, index) => (
                    <li
                      key={`${item}-${index}`}
                      className="grid grid-cols-[24px_minmax(0,1fr)] gap-2 text-sm leading-6 text-slate-700"
                    >
                      <span className="font-mono text-xs font-semibold text-sky-600">
                        {index + 1}
                      </span>
                      <span>{item}</span>
                    </li>
                  ))}
                </ul>
              </section>
            )}

            {relatedConcepts.length > 0 && (
              <section>
                <div className="mb-2 text-xs font-semibold uppercase text-slate-400">
                  关联概念
                </div>
                <div className="flex flex-wrap gap-2">
                  {relatedConcepts.map((related) => (
                    <button
                      key={related.id}
                      onClick={() => setActiveId(related.id)}
                      className="rounded-md border border-sky-100 bg-sky-50 px-3 py-1.5 text-xs font-medium text-sky-700 transition hover:border-sky-300 hover:bg-sky-100"
                    >
                      {related.term}
                    </button>
                  ))}
                </div>
              </section>
            )}

            {previewError && (
              <p className="rounded-lg border border-amber-200 bg-amber-50 px-3 py-2 text-xs leading-5 text-amber-800">
                {previewError}
              </p>
            )}
          </div>

          <div className="border-t border-slate-200 p-5">
            <button
              onClick={() => onSelectConcept(active.id)}
              className="inline-flex w-full items-center justify-center gap-2 rounded-lg bg-slate-950 px-4 py-2.5 text-sm font-semibold text-white transition hover:bg-sky-700"
            >
              <Sparkles className="h-4 w-4" />
              打开完整概念实验室
            </button>
          </div>
        </aside>
      </div>
    </section>
  );
}

function layoutConceptNodes(concepts: Concept[]) {
  const centerX = 360;
  const centerY = 180;
  const radiusX = 245;
  const radiusY = 118;

  return concepts.slice(0, 12).map((concept, index, list) => {
    if (index === 0) {
      return { concept, x: centerX, y: centerY };
    }
    const angle = ((index - 1) / Math.max(list.length - 1, 1)) * Math.PI * 2 - Math.PI / 2;
    const ringOffset = index % 2 === 0 ? 1 : 0.82;
    return {
      concept,
      x: centerX + Math.cos(angle) * radiusX * ringOffset,
      y: centerY + Math.sin(angle) * radiusY * ringOffset,
    };
  });
}

function shortLabel(term: string, index: number) {
  const label = term.split(/[(/（,，]/)[0]?.trim() || `C${index + 1}`;
  return label.length > 10 ? `${label.slice(0, 9)}...` : label;
}

function ConceptBriefPanel({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div className="rounded-lg border border-slate-200 p-4">
      <div className="mb-2 text-xs font-semibold uppercase text-slate-400">
        {title}
      </div>
      <p className="text-sm leading-7 text-slate-700">{children}</p>
    </div>
  );
}

function MechanismMiniCell({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-md bg-slate-50 px-3 py-2">
      <div className="mb-1 text-xs font-semibold text-slate-400">{label}</div>
      <p className="leading-6 text-slate-700">{value || "等待 AI 补充这一环节。"}</p>
    </div>
  );
}

function LoadingLine({ label }: { label: string }) {
  return (
    <div className="flex items-center gap-2 text-sm text-slate-500">
      <Loader2 className="h-4 w-4 animate-spin text-sky-600" />
      {label}
    </div>
  );
}

function buildFallbackIntuition(concept: Concept) {
  return `${concept.term} 不是一个需要死记的标签。先把它当成论文里的一件工具：它把原本零散、难判断的现象组织起来，让读者能追问输入是什么、处理发生在哪里、输出为什么改变。${concept.definition}`;
}
