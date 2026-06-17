import { useMemo, useState } from "react";
import { Network, Sparkles } from "lucide-react";
import type { Concept } from "@/types";
import { cn } from "@/lib/cn";

interface Props {
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
};

export default function ConceptGraphBlock({ concepts, onSelectConcept }: Props) {
  const [activeId, setActiveId] = useState(concepts[0]?.id ?? "");
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

  if (concepts.length === 0 || !active) return null;

  return (
    <section className="reader-panel my-8 overflow-hidden p-0">
      <div className="grid gap-0 lg:grid-cols-[minmax(0,1fr)_320px]">
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

        <aside className="flex min-h-[360px] flex-col bg-white p-5">
          <div className="mb-3 flex items-center justify-between gap-3">
            <span
              className={cn(
                "rounded-full border px-2.5 py-1 text-xs font-semibold",
                difficultyStyles[active.difficulty] ?? "border-slate-200 bg-slate-50 text-slate-600",
              )}
            >
              {difficultyLabel[active.difficulty] ?? active.difficulty}
            </span>
            <span className="text-xs text-slate-400">{concepts.length} 个概念</span>
          </div>
          <h2 className="text-xl font-semibold leading-snug text-slate-950">
            {active.term}
          </h2>
          <p className="mt-3 text-sm leading-6 text-slate-600">{active.definition}</p>

          {active.related.length > 0 && (
            <div className="mt-5">
              <div className="mb-2 text-xs font-semibold uppercase text-slate-400">
                关联概念
              </div>
              <div className="flex flex-wrap gap-2">
                {active.related.map((id) => {
                  const related = concepts.find((concept) => concept.id === id);
                  if (!related) return null;
                  return (
                    <button
                      key={id}
                      onClick={() => setActiveId(id)}
                      className="rounded-full border border-sky-100 bg-sky-50 px-3 py-1 text-xs font-medium text-sky-700 transition hover:border-sky-300 hover:bg-sky-100"
                    >
                      {related.term}
                    </button>
                  );
                })}
              </div>
            </div>
          )}

          <button
            onClick={() => onSelectConcept(active.id)}
            className="mt-auto inline-flex items-center justify-center gap-2 rounded-lg bg-slate-950 px-4 py-2.5 text-sm font-semibold text-white transition hover:bg-sky-700"
          >
            <Sparkles className="h-4 w-4" />
            深入研究
          </button>
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
