import { useCallback, useEffect, useRef, useState } from "react";
import { Upload, FileText, AlertCircle, CheckCircle2, Clock } from "lucide-react";
import { usePaperStore } from "@/stores/usePaperStore";
import { cn } from "@/lib/cn";
import type { PaperSummary } from "@/types";

interface Props {
  onDone: (paperId: string) => void;
  onOpenPaper: (paperId: string, status: PaperSummary["status"]) => void;
}

export default function UploadView({ onDone, onOpenPaper }: Props) {
  const { papers, loadPapers, uploading, uploadPaper, health } =
    usePaperStore();
  const [error, setError] = useState<string | null>(null);
  const [dragging, setDragging] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  // 定时刷新论文列表（以获取 processing/completed 状态变更）
  useEffect(() => {
    const timer = setInterval(loadPapers, 3000);
    return () => clearInterval(timer);
  }, [loadPapers]);

  const handleFile = useCallback(
    async (file: File) => {
      if (file.type !== "application/pdf" && !file.name.endsWith(".pdf")) {
        setError("仅支持 PDF 文件");
        return;
      }
      if (file.size > 50 * 1024 * 1024) {
        setError("文件不能超过 50 MB");
        return;
      }
      setError(null);
      try {
        const paperId = await uploadPaper(file);
        onDone(paperId);
      } catch (e) {
        setError(e instanceof Error ? e.message : "上传失败");
      }
    },
    [uploadPaper, onDone],
  );

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      setDragging(false);
      const file = e.dataTransfer.files[0];
      if (file) handleFile(file);
    },
    [handleFile],
  );

  const handleClick = (p: PaperSummary) => {
    onOpenPaper(p.id, p.status);
  };

  const statusIcon = (s: PaperSummary["status"]) => {
    switch (s) {
      case "uploaded":
        return <Clock className="w-4 h-4 text-yellow-500" />;
      case "processing":
        return (
          <span className="inline-block w-4 h-4 border-2 border-blue-400 border-t-transparent rounded-full animate-spin" />
        );
      case "completed":
        return <CheckCircle2 className="w-4 h-4 text-green-500" />;
      case "failed":
        return <AlertCircle className="w-4 h-4 text-red-500" />;
    }
  };

  return (
    <main className="max-w-2xl mx-auto px-6 py-20">
      <h1 className="text-3xl font-bold tracking-tight text-gray-900 mb-3">
        解读你的学术论文
      </h1>
      <p className="text-gray-500 mb-10">
        上传一篇 PDF，AI 将自动生成交互式讲解网页，帮你和读者快速理解论文核心内容。
      </p>

      {/* 上传区 */}
      <div
        onDragOver={(e) => {
          e.preventDefault();
          setDragging(true);
        }}
        onDragLeave={() => setDragging(false)}
        onDrop={handleDrop}
        onClick={() => inputRef.current?.click()}
        className={cn(
          "border-2 border-dashed rounded-xl p-12 text-center cursor-pointer transition-all duration-200",
          dragging
            ? "border-blue-500 bg-blue-50 scale-[1.01]"
            : "border-gray-300 bg-white hover:border-blue-400 hover:bg-blue-50/40",
          uploading && "opacity-60 pointer-events-none",
        )}
      >
        <input
          ref={inputRef}
          type="file"
          accept=".pdf"
          className="hidden"
          onChange={(e) => {
            const f = e.target.files?.[0];
            if (f) handleFile(f);
          }}
        />
        {uploading ? (
          <div className="flex flex-col items-center gap-3">
            <span className="inline-block w-8 h-8 border-3 border-blue-500 border-t-transparent rounded-full animate-spin" />
            <span className="text-gray-500">正在上传并提取文本...</span>
          </div>
        ) : (
          <div className="flex flex-col items-center gap-3">
            <Upload className="w-10 h-10 text-gray-400" />
            <span className="text-gray-600 font-medium">
              拖拽 PDF 到此，或点击选择
            </span>
            <span className="text-sm text-gray-400">最大 50 MB</span>
          </div>
        )}
      </div>

      {error && (
        <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg text-sm text-red-700 flex items-center gap-2">
          <AlertCircle className="w-4 h-4" />
          {error}
        </div>
      )}

      {!health?.llm_configured && (
        <div className="mt-4 p-3 bg-amber-50 border border-amber-200 rounded-lg text-sm text-amber-700 flex items-center gap-2">
          <AlertCircle className="w-4 h-4" />
          尚未配置 LLM（OPENAI_API_KEY），论文可以上传保存，但无法自动解读。
        </div>
      )}

      {/* 已有论文列表 */}
      {papers.length > 0 && (
        <section className="mt-12">
          <h2 className="text-lg font-semibold text-gray-800 mb-4">
            已上传的论文
          </h2>
          <div className="space-y-2">
            {papers.map((p) => (
              <div
                key={p.id}
                onClick={() => handleClick(p)}
                className={cn(
                  "flex items-center gap-4 p-4 rounded-lg border bg-white transition-colors",
                  "cursor-pointer hover:border-blue-400 hover:bg-blue-50/30",
                )}
              >
                <FileText className="w-5 h-5 text-gray-400 shrink-0" />
                <div className="flex-1 min-w-0">
                  <div className="text-sm font-medium text-gray-800 truncate">
                    {p.title}
                  </div>
                  <div className="text-xs text-gray-400 mt-0.5">
                    {p.filename} · {p.char_count.toLocaleString()} 字符
                  </div>
                </div>
                <div className="flex items-center gap-2 shrink-0">
                  {statusIcon(p.status)}
                  <span className="text-xs text-gray-400">
                    {p.status === "uploaded"
                      ? "待处理"
                      : p.status === "processing"
                        ? "解读中"
                        : p.status === "completed"
                          ? "已完成"
                          : "失败"}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </section>
      )}
    </main>
  );
}
