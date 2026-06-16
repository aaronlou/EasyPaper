import type { TimelineBlock as TimelineBlockType } from "@/types";

export default function TimelineBlock({ block }: { block: TimelineBlockType }) {
  return (
    <div className="relative pl-8 border-l-2 border-blue-200 space-y-8 my-6">
      {block.items.map((item, i) => (
        <div key={i} className="relative">
          <div className="absolute -left-[2.15rem] top-1 w-3 h-3 rounded-full bg-blue-500 border-2 border-white shadow" />
          <div className="text-xs font-mono text-blue-600 mb-1">
            {item.year}
          </div>
          <div className="font-semibold text-gray-800 mb-1">{item.title}</div>
          <div className="text-sm text-gray-500">{item.body}</div>
        </div>
      ))}
    </div>
  );
}
