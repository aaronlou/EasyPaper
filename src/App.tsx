import { useCallback, useEffect, useState } from "react";
import { usePaperStore } from "@/stores/usePaperStore";
import UploadView from "@/views/UploadView";
import ProcessingView from "@/views/ProcessingView";
import ReaderView from "@/views/ReaderView";

type View = "upload" | "processing" | "reader";

export default function App() {
  const [view, setView] = useState<View>("upload");
  const [currentPaperId, setCurrentPaperId] = useState<string | null>(null);

  const { checkHealth, loadPapers } = usePaperStore();

  // 启动时做一次健康检查 + 加载论文列表
  useEffect(() => {
    checkHealth();
    loadPapers();
  }, [checkHealth, loadPapers]);

  const handleUploadComplete = useCallback((paperId: string) => {
    setCurrentPaperId(paperId);
    setView("processing");
  }, []);

  const handleProcessingDone = useCallback((paperId: string) => {
    setCurrentPaperId(paperId);
    setView("reader");
  }, []);

  const handleOpenPaper = useCallback((paperId: string, status?: string) => {
    setCurrentPaperId(paperId);
    setView(status === "completed" ? "reader" : "processing");
  }, []);

  const handleBackToUpload = useCallback(() => {
    setView("upload");
    setCurrentPaperId(null);
  }, []);

  return (
    <div className="min-h-screen bg-[#f8f9fa]">
      {/* 极简顶栏 */}
      <header className="border-b bg-white px-6 py-3 flex items-center justify-between sticky top-0 z-50">
        <button
          onClick={handleBackToUpload}
          className="text-lg font-semibold tracking-tight text-gray-800 hover:text-blue-600 transition-colors"
        >
          📄 EasyPaper
        </button>
        {currentPaperId && (
          <span className="text-xs text-gray-400 font-mono">
            {currentPaperId.slice(0, 8)}
          </span>
        )}
      </header>

      {/* 视图切换 */}
      {view === "upload" && (
        <UploadView onDone={handleUploadComplete} onOpenPaper={handleOpenPaper} />
      )}
      {view === "processing" && currentPaperId && (
        <ProcessingView
          paperId={currentPaperId}
          onDone={() => handleProcessingDone(currentPaperId)}
          onBack={handleBackToUpload}
          onOpenReader={(paperId) => handleProcessingDone(paperId)}
        />
      )}
      {view === "reader" && currentPaperId && (
        <ReaderView paperId={currentPaperId} />
      )}
    </div>
  );
}
