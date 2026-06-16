import { useState } from "react";
import { Lightbulb } from "lucide-react";
import type { ConceptCardBlock as ConceptCardBlockType } from "@/types";
import { cn } from "@/lib/cn";

export default function ConceptCardBlock({
  block,
}: {
  block: ConceptCardBlockType;
}) {
  const [expanded, setExpanded] = useState(false);

  return (
    <div
      onClick={() => setExpanded(!expanded)}
      className={cn(
        "border rounded-xl p-4 cursor-pointer transition-all duration-200 my-3",
        expanded
          ? "border-blue-400 bg-blue-50/30 shadow-sm"
          : "border-gray-200 bg-white hover:border-blue-300 hover:bg-blue-50/20",
      )}
    >
      <div className="flex items-start gap-3">
        <span className="text-xl shrink-0 mt-0.5">{block.icon ?? "💡"}</span>
        <div className="flex-1 min-w-0">
          <div className="font-semibold text-gray-800">{block.term}</div>
          <div
            className={cn(
              "text-sm text-gray-500 transition-all duration-300 overflow-hidden",
              expanded ? "mt-2 max-h-96 opacity-100" : "max-h-0 opacity-0",
            )}
          >
            {block.definition}
          </div>
        </div>
        <Lightbulb
          className={cn(
            "w-4 h-4 shrink-0 mt-1 transition-colors",
            expanded ? "text-blue-500" : "text-gray-300",
          )}
        />
      </div>
    </div>
  );
}
