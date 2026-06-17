import { BrainCircuit } from "lucide-react";
import { useReaderContext } from "@/contexts/ReaderContext";
import type { ConceptCardBlock as ConceptCardBlockType } from "@/types";
import { cn } from "@/lib/cn";

interface Props {
  block: ConceptCardBlockType;
}

export default function ConceptCardBlock({ block }: Props) {
  const { interpretation, setActiveConceptId } = useReaderContext();

  // 优先匹配 interpretation.concepts 中的 id，便于深潜接口查找
  const matchedConcept = interpretation.concepts.find(
    (c) => c.term === block.term || c.id === block.id,
  );
  const conceptId = matchedConcept?.id ?? block.id;

  return (
    <div
      onClick={() => setActiveConceptId(conceptId)}
      className={cn(
        "group relative border rounded-xl p-4 cursor-pointer transition-all duration-200 my-3",
        "border-gray-200 bg-white hover:border-blue-300 hover:bg-blue-50/20 hover:shadow-sm",
      )}
    >
      <div className="flex items-start gap-3">
        <span className="text-xl shrink-0 mt-0.5">{block.icon ?? "💡"}</span>
        <div className="flex-1 min-w-0">
          <div className="font-semibold text-gray-800 group-hover:text-blue-700 transition-colors">
            {block.term}
          </div>
          <p className="mt-1 text-sm text-gray-500 line-clamp-2">
            {block.definition}
          </p>
          <div className="mt-2 flex items-center gap-1 text-xs text-blue-500 font-medium opacity-0 group-hover:opacity-100 transition-opacity">
            <BrainCircuit className="w-3 h-3" />
            <span>打开概念实验室</span>
          </div>
        </div>
      </div>
    </div>
  );
}
