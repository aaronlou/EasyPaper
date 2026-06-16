import type { ComparisonBlock as ComparisonBlockType } from "@/types";

export default function ComparisonBlock({
  block,
}: {
  block: ComparisonBlockType;
}) {
  return (
    <div className="overflow-x-auto my-6 rounded-lg border border-gray-200">
      <table className="w-full text-sm">
        <thead>
          <tr className="bg-gray-50 border-b border-gray-200">
            {block.columns.map((col, i) => (
              <th
                key={i}
                className="px-4 py-3 text-left font-medium text-gray-700 first:font-mono first:text-xs first:uppercase first:tracking-wide first:text-blue-600"
              >
                {col}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {block.rows.map((row, ri) => (
            <tr key={ri} className="border-b border-gray-100 last:border-0 hover:bg-gray-50/50">
              <td className="px-4 py-2.5 font-mono text-xs text-blue-600">
                {row.label}
              </td>
              {row.cells.map((cell, ci) => (
                <td key={ci} className="px-4 py-2.5 text-gray-700">
                  {cell}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
