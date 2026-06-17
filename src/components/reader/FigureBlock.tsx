import type { FigureBlock as FigureBlockType } from "@/types";
import VisualSvgFrame from "@/components/reader/VisualSvgFrame";

interface Props {
  block: FigureBlockType;
}

export default function FigureBlock({ block }: Props) {
  return <VisualSvgFrame svg={block.svg} caption={block.caption} label="概念图示" />;
}
