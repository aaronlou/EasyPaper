import type { SectionBlock as SectionBlockType } from "@/types";

export default function SectionBlock({ block }: { block: SectionBlockType }) {
  return (
    <section className="mt-12 border-t border-slate-200 pt-8">
      <div className="mb-3 font-mono text-xs font-semibold uppercase tracking-wide text-sky-600">
        Section {block.num}
      </div>
      <h2 className="text-2xl font-bold leading-tight text-slate-950 md:text-3xl">
        {block.title}
      </h2>
    </section>
  );
}
