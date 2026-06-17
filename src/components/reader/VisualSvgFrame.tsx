import { useState } from "react";
import { Maximize2, Minimize2 } from "lucide-react";
import { cn } from "@/lib/cn";

interface Props {
  svg: string;
  caption?: string;
  label: string;
}

export default function VisualSvgFrame({ svg, caption, label }: Props) {
  const [expanded, setExpanded] = useState(false);

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

      <div
        className={cn(
          "visual-svg-frame flex justify-center overflow-auto bg-white p-4",
          expanded ? "min-h-0 flex-1 items-center" : "max-h-[520px]",
        )}
        dangerouslySetInnerHTML={{ __html: svg }}
      />
    </figure>
  );
}
