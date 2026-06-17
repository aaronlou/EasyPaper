import type { ParagraphBlock as ParagraphBlockType } from "@/types";

export default function ParagraphBlock({
  block,
}: {
  block: ParagraphBlockType;
}) {
  return (
    <p className="text-[16px] leading-8 text-slate-700 whitespace-pre-line">
      {block.text}
    </p>
  );
}
