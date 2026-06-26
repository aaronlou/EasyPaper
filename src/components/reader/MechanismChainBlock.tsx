import { useMemo, useState } from "react";
import { ArrowRight, Braces, CheckCircle2, Cpu, FileSearch } from "lucide-react";
import type { MechanismChainBlock as MechanismChainBlockType } from "@/types";
import { cn } from "@/lib/cn";

interface Props {
  block: MechanismChainBlockType;
}

const detailLabels = [
  { key: "input", label: "输入", tone: "text-sky-700 bg-sky-50 border-sky-100" },
  { key: "process", label: "处理", tone: "text-teal-700 bg-teal-50 border-teal-100" },
  { key: "output", label: "输出", tone: "text-amber-700 bg-amber-50 border-amber-100" },
] as const;

export default function MechanismChainBlock({ block }: Props) {
  const steps = useMemo(
    () => block.steps.filter((step) => step.title.trim()),
    [block.steps],
  );
  const [activeIndex, setActiveIndex] = useState(0);
  const active = steps[Math.min(activeIndex, steps.length - 1)];

  if (!active || steps.length === 0) return null;

  return (
    <section className="reader-panel my-6 overflow-hidden p-0">
      <div className="border-b border-slate-200 bg-slate-50 px-4 py-3">
        <div className="text-xs font-semibold uppercase text-sky-600">
          机制链路
        </div>
        <h3 className="mt-1 text-base font-semibold leading-6 text-slate-900">
          {block.title ?? "从输入到输出的论文机制"}
        </h3>
      </div>

      <div className="grid gap-0 xl:grid-cols-[minmax(0,0.95fr)_minmax(340px,0.72fr)]">
        <div className="border-b border-slate-200 bg-white p-4 xl:border-b-0 xl:border-r">
          <div className="flex gap-3 overflow-x-auto pb-2 xl:grid xl:grid-cols-1 xl:overflow-visible xl:pb-0">
            {steps.map((step, index) => {
              const selected = index === activeIndex;
              return (
                <button
                  key={`${step.title}-${index}`}
                  type="button"
                  onClick={() => setActiveIndex(index)}
                  className={cn(
                    "group grid min-w-[260px] grid-cols-[34px_minmax(0,1fr)] gap-3 rounded-lg border p-3 text-left transition xl:min-w-0",
                    selected
                      ? "border-sky-300 bg-sky-50 shadow-sm"
                      : "border-slate-200 bg-white hover:border-sky-200 hover:bg-slate-50",
                  )}
                >
                  <span
                    className={cn(
                      "flex h-8 w-8 items-center justify-center rounded-full text-sm font-semibold",
                      selected
                        ? "bg-sky-600 text-white"
                        : "bg-slate-100 text-slate-500 group-hover:bg-sky-100 group-hover:text-sky-700",
                    )}
                  >
                    {index + 1}
                  </span>
                  <span className="min-w-0">
                    <span className="block text-sm font-semibold leading-5 text-slate-900">
                      {step.title}
                    </span>
                    <span className="mt-1 line-clamp-2 block text-xs leading-5 text-slate-500">
                      {step.why_it_matters || step.output}
                    </span>
                  </span>
                </button>
              );
            })}
          </div>
        </div>

        <aside className="bg-slate-50/70 p-4">
          <div className="rounded-lg border border-slate-200 bg-white p-4 shadow-sm">
            <div className="mb-4 flex items-start gap-3">
              <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-slate-900 text-white">
                <Cpu className="h-4 w-4" />
              </div>
              <div className="min-w-0">
                <div className="text-xs font-medium text-slate-400">
                  Step {activeIndex + 1} / {steps.length}
                </div>
                <h4 className="mt-1 text-lg font-semibold leading-7 text-slate-950">
                  {active.title}
                </h4>
              </div>
            </div>

            <div className="space-y-3">
              {detailLabels.map(({ key, label, tone }, index) => (
                <div key={key} className="grid gap-2 sm:grid-cols-[72px_minmax(0,1fr)]">
                  <div
                    className={cn(
                      "inline-flex h-7 w-fit items-center gap-1.5 rounded-md border px-2 text-xs font-semibold",
                      tone,
                    )}
                  >
                    {index === 1 ? (
                      <Braces className="h-3.5 w-3.5" />
                    ) : index === 2 ? (
                      <CheckCircle2 className="h-3.5 w-3.5" />
                    ) : (
                      <ArrowRight className="h-3.5 w-3.5" />
                    )}
                    {label}
                  </div>
                  <p className="min-w-0 text-sm leading-7 text-slate-700">
                    {active[key] || "这一步没有给出明确说明。"}
                  </p>
                </div>
              ))}
            </div>

            <div className="mt-4 rounded-lg border border-slate-200 bg-slate-50 p-3">
              <div className="mb-1 text-xs font-semibold uppercase text-slate-400">
                为什么关键
              </div>
              <p className="text-sm leading-7 text-slate-700">
                {active.why_it_matters || "它连接了上一步的产物和下一步的判断，是读者复述机制时不能跳过的环节。"}
              </p>
            </div>

            {active.evidence_anchor && (
              <div className="mt-3 flex gap-2 rounded-lg border border-amber-100 bg-amber-50 px-3 py-2 text-sm leading-6 text-amber-800">
                <FileSearch className="mt-0.5 h-4 w-4 shrink-0" />
                <span>{active.evidence_anchor}</span>
              </div>
            )}
          </div>

          {block.note && (
            <p className="mt-3 text-sm leading-7 text-slate-500">
              {block.note}
            </p>
          )}
        </aside>
      </div>
    </section>
  );
}
