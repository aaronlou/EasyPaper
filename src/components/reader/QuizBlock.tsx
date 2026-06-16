import { useState } from "react";
import { CheckCircle2, XCircle, HelpCircle } from "lucide-react";
import type { QuizBlock as QuizBlockType } from "@/types";
import { cn } from "@/lib/cn";

export default function QuizBlock({ block }: { block: QuizBlockType }) {
  const [answered, setAnswered] = useState(false);
  const [selected, setSelected] = useState<number | null>(null);

  const handleSelect = (idx: number) => {
    if (answered) return; // 已经回答过了，不再变化
    setSelected(idx);
    setAnswered(true);
  };

  const correctIdx = block.options.findIndex((o) => o.correct);

  return (
    <div className="border-2 border-gray-200 rounded-xl p-6 my-6 bg-white">
      <div className="flex items-start gap-3 mb-4">
        <HelpCircle className="w-5 h-5 text-blue-500 shrink-0 mt-0.5" />
        <p className="font-semibold text-gray-800">{block.question}</p>
      </div>

      <div className="space-y-2">
        {block.options.map((opt, idx) => {
          let borderColor = "border-gray-200 hover:border-blue-300";
          let bgColor = "";
          let showIcon = null;

          if (answered) {
            if (idx === correctIdx) {
              borderColor = "border-green-400";
              bgColor = "bg-green-50";
              showIcon = <CheckCircle2 className="w-4 h-4 text-green-500" />;
            } else if (idx === selected && !opt.correct) {
              borderColor = "border-red-400";
              bgColor = "bg-red-50";
              showIcon = <XCircle className="w-4 h-4 text-red-500" />;
            }
          }

          return (
            <div
              key={idx}
              onClick={() => handleSelect(idx)}
              className={cn(
                "flex items-center gap-3 p-3 rounded-lg border cursor-pointer transition-colors",
                borderColor,
                bgColor,
                answered && "cursor-default",
              )}
            >
              <span
                className={cn(
                  "w-7 h-7 rounded-full border flex items-center justify-center text-xs font-mono shrink-0",
                  answered
                    ? idx === correctIdx
                      ? "border-green-400 bg-green-100 text-green-700"
                      : idx === selected
                        ? "border-red-400 bg-red-100 text-red-700"
                        : "border-gray-200 bg-gray-100 text-gray-400"
                    : "border-gray-300 text-gray-500",
                )}
              >
                {showIcon ?? String.fromCharCode(65 + idx)}
              </span>
              <span className="text-sm text-gray-700">{opt.text}</span>
            </div>
          );
        })}
      </div>

      {answered && (
        <div className="mt-4 p-3 bg-green-50 border border-green-200 rounded-lg text-sm text-green-800">
          ✓ {block.explain}
        </div>
      )}
    </div>
  );
}
