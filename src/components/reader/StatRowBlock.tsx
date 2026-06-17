import type { StatRowBlock as StatRowBlockType } from "@/types";

export default function StatRowBlock({ block }: { block: StatRowBlockType }) {
  return (
    <div className="reader-panel my-6 grid grid-cols-2 gap-px overflow-hidden bg-slate-200 sm:grid-cols-4">
      {block.stats.map((s, i) => (
        <div key={i} className="bg-white p-5 text-center transition hover:bg-sky-50">
          <div className="text-2xl font-bold text-sky-700">{s.value}</div>
          <div className="mt-1 text-xs uppercase tracking-wide text-slate-400">
            {s.label}
          </div>
        </div>
      ))}
    </div>
  );
}
