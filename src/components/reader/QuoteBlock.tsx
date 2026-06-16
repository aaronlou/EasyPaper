import type { QuoteBlock as QuoteBlockType } from "@/types";

export default function QuoteBlock({ block }: { block: QuoteBlockType }) {
  return (
    <blockquote className="border-l-3 border-blue-400 bg-blue-50/50 pl-5 py-3 pr-4 rounded-r-lg my-4">
      <p className="text-gray-700 italic leading-relaxed">"{block.text}"</p>
      {block.cite && (
        <cite className="block mt-2 text-xs text-gray-400 not-italic">
          — {block.cite}
        </cite>
      )}
    </blockquote>
  );
}
