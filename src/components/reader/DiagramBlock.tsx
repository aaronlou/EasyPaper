import type { DiagramBlock as DiagramBlockType } from "@/types";
import VisualSvgFrame from "@/components/reader/VisualSvgFrame";

interface Props {
  block: DiagramBlockType;
}

export default function DiagramBlock({ block }: Props) {
  return <VisualSvgFrame svg={block.svg} caption={block.caption} label="流程 / 架构图" />;
}
