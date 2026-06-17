import { useEffect, useMemo, useRef, useState } from "react";
import { usePaperStore } from "@/stores/usePaperStore";
import * as api from "@/lib/api";
import type { ProgressInfo } from "@/types";

interface Props {
  paperId: string;
  onDone: () => void;
}

const STEPS = [
  { phase: "uploaded", label: "文本提取", desc: "从 PDF 提取标题、作者与正文" },
  { phase: "interpreting", label: "结构分析", desc: "AI 理解论文框架与核心贡献" },
  { phase: "interpreting", label: "讲解生成", desc: "把专业内容转写成通俗讲解" },
  { phase: "parsing", label: "结果整理", desc: "解析并校验结构化输出" },
  { phase: "saving", label: "页面准备", desc: "写入数据库并渲染阅读器" },
];

const INTERPRETING_MESSAGES = [
  "正在阅读摘要与引言...",
  "正在提取核心贡献与创新点...",
  "正在梳理关键技术细节...",
  "正在把专业术语翻译成通俗语言...",
  "正在设计概念卡片与自测题目...",
  "正在润色讲解表达...",
];

function formatElapsed(seconds: number): string {
  if (seconds < 60) return `${seconds} 秒`;
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return s === 0 ? `${m} 分钟` : `${m} 分 ${s} 秒`;
}

export default function ProcessingView({ paperId, onDone }: Props) {
  const { current, loadPaper } = usePaperStore();
  const doneCalled = useRef(false);
  const onDoneRef = useRef(onDone);
  const [progress, setProgress] = useState<ProgressInfo | null>(null);
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
    };

    poll();
    timer = setInterval(poll, 2500);

    return () => {
      active = false;
      clearInterval(timer);
    };
  }, [paperId, loadPaper]);

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
        // 在 interpreting 阶段根据已用时间或 percent 细分两步
        return (progress?.percent ?? 35) < 50 ? 1 : 2;
      case "parsing":
        return 3;
      case "saving":
      case "completed":
        return 4;
      default:
        return 0;
    }
  }, [phase, progress?.percent]);

  // interpreting 阶段的动态文案
  const dynamicMessage = useMemo(() => {
    if (phase !== "interpreting") return progress?.message ?? "";
    const idx = Math.floor(elapsed / 7) % INTERPRETING_MESSAGES.length;
    return INTERPRETING_MESSAGES[idx];
  }, [phase, progress?.message, elapsed]);

  const isFailed = status === "failed" || phase === "failed";

  return (
    <main className="max-w-2xl mx-auto px-6 py-20 text-center">
      {isFailed ? (
        <div className="rounded-2xl bg-red-50 p-8">
          <span className="text-5xl mb-4 inline-block">⚠️</span>
          <h2 className="text-2xl font-semibold text-red-700 mb-2">解读失败</h2>
          <p className="text-gray-700 font-medium mb-1">{title}</p>
          <p className="text-gray-500 text-sm mt-2">
            {progress?.message || "请检查 LLM 配置（OPENAI_API_KEY），或确保 PDF 文本可读。"}
          </p>
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
            <p className="text-blue-600 text-sm min-h-[1.5rem]">{dynamicMessage}</p>
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
              已用时 {formatElapsed(elapsed)} · 通常需要 30 秒 ~ 2 分钟
            </p>
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
