import { useEffect, useState } from "react";
import { usePaperStore } from "@/stores/usePaperStore";
import { renderBlock } from "@/renderer/blockRenderer";
import { ReaderProvider } from "@/contexts/ReaderContext";
import ConceptDeepDiveModal from "@/components/reader/ConceptDeepDiveModal";
import ConceptGraphBlock from "@/components/reader/ConceptGraphBlock";
import { BookOpen, Layers3 } from "lucide-react";
import type { SectionBlock as SectionBlockType } from "@/types";

interface Props {
  paperId: string;
}

export default function ReaderView({ paperId }: Props) {
  const { current, loadPaper, loadingCurrent } = usePaperStore();
  const [activeConceptId, setActiveConceptId] = useState<string | null>(null);

  useEffect(() => {
    loadPaper(paperId);
  }, [paperId, loadPaper]);

  if (loadingCurrent || !current || !current.interpretation) {
    return (
      <main className="max-w-3xl mx-auto px-6 py-32 text-center text-gray-400">
        <span className="inline-block w-8 h-8 border-3 border-gray-300 border-t-gray-500 rounded-full animate-spin mb-4" />
        <p>加载中...</p>
      </main>
    );
  }

  const { interpretation, paper } = current;
  const blocks = interpretation.blocks;
  const sections = blocks.filter(
    (block): block is SectionBlockType => block.type === "section",
  );

  return (
    <ReaderProvider
      paperId={paperId}
      interpretation={interpretation}
      activeConceptId={activeConceptId}
      setActiveConceptId={setActiveConceptId}
    >
      <main className="mx-auto grid max-w-7xl gap-8 px-5 py-8 pb-32 lg:grid-cols-[minmax(0,1fr)_280px] lg:px-8">
        <div className="min-w-0">
          <header className="reader-hero mb-8 overflow-hidden rounded-2xl border border-slate-200 bg-white">
            <div className="border-b border-slate-200 bg-[#f7fbff] px-6 py-5">
              <div className="mb-4 flex flex-wrap items-center gap-2 text-xs font-semibold uppercase tracking-wide text-slate-500">
                <span className="inline-flex items-center gap-1.5 rounded-full border border-sky-100 bg-white px-3 py-1 text-sky-700">
                  <BookOpen className="h-3.5 w-3.5" />
                  Interactive Paper
                </span>
                <span>{paper.char_count.toLocaleString()} chars</span>
              </div>
              <h1 className="max-w-4xl text-3xl font-bold leading-tight text-slate-950 md:text-4xl">
                {paper.title}
              </h1>
              {paper.authors.length > 0 && (
                <p className="mt-3 text-sm text-slate-500">
                  {paper.authors.join(", ")}
                </p>
              )}
            </div>
            {interpretation.summary && (
              <div className="grid gap-4 px-6 py-5 md:grid-cols-[150px_minmax(0,1fr)]">
                <div className="text-sm font-semibold text-slate-400">核心贡献</div>
                <p className="text-base leading-7 text-slate-700">
                  {interpretation.summary}
                </p>
              </div>
            )}
          </header>

          <ConceptGraphBlock
            concepts={interpretation.concepts}
            onSelectConcept={setActiveConceptId}
          />

          <div className="space-y-6">
            {blocks.map((block) => (
              <div key={block.id} id={block.type === "section" ? block.id : undefined}>
                {renderBlock(block)}
              </div>
            ))}
          </div>

          <div className="mt-16 border-t border-slate-200 pt-8 text-center text-sm font-mono text-slate-300">
            END OF INTERPRETATION
          </div>
        </div>

        <aside className="hidden lg:block">
          <div className="sticky top-20 space-y-4">
            {sections.length > 0 && (
              <nav className="reader-panel p-4">
                <div className="mb-3 flex items-center gap-2 text-sm font-semibold text-slate-800">
                  <Layers3 className="h-4 w-4 text-sky-600" />
                  章节
                </div>
                <div className="space-y-1">
                  {sections.map((section) => (
                    <a
                      key={section.id}
                      href={`#${section.id}`}
                      className="block rounded-md px-2 py-1.5 text-sm text-slate-500 transition hover:bg-sky-50 hover:text-sky-700"
                    >
                      <span className="mr-2 font-mono text-xs text-slate-300">
                        {section.num}
                      </span>
                      {section.title}
                    </a>
                  ))}
                </div>
              </nav>
            )}

            {interpretation.concepts.length > 0 && (
              <div className="reader-panel p-4">
                <div className="mb-3 text-sm font-semibold text-slate-800">
                  概念索引
                </div>
                <div className="flex flex-wrap gap-2">
                  {interpretation.concepts.slice(0, 14).map((concept) => (
                    <button
                      key={concept.id}
                      onClick={() => setActiveConceptId(concept.id)}
                      className="rounded-full border border-slate-200 bg-white px-2.5 py-1 text-xs text-slate-600 transition hover:border-sky-300 hover:bg-sky-50 hover:text-sky-700"
                    >
                      {concept.term}
                    </button>
                  ))}
                </div>
              </div>
            )}
          </div>
        </aside>
      </main>

      <ConceptDeepDiveModal
        conceptId={activeConceptId}
        onClose={() => setActiveConceptId(null)}
      />
    </ReaderProvider>
  );
}
