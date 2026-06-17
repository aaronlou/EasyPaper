import { useEffect, useMemo, useRef, useState } from "react";
import { usePaperStore } from "@/stores/usePaperStore";
import * as api from "@/lib/api";
import type { ProgressInfo } from "@/types";

interface Props {
  paperId: string;
  onDone: () => void;
  onBack: () => void;
  onOpenReader: (paperId: string) => void;
}

const STEPS = [
  { phase: "uploaded", label: "文本提取", desc: "从 PDF 提取标题、作者与正文" },
  { phase: "reading", label: "并行阅读", desc: "多个 reader agent 分片理解论文" },
  { phase: "reading", label: "多 Agent 汇总", desc: "合并概念、证据与机制笔记" },
  { phase: "parsing", label: "结构化解析", desc: "组装图示、表格与自测题" },
  { phase: "saving", label: "页面准备", desc: "写入数据库并渲染阅读器" },
];

const SUPPORTING_MESSAGES = [
  "阅读 agent 正在各自提炼核心观点...",
  "结构 agent 正在检查机制链路是否讲得通...",
  "讲解 agent 正在把术语转成可复述的直觉...",
  "证据 agent 正在保留论文中的关键数字与引用...",
  "Reducer 正在准备图示、概念卡和自测题...",
];

type AgentLaneStatus = "waiting" | "active" | "done";

interface AgentLane {
  label: string;
  detail: string;
  status: AgentLaneStatus;
}

function formatElapsed(seconds: number): string {
  if (seconds < 60) return `${seconds} 秒`;
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return s === 0 ? `${m} 分钟` : `${m} 分 ${s} 秒`;
}

export default function ProcessingView({ paperId, onDone, onBack, onOpenReader }: Props) {
  const { current, papers, loadPaper, loadPapers, retryInterpretation } = usePaperStore();
  const doneCalled = useRef(false);
  const onDoneRef = useRef(onDone);
  const [progress, setProgress] = useState<ProgressInfo | null>(null);
  const [retrying, setRetrying] = useState(false);
  const [retryError, setRetryError] = useState<string | null>(null);
  const [elapsed, setElapsed] = useState(0);
  const [startAt] = useState(() => Date.now());

  useEffect(() => {
    onDoneRef.current = onDone;
  }, [onDone]);

  // 轮询论文详情（用于判断 completed / failed）
  useEffect(() => {
    let active = true;
    let timer: ReturnType<typeof setInterval>;

    const poll = async () => {
      if (!active) return;
      await loadPaper(paperId);
      await loadPapers();
    };

    poll();
    timer = setInterval(poll, 2500);

    return () => {
      active = false;
      clearInterval(timer);
    };
  }, [paperId, loadPaper, loadPapers]);

  // 轮询进度
  useEffect(() => {
    let active = true;
    let timer: ReturnType<typeof setInterval>;

    const poll = async () => {
      if (!active) return;
      try {
        const p = await api.getProgress(paperId);
        setProgress(p);
      } catch {
        // 忽略进度接口短暂失败
      }
    };

    poll();
    timer = setInterval(poll, 1200);

    return () => {
      active = false;
      clearInterval(timer);
    };
  }, [paperId]);

  // 已用时间计时器
  useEffect(() => {
    const timer = setInterval(() => {
      setElapsed(Math.floor((Date.now() - startAt) / 1000));
    }, 1000);
    return () => clearInterval(timer);
  }, [startAt]);

  // 解读完成时跳转
  useEffect(() => {
    if (current?.paper.status === "completed" && !doneCalled.current) {
      doneCalled.current = true;
      const t = setTimeout(() => onDoneRef.current(), 900);
      return () => clearTimeout(t);
    }
  }, [current?.paper.status]);

  const status = current?.paper.status ?? "processing";
  const title = current?.paper.title ?? "处理中...";
  const phase = progress?.phase ?? "interpreting";

  // 当前步骤索引：根据 phase 映射
  const currentStepIndex = useMemo(() => {
    switch (phase) {
      case "uploaded":
        return 0;
      case "interpreting":
      case "reading":
        return (progress?.percent ?? 35) < 60 ? 1 : 2;
      case "parsing":
        return 3;
      case "saving":
      case "completed":
        return 4;
      default:
        return 0;
    }
  }, [phase, progress?.percent]);

  // 后端实时进度为主，本地轮播只做辅助，避免覆盖真实 reader agent 状态。
  const dynamicMessage = useMemo(() => {
    if (progress?.message) return progress.message;
    const idx = Math.floor(elapsed / 7) % SUPPORTING_MESSAGES.length;
    return SUPPORTING_MESSAGES[idx];
  }, [phase, progress?.message, elapsed]);

  const helperMessage = useMemo(() => {
    if (phase !== "interpreting" && phase !== "reading") return "";
    const idx = Math.floor(elapsed / 7) % SUPPORTING_MESSAGES.length;
    return SUPPORTING_MESSAGES[idx];
  }, [phase, elapsed]);

  const agentLanes = useMemo<AgentLane[]>(() => {
    const percent = progress?.percent ?? 10;
    const makeStatus = (activeAt: number, doneAt: number): AgentLaneStatus => {
      if (percent >= doneAt || phase === "completed" || phase === "saving") return "done";
      if (percent >= activeAt) return "active";
      return "waiting";
    };

    return [
      {
        label: "Reader Agents",
        detail: "分片阅读、提取概念与证据",
        status: makeStatus(20, 72),
      },
      {
        label: "Structure Agent",
        detail: "整理问题、机制、取舍链路",
        status: makeStatus(45, 82),
      },
      {
        label: "Teaching Agent",
        detail: "生成费曼式图示、表格与自测",
        status: makeStatus(55, 90),
      },
    ];
  }, [phase, progress?.percent]);

  const isFailed = status === "failed" || phase === "failed";
  const completedAlternatives = papers.filter(
    (paper) =>
      paper.status === "completed" &&
      paper.id !== paperId &&
      current?.paper.title &&
      paper.title === current.paper.title,
  );

  const handleRetry = async () => {
    setRetrying(true);
    setRetryError(null);
    try {
      await retryInterpretation(paperId);
      await loadPaper(paperId);
    } catch (error) {
      setRetryError(error instanceof Error ? error.message : "重新解读失败");
    } finally {
      setRetrying(false);
    }
  };

  return (
    <main className="max-w-2xl mx-auto px-6 py-20 text-center">
      {isFailed ? (
        <div className="rounded-2xl bg-red-50 p-8">
          <h2 className="text-2xl font-semibold text-red-700 mb-2">解读失败</h2>
          <p className="text-gray-700 font-medium mb-1">{title}</p>
          <p className="text-gray-500 text-sm mt-2">
            {progress?.message ||
              "请检查 LLM 配置（OPENAI_API_KEY），或确保 PDF 文本可读。"}
          </p>
          {retryError && (
            <p className="mt-3 text-sm text-red-700">{retryError}</p>
          )}
          <div className="mt-6 flex flex-wrap justify-center gap-3">
            <button
              type="button"
              onClick={handleRetry}
              disabled={retrying}
              className="rounded-lg bg-red-600 px-4 py-2 text-sm font-medium text-white transition hover:bg-red-700 disabled:cursor-not-allowed disabled:opacity-60"
            >
              {retrying ? "正在重试..." : "重新解读"}
            </button>
            <button
              type="button"
              onClick={onBack}
              className="rounded-lg border border-red-200 bg-white px-4 py-2 text-sm font-medium text-red-700 transition hover:bg-red-100"
            >
              返回列表
            </button>
            {completedAlternatives.map((paper) => (
              <button
                key={paper.id}
                type="button"
                onClick={() => onOpenReader(paper.id)}
                className="rounded-lg bg-red-600 px-4 py-2 text-sm font-medium text-white transition hover:bg-red-700"
              >
                打开历史已完成解读
              </button>
            ))}
          </div>
        </div>
      ) : status === "completed" ? (
        <div className="rounded-2xl bg-green-50 p-8">
          <span className="inline-flex items-center justify-center w-16 h-16 bg-green-100 rounded-full mb-4">
            <span className="text-3xl">✓</span>
          </span>
          <h2 className="text-2xl font-semibold text-gray-800 mb-2">解读完成！</h2>
          <p className="text-gray-500 text-sm">马上跳转到交互式讲解页面...</p>
        </div>
      ) : (
        <>
          <div className="mb-8">
            <span className="inline-block w-14 h-14 border-4 border-blue-500 border-t-transparent rounded-full animate-spin mb-5" />
            <h2 className="text-2xl font-semibold text-gray-800 mb-2">AI 正在解读</h2>
            <p className="text-gray-600 font-medium mb-1">{title}</p>
            {progress?.stage && (
              <p className="mb-2 text-xs font-semibold uppercase tracking-wide text-slate-400">
                当前阶段：{progress.stage}
              </p>
            )}
            <p className="text-blue-600 text-sm min-h-[1.5rem]">{dynamicMessage}</p>
            {helperMessage && helperMessage !== dynamicMessage && (
              <p className="mt-1 text-xs text-gray-400">{helperMessage}</p>
            )}
          </div>

          {/* 进度条 */}
          <div className="mb-10">
            <div className="flex justify-between text-xs text-gray-400 mb-2">
              <span>总进度</span>
              <span>{progress?.percent ?? 10}%</span>
            </div>
            <div className="h-2 bg-gray-100 rounded-full overflow-hidden">
              <div
                className="h-full bg-blue-500 rounded-full transition-all duration-700 ease-out"
                style={{ width: `${progress?.percent ?? 10}%` }}
              />
            </div>
            <p className="text-xs text-gray-400 mt-3">
              已用时 {formatElapsed(elapsed)} · 并行阅读通常需要 30 秒 ~ 2 分钟
            </p>
          </div>

          <div className="mb-10 grid gap-3 rounded-2xl border border-slate-200 bg-white p-4 text-left shadow-sm sm:grid-cols-3">
            {agentLanes.map((lane) => (
              <div key={lane.label} className="min-w-0 rounded-lg bg-slate-50 p-3">
                <div className="mb-2 flex items-center justify-between gap-2">
                  <span className="truncate text-xs font-semibold text-slate-700">
                    {lane.label}
                  </span>
                  <span
                    className={[
                      "h-2.5 w-2.5 rounded-full",
                      lane.status === "done"
                        ? "bg-emerald-500"
                        : lane.status === "active"
                        ? "bg-blue-500 animate-pulse"
                        : "bg-slate-300",
                    ].join(" ")}
                  />
                </div>
                <p className="text-xs leading-5 text-slate-500">{lane.detail}</p>
                <p className="mt-2 text-[11px] font-medium text-slate-400">
                  {lane.status === "done"
                    ? "完成"
                    : lane.status === "active"
                    ? "处理中"
                    : "等待中"}
                </p>
              </div>
            ))}
          </div>

          {/* 步骤条 */}
          <div className="relative">
            <div className="absolute top-5 left-[10%] right-[10%] h-0.5 bg-gray-100 -z-10" />
            <ol className="flex justify-between">
              {STEPS.map((step, idx) => {
                const done = idx < currentStepIndex;
                const active = idx === currentStepIndex;
                return (
                  <li key={step.label} className="flex flex-col items-center w-1/5">
                    <div
                      className={[
                        "w-10 h-10 rounded-full flex items-center justify-center text-sm font-semibold border-2 mb-3 transition-colors",
                        done
                          ? "bg-green-500 border-green-500 text-white"
                          : active
                          ? "bg-white border-blue-500 text-blue-600"
                          : "bg-white border-gray-200 text-gray-300",
                      ].join(" ")}
                    >
                      {done ? "✓" : idx + 1}
                    </div>
                    <span
                      className={[
                        "text-xs font-medium mb-0.5",
                        done || active ? "text-gray-700" : "text-gray-400",
                      ].join(" ")}
                    >
                      {step.label}
                    </span>
                    <span className="text-[10px] text-gray-400 leading-tight px-1">
                      {step.desc}
                    </span>
                  </li>
                );
              })}
            </ol>
          </div>
        </>
      )}
    </main>
  );
}
