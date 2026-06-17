import { useMemo, useState } from "react";
import type { ChartBlock as ChartBlockType } from "@/types";
import { cn } from "@/lib/cn";

interface Props {
  block: ChartBlockType;
}

const COLORS = ["#0ea5e9", "#14b8a6", "#f59e0b", "#ef4444", "#6366f1", "#84cc16"];

function BarChart({
  data,
  xLabel,
  yLabel,
  activeIndex,
  onHover,
}: {
  data: { label: string; value: number }[];
  xLabel?: string;
  yLabel?: string;
  activeIndex: number | null;
  onHover: (index: number | null) => void;
}) {
  const width = 500;
  const height = 280;
  const margin = { top: 24, right: 24, bottom: 64, left: 56 };
  const chartW = width - margin.left - margin.right;
  const chartH = height - margin.top - margin.bottom;
  const max = Math.max(...data.map((d) => d.value), 0.1);
  const barGap = 16;
  const barW = (chartW - barGap * (data.length + 1)) / data.length;

  return (
    <svg viewBox={`0 0 ${width} ${height}`} className="w-full max-w-lg">
      {/* 坐标轴 */}
      <line x1={margin.left} y1={margin.top} x2={margin.left} y2={margin.top + chartH} stroke="#e5e7eb" strokeWidth={1} />
      <line x1={margin.left} y1={margin.top + chartH} x2={margin.left + chartW} y2={margin.top + chartH} stroke="#e5e7eb" strokeWidth={1} />

      {/* Y 轴标签 */}
      {[0, 0.25, 0.5, 0.75, 1].map((t, i) => {
        const y = margin.top + chartH - t * chartH;
        return (
          <g key={i}>
            <line x1={margin.left - 4} y1={y} x2={margin.left} y2={y} stroke="#9ca3af" strokeWidth={1} />
            <text x={margin.left - 8} y={y + 4} textAnchor="end" fontSize={10} fill="#6b7280">
              {(max * t).toFixed(1)}
            </text>
          </g>
        );
      })}

      {/* 柱子 */}
      {data.map((d, i) => {
        const h = (d.value / max) * chartH;
        const x = margin.left + barGap + i * (barW + barGap);
        const y = margin.top + chartH - h;
        const active = activeIndex === i;
        return (
          <g key={i} onMouseEnter={() => onHover(i)} onMouseLeave={() => onHover(null)}>
            <rect
              x={x}
              y={y}
              width={barW}
              height={h}
              rx={6}
              fill={COLORS[i % COLORS.length]}
              opacity={activeIndex === null || active ? 1 : 0.35}
              className="transition-opacity duration-150"
            />
            <text x={x + barW / 2} y={y - 6} textAnchor="middle" fontSize={11} fill="#374151">
              {d.value}
            </text>
            <text x={x + barW / 2} y={margin.top + chartH + 18} textAnchor="middle" fontSize={10} fill="#6b7280">
              {d.label}
            </text>
          </g>
        );
      })}

      {/* 轴标题 */}
      {xLabel && (
        <text x={margin.left + chartW / 2} y={height - 16} textAnchor="middle" fontSize={12} fill="#374151">
          {xLabel}
        </text>
      )}
      {yLabel && (
        <text x={16} y={margin.top + chartH / 2} textAnchor="middle" fontSize={12} fill="#374151" transform={`rotate(-90, 16, ${margin.top + chartH / 2})`}>
          {yLabel}
        </text>
      )}
    </svg>
  );
}

function LineChart({
  data,
  xLabel,
  yLabel,
  activeIndex,
  onHover,
}: {
  data: { label: string; value: number }[];
  xLabel?: string;
  yLabel?: string;
  activeIndex: number | null;
  onHover: (index: number | null) => void;
}) {
  const width = 500;
  const height = 280;
  const margin = { top: 24, right: 24, bottom: 64, left: 56 };
  const chartW = width - margin.left - margin.right;
  const chartH = height - margin.top - margin.bottom;
  const max = Math.max(...data.map((d) => d.value), 0.1);
  const stepX = data.length > 1 ? chartW / (data.length - 1) : chartW / 2;

  const points = data.map((d, i) => ({
    x: margin.left + (data.length > 1 ? i * stepX : chartW / 2),
    y: margin.top + chartH - (d.value / max) * chartH,
    label: d.label,
    value: d.value,
  }));

  const polylinePoints = points.map((p) => `${p.x},${p.y}`).join(" ");

  return (
    <svg viewBox={`0 0 ${width} ${height}`} className="w-full max-w-lg">
      <line x1={margin.left} y1={margin.top} x2={margin.left} y2={margin.top + chartH} stroke="#e5e7eb" strokeWidth={1} />
      <line x1={margin.left} y1={margin.top + chartH} x2={margin.left + chartW} y2={margin.top + chartH} stroke="#e5e7eb" strokeWidth={1} />

      {[0, 0.25, 0.5, 0.75, 1].map((t, i) => {
        const y = margin.top + chartH - t * chartH;
        return (
          <g key={i}>
            <line x1={margin.left - 4} y1={y} x2={margin.left} y2={y} stroke="#9ca3af" strokeWidth={1} />
            <text x={margin.left - 8} y={y + 4} textAnchor="end" fontSize={10} fill="#6b7280">
              {(max * t).toFixed(1)}
            </text>
          </g>
        );
      })}

      <polyline fill="none" stroke="#3b82f6" strokeWidth={2} points={polylinePoints} />
      {points.map((p, i) => (
        <g key={i} onMouseEnter={() => onHover(i)} onMouseLeave={() => onHover(null)}>
          <circle
            cx={p.x}
            cy={p.y}
            r={activeIndex === i ? 7 : 4}
            fill="#0ea5e9"
            stroke="#fff"
            strokeWidth={2}
            opacity={activeIndex === null || activeIndex === i ? 1 : 0.45}
            className="transition-all duration-150"
          />
          <text x={p.x} y={margin.top + chartH + 18} textAnchor="middle" fontSize={10} fill="#6b7280">
            {p.label}
          </text>
          <text x={p.x} y={p.y - 10} textAnchor="middle" fontSize={10} fill="#374151">
            {p.value}
          </text>
        </g>
      ))}

      {xLabel && (
        <text x={margin.left + chartW / 2} y={height - 16} textAnchor="middle" fontSize={12} fill="#374151">
          {xLabel}
        </text>
      )}
      {yLabel && (
        <text x={16} y={margin.top + chartH / 2} textAnchor="middle" fontSize={12} fill="#374151" transform={`rotate(-90, 16, ${margin.top + chartH / 2})`}>
          {yLabel}
        </text>
      )}
    </svg>
  );
}

function PieChart({
  data,
  activeIndex,
  onHover,
}: {
  data: { label: string; value: number }[];
  activeIndex: number | null;
  onHover: (index: number | null) => void;
}) {
  const width = 400;
  const height = 320;
  const cx = 140;
  const cy = 140;
  const radius = 120;
  const total = data.reduce((sum, d) => sum + d.value, 0);

  const slices = data.map((d, i) => {
    const prev = data.slice(0, i).reduce((sum, x) => sum + x.value, 0);
    const startAngle = (prev / total) * 2 * Math.PI;
    const angle = (d.value / total) * 2 * Math.PI;
    const endAngle = startAngle + angle;
    const x1 = radius * Math.cos(startAngle);
    const y1 = radius * Math.sin(startAngle);
    const x2 = radius * Math.cos(endAngle);
    const y2 = radius * Math.sin(endAngle);
    const largeArc = angle > Math.PI ? 1 : 0;
    const path = `M 0 0 L ${x1} ${y1} A ${radius} ${radius} 0 ${largeArc} 1 ${x2} ${y2} Z`;
    const midAngle = startAngle + angle / 2;
    const lx = (radius * 0.7) * Math.cos(midAngle);
    const ly = (radius * 0.7) * Math.sin(midAngle);
    return (
      <g key={i}>
        <path
          d={path}
          fill={COLORS[i % COLORS.length]}
          stroke="#fff"
          strokeWidth={2}
          opacity={activeIndex === null || activeIndex === i ? 1 : 0.35}
          transform={activeIndex === i ? `translate(${Math.cos(midAngle) * 6} ${Math.sin(midAngle) * 6})` : undefined}
          onMouseEnter={() => onHover(i)}
          onMouseLeave={() => onHover(null)}
          className="cursor-pointer transition-all duration-150"
        />
        <text x={lx} y={ly} textAnchor="middle" dominantBaseline="middle" fontSize={11} fill="#fff" fontWeight={600}>
          {((d.value / total) * 100).toFixed(0)}%
        </text>
      </g>
    );
  });

  return (
    <svg viewBox={`0 0 ${width} ${height}`} className="w-full max-w-md">
      <g transform={`translate(${cx}, ${cy})`}>
        {slices}
      </g>
      {/* 图例 */}
      <g transform={`translate(${cx + radius + 32}, 40)`}>
        {data.map((d, i) => (
          <g key={i} transform={`translate(0, ${i * 24})`}>
            <rect width={14} height={14} rx={3} fill={COLORS[i % COLORS.length]} />
            <text x={22} y={12} fontSize={12} fill="#374151">
              {d.label}: {d.value}
            </text>
          </g>
        ))}
      </g>
    </svg>
  );
}

export default function ChartBlock({ block }: Props) {
  const data = block.data || [];
  const [activeIndex, setActiveIndex] = useState<number | null>(null);
  const activePoint = activeIndex === null ? null : data[activeIndex];
  const total = useMemo(() => data.reduce((sum, point) => sum + point.value, 0), [data]);
  if (data.length === 0) return null;

  return (
    <figure className="reader-panel my-6 overflow-hidden p-0">
      <div className="flex flex-wrap items-center justify-between gap-3 border-b border-slate-200 px-4 py-3">
        <figcaption className="text-base font-semibold text-slate-800">
          {block.title ?? "数据图表"}
        </figcaption>
        <div className="rounded-full bg-slate-100 px-2.5 py-1 text-xs font-medium text-slate-500">
          {block.chart_type}
        </div>
      </div>
      <div className="grid gap-0 md:grid-cols-[minmax(0,1fr)_180px]">
        <div className="flex justify-center overflow-x-auto p-4">
        {block.chart_type === "line" ? (
          <LineChart
            data={data}
            xLabel={block.x_label}
            yLabel={block.y_label}
            activeIndex={activeIndex}
            onHover={setActiveIndex}
          />
        ) : block.chart_type === "pie" ? (
          <PieChart data={data} activeIndex={activeIndex} onHover={setActiveIndex} />
        ) : (
          <BarChart
            data={data}
            xLabel={block.x_label}
            yLabel={block.y_label}
            activeIndex={activeIndex}
            onHover={setActiveIndex}
          />
        )}
        </div>
        <div className="border-t border-slate-200 bg-slate-50 p-4 md:border-l md:border-t-0">
          <div className="mb-3 text-xs font-semibold uppercase text-slate-400">
            数据点
          </div>
          <div className="space-y-2">
            {data.map((point, index) => (
              <button
                key={`${point.label}-${index}`}
                onMouseEnter={() => setActiveIndex(index)}
                onMouseLeave={() => setActiveIndex(null)}
                className={cn(
                  "flex w-full items-center justify-between gap-2 rounded-md px-2 py-1.5 text-left text-xs transition",
                  activeIndex === index ? "bg-white shadow-sm" : "hover:bg-white/70",
                )}
              >
                <span className="flex min-w-0 items-center gap-2 text-slate-600">
                  <span
                    className="h-2.5 w-2.5 shrink-0 rounded-full"
                    style={{ backgroundColor: COLORS[index % COLORS.length] }}
                  />
                  <span className="truncate">{point.label}</span>
                </span>
                <span className="font-mono font-semibold text-slate-800">
                  {point.value}
                </span>
              </button>
            ))}
          </div>
          {activePoint && total > 0 && (
            <div className="mt-4 rounded-lg border border-slate-200 bg-white p-3 text-xs text-slate-600">
              <div className="font-semibold text-slate-900">{activePoint.label}</div>
              <div className="mt-1">
                占比 {((activePoint.value / total) * 100).toFixed(1)}%
              </div>
            </div>
          )}
        </div>
      </div>
    </figure>
  );
}
