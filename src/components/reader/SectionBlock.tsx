import type { SectionBlock as SectionBlockType } from "@/types";

export default function SectionBlock({ block }: { block: SectionBlockType }) {
  return (
    <h2 className="text-2xl font-bold text-gray-900 mt-10 mb-4 flex items-baseline gap-3">
      <span className="text-blue-600 text-base font-mono font-normal shrink-0">
        §{block.num}
      </span>
      <span>{block.title}</span>
    </h2>
  );
}
