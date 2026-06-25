import { useCallback, useEffect, useState } from "react";
import { Settings2 } from "lucide-react";
import { usePaperStore } from "@/stores/usePaperStore";
import AiSettingsModal from "@/components/AiSettingsModal";
import UploadView from "@/views/UploadView";
import ProcessingView from "@/views/ProcessingView";
import ReaderView from "@/views/ReaderView";

type View = "upload" | "processing" | "reader";

export default function App() {
  const [view, setView] = useState<View>("upload");
  const [currentPaperId, setCurrentPaperId] = useState<string | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);

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
        <div className="flex items-center gap-3">
          {currentPaperId && (
            <span className="text-xs text-gray-400 font-mono">
              {currentPaperId.slice(0, 8)}
            </span>
          )}
          <button
            onClick={() => setSettingsOpen(true)}
            className="inline-flex h-9 w-9 items-center justify-center rounded-lg border border-slate-200 bg-white text-slate-500 transition hover:border-sky-300 hover:text-sky-700"
            aria-label="AI 模型配置"
            title="AI 模型配置"
          >
            <Settings2 className="h-4 w-4" />
          </button>
        </div>
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
      <footer className="border-t border-slate-200 bg-white/80 px-4 py-4 text-center text-xs text-slate-500">
        <span>终身学习</span>
        <span className="mx-2 text-slate-300">|</span>
        <a
          href="https://beian.miit.gov.cn/"
          target="_blank"
          rel="noreferrer"
          className="transition-colors hover:text-sky-700"
        >
          浙ICP备2024126456号-5
        </a>
      </footer>
      <AiSettingsModal
        open={settingsOpen}
        onClose={() => setSettingsOpen(false)}
        onSaved={checkHealth}
      />
    </div>
  );
}
