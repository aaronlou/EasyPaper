import { useEffect, useRef } from "react";
import { usePaperStore } from "@/stores/usePaperStore";

interface Props {
  paperId: string;
  onDone: () => void;
}

export default function ProcessingView({ paperId, onDone }: Props) {
  const { current, loadPaper } = usePaperStore();
  const doneCalled = useRef(false);

  useEffect(() => {
    let active = true;
    let timer: ReturnType<typeof setInterval>;

    const poll = async () => {
      if (!active) return;
      await loadPaper(paperId);
    };

    // 立即拉一次
    poll();

    // 每 2 秒轮询
    timer = setInterval(poll, 2000);

    return () => {
      active = false;
      clearInterval(timer);
    };
  }, [paperId, loadPaper]);

  // 当解读完成时跳转
  useEffect(() => {
    if (current?.paper.status === "completed" && !doneCalled.current) {
      doneCalled.current = true;
      // 稍等一下让用户看到"完成"状态
      const t = setTimeout(onDone, 1200);
      return () => clearTimeout(t);
    }
  }, [current?.paper.status, onDone]);

  const status = current?.paper.status ?? "processing";
  const title = current?.paper.title ?? "处理中...";

  return (
    <main className="max-w-xl mx-auto px-6 py-32 text-center">
      {status === "processing" || status === "uploaded" ? (
        <>
          <span className="inline-block w-12 h-12 border-4 border-blue-500 border-t-transparent rounded-full animate-spin mb-6" />
          <h2 className="text-xl font-semibold text-gray-800 mb-2">
            AI 正在解读
          </h2>
          <p className="text-gray-500 text-sm mb-1">{title}</p>
          <p className="text-gray-400 text-xs">
            正在调用 LLM 分析论文结构、生成讲解内容...通常需要 30 秒到 2 分钟。
          </p>
        </>
      ) : status === "completed" ? (
        <>
          <span className="inline-block w-12 h-12 bg-green-100 rounded-full flex items-center justify-center mb-6">
            <span className="text-2xl">✓</span>
          </span>
          <h2 className="text-xl font-semibold text-gray-800 mb-2">
            解读完成！
          </h2>
          <p className="text-gray-400 text-sm">马上跳转到交互式讲解页面...</p>
        </>
      ) : (
        <>
          <span className="text-4xl mb-6">⚠️</span>
          <h2 className="text-xl font-semibold text-red-700 mb-2">
            解读失败
          </h2>
          <p className="text-gray-500 text-sm">{title}</p>
          <p className="text-gray-400 text-xs mt-2">
            请检查 LLM 配置（OPENAI_API_KEY），或确保 PDF 文本可读。
          </p>
        </>
      )}
    </main>
  );
}
