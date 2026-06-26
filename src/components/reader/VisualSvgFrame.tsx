import { useMemo, useState } from "react";
import { ArrowRight, CheckCircle2, Maximize2, Minimize2 } from "lucide-react";
import { cn } from "@/lib/cn";
import { sanitizeSvg } from "@/lib/sanitizeSvg";

interface Props {
  svg: string;
  caption?: string;
  label: string;
}

interface ExtractedMechanismStep {
  index: string;
  title: string;
  input?: string;
  process?: string;
  output?: string;
}

interface ExtractedMechanismDiagram {
  title: string;
  note?: string;
  steps: ExtractedMechanismStep[];
}

export default function VisualSvgFrame({ svg, caption, label }: Props) {
  const [expanded, setExpanded] = useState(false);
  const safeSvg = useMemo(() => sanitizeSvg(svg), [svg]);
  const extractedMechanism = useMemo(
    () => (safeSvg ? extractMechanismDiagram(safeSvg) : null),
    [safeSvg],
  );

  return (
    <figure
      className={cn(
        "reader-panel my-6 overflow-hidden p-0 transition-all duration-200",
        expanded && "fixed inset-4 z-50 my-0 flex flex-col bg-white shadow-2xl",
      )}
    >
      <div className="flex items-center justify-between gap-3 border-b border-slate-200 bg-slate-50 px-4 py-3">
        <div className="min-w-0">
          <div className="text-xs font-semibold uppercase text-sky-600">{label}</div>
          {caption && (
            <figcaption className="truncate text-sm font-medium text-slate-700">
              {caption}
            </figcaption>
          )}
        </div>
        <button
          type="button"
          onClick={() => setExpanded((value) => !value)}
          className="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-md border border-slate-200 bg-white text-slate-500 transition hover:border-sky-300 hover:text-sky-700"
          aria-label={expanded ? "还原图示" : "放大图示"}
          title={expanded ? "还原图示" : "放大图示"}
        >
          {expanded ? <Minimize2 className="h-4 w-4" /> : <Maximize2 className="h-4 w-4" />}
        </button>
      </div>

      {extractedMechanism ? (
        <LegacyMechanismDiagram diagram={extractedMechanism} expanded={expanded} />
      ) : safeSvg ? (
        <div
          className={cn(
            "visual-svg-frame flex justify-center overflow-auto bg-white p-4",
            expanded ? "min-h-0 flex-1 items-center" : "max-h-[520px]",
          )}
          dangerouslySetInnerHTML={{ __html: safeSvg }}
        />
      ) : (
        <div className="bg-white px-4 py-6 text-sm text-slate-500">
          图示格式无效，已跳过渲染。
        </div>
      )}
    </figure>
  );
}

function LegacyMechanismDiagram({
  diagram,
  expanded,
}: {
  diagram: ExtractedMechanismDiagram;
  expanded: boolean;
}) {
  const [activeIndex, setActiveIndex] = useState(0);
  const active = diagram.steps[Math.min(activeIndex, diagram.steps.length - 1)];

  return (
    <div
      className={cn(
        "bg-white p-4",
        expanded ? "min-h-0 flex-1 overflow-auto" : "max-h-[640px] overflow-auto",
      )}
    >
      <div className="rounded-lg border border-slate-200 bg-slate-50 p-4">
        <h3 className="text-base font-semibold leading-6 text-slate-950">
          {diagram.title}
        </h3>
        <div className="mt-4 grid gap-4 lg:grid-cols-[minmax(0,0.9fr)_minmax(300px,0.72fr)]">
          <div className="flex gap-3 overflow-x-auto pb-2 lg:grid lg:grid-cols-1 lg:overflow-visible lg:pb-0">
            {diagram.steps.map((step, index) => {
              const selected = index === activeIndex;
              return (
                <button
                  key={`${step.index}-${step.title}-${index}`}
                  type="button"
                  onClick={() => setActiveIndex(index)}
                  className={cn(
                    "grid min-w-[250px] grid-cols-[32px_minmax(0,1fr)] gap-3 rounded-lg border bg-white p-3 text-left transition lg:min-w-0",
                    selected
                      ? "border-sky-300 shadow-sm"
                      : "border-slate-200 hover:border-sky-200",
                  )}
                >
                  <span
                    className={cn(
                      "flex h-8 w-8 items-center justify-center rounded-full text-sm font-semibold",
                      selected ? "bg-sky-600 text-white" : "bg-slate-100 text-slate-500",
                    )}
                  >
                    {step.index}
                  </span>
                  <span className="min-w-0">
                    <span className="block text-sm font-semibold leading-5 text-slate-900">
                      {step.title}
                    </span>
                    <span className="mt-1 line-clamp-2 block text-xs leading-5 text-slate-500">
                      {step.output || step.process || step.input}
                    </span>
                  </span>
                </button>
              );
            })}
          </div>

          {active && (
            <div className="rounded-lg border border-slate-200 bg-white p-4">
              <div className="mb-3 text-xs font-medium text-slate-400">
                Step {active.index} / {diagram.steps.length}
              </div>
              <h4 className="text-lg font-semibold leading-7 text-slate-950">
                {active.title}
              </h4>
              <div className="mt-4 space-y-3">
                <LegacyDetail label="输入" value={active.input} icon="arrow" />
                <LegacyDetail label="处理" value={active.process} icon="process" />
                <LegacyDetail label="输出" value={active.output} icon="check" />
              </div>
            </div>
          )}
        </div>
        {diagram.note && (
          <p className="mt-4 text-sm leading-7 text-slate-500">{diagram.note}</p>
        )}
      </div>
    </div>
  );
}

function LegacyDetail({
  label,
  value,
  icon,
}: {
  label: string;
  value?: string;
  icon: "arrow" | "process" | "check";
}) {
  if (!value) return null;
  return (
    <div className="grid gap-2 sm:grid-cols-[64px_minmax(0,1fr)]">
      <span className="inline-flex h-7 w-fit items-center gap-1.5 rounded-md border border-slate-200 bg-slate-50 px-2 text-xs font-semibold text-slate-600">
        {icon === "check" ? (
          <CheckCircle2 className="h-3.5 w-3.5 text-emerald-600" />
        ) : (
          <ArrowRight className="h-3.5 w-3.5 text-sky-600" />
        )}
        {label}
      </span>
      <p className="min-w-0 text-sm leading-7 text-slate-700">{value}</p>
    </div>
  );
}

function extractMechanismDiagram(svg: string): ExtractedMechanismDiagram | null {
  if (typeof window === "undefined") return null;

  const doc = new DOMParser().parseFromString(svg, "image/svg+xml");
  const root = doc.querySelector("svg");
  if (!root) return null;

  const titleText = textContent(root.querySelector(":scope > text"));
  const looksLikeGeneratedMechanism =
    root.getAttribute("aria-label")?.includes("机制") ||
    titleText.includes("mechanism") ||
    titleText.includes("Feynman Path");
  if (!looksLikeGeneratedMechanism) return null;

  const topLevelTexts = Array.from(root.querySelectorAll(":scope > text")).map(textContent);
  const note = topLevelTexts
    .slice(1)
    .find((text) => text.includes("读法") || text.includes("输入"));

  const steps: ExtractedMechanismStep[] = Array.from(root.querySelectorAll("g"))
    .map<ExtractedMechanismStep | null>((group) => {
      const texts = Array.from(group.querySelectorAll("text"))
        .map(textContent)
        .filter(Boolean);
      if (texts.length < 3 || !/^\d+$/.test(texts[0])) return null;
      const [, title, ...details] = texts;
      return {
        index: texts[0],
        title,
        input: stripDetailLabel(details.find((text) => /^输入[:：]/.test(text))),
        process: stripDetailLabel(details.find((text) => /^处理[:：]/.test(text))),
        output: stripDetailLabel(details.find((text) => /^输出[:：]/.test(text))),
      };
    })
    .filter((step): step is ExtractedMechanismStep => step !== null);

  if (steps.length < 2) return null;

  return {
    title: titleText || "机制链路",
    note,
    steps,
  };
}

function textContent(element: Element | null) {
  return element?.textContent?.trim() ?? "";
}

function stripDetailLabel(value?: string) {
  return value?.replace(/^(输入|处理|输出)[:：]\s*/, "").trim();
}
