import type { ParagraphBlock as ParagraphBlockType } from "@/types";

export default function ParagraphBlock({
  block,
}: {
  block: ParagraphBlockType;
}) {
  return (
    <p className="text-gray-700 leading-relaxed whitespace-pre-line">
      {block.text}
    </p>
  );
}
