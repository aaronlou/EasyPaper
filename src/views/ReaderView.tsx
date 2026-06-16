import { useEffect } from "react";
import { usePaperStore } from "@/stores/usePaperStore";
import { renderBlock } from "@/renderer/blockRenderer";

interface Props {
  paperId: string;
}

export default function ReaderView({ paperId }: Props) {
  const { current, loadPaper, loadingCurrent } = usePaperStore();

  useEffect(() => {
    loadPaper(paperId);
  }, [paperId, loadPaper]);

  if (loadingCurrent || !current || !current.interpretation) {
    return (
      <main className="max-w-3xl mx-auto px-6 py-32 text-center text-gray-400">
        <span className="inline-block w-8 h-8 border-3 border-gray-300 border-t-gray-500 rounded-full animate-spin mb-4" />
        <p>加载中...</p>
      </main>
    );
  }

  const { interpretation, paper } = current;
  const blocks = interpretation.blocks;

  return (
    <main className="max-w-3xl mx-auto px-6 py-12 pb-32">
      {/* 论文信息 header */}
      <header className="mb-12 pb-8 border-b">
        <h1 className="text-3xl font-bold text-gray-900 leading-tight mb-3">
          {paper.title}
        </h1>
        {paper.authors.length > 0 && (
          <p className="text-gray-500 text-sm mb-2">
            {paper.authors.join(", ")}
          </p>
        )}
        {interpretation.summary && (
          <p className="text-base text-gray-600 mt-4 leading-relaxed bg-blue-50/60 border border-blue-100 rounded-lg p-4">
            💡 {interpretation.summary}
          </p>
        )}
      </header>

      {/* Block 渲染 */}
      <div className="space-y-6">
        {blocks.map((block) => (
          <div key={block.id}>{renderBlock(block)}</div>
        ))}
      </div>

      {/* End of paper marker */}
      <div className="mt-16 pt-8 border-t text-center text-sm text-gray-300 font-mono">
        — END OF INTERPRETATION —
      </div>
    </main>
  );
}
