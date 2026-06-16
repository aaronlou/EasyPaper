import type { CodeFragmentBlock as CodeFragmentBlockType } from "@/types";

export default function CodeFragmentBlock({
  block,
}: {
  block: CodeFragmentBlockType;
}) {
  return (
    <div className="my-4 rounded-lg border border-gray-300 overflow-hidden">
      {block.lang && (
        <div className="bg-gray-100 px-4 py-1.5 text-xs font-mono text-gray-400 border-b border-gray-200">
          {block.lang}
        </div>
      )}
      <pre className="bg-gray-900 text-gray-100 p-4 overflow-x-auto text-sm leading-relaxed">
        <code>{block.code}</code>
      </pre>
    </div>
  );
}
