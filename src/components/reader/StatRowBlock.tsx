import type { StatRowBlock as StatRowBlockType } from "@/types";

export default function StatRowBlock({ block }: { block: StatRowBlockType }) {
  return (
    <div className="grid grid-cols-2 sm:grid-cols-4 gap-px bg-gray-200 rounded-lg overflow-hidden my-6">
      {block.stats.map((s, i) => (
        <div key={i} className="bg-white p-5 text-center">
          <div className="text-2xl font-bold text-blue-600">{s.value}</div>
          <div className="text-xs text-gray-400 mt-1 uppercase tracking-wide">
            {s.label}
          </div>
        </div>
      ))}
    </div>
  );
}
