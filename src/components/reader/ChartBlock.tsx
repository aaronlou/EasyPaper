import { useMemo, useState } from "react";
import type { ChartBlock as ChartBlockType } from "@/types";
import { cn } from "@/lib/cn";

interface Props {
  block: ChartBlockType;
}

const COLORS = ["#0284c7", "#0f766e", "#d97706", "#dc2626", "#4f46e5", "#65a30d"];

function formatNumber(value: number) {
  return Number.isInteger(value) ? String(value) : value.toFixed(2);
}

function BarChart({
  data,
  activeIndex,
  onHover,
}: {
  data: { label: string; value: number }[];
  activeIndex: number | null;
  onHover: (index: number | null) => void;
}) {
  const width = 560;
  const height = 300;
  const margin = { top: 28, right: 24, bottom: 38, left: 56 };
  const chartW = width - margin.left - margin.right;
  const chartH = height - margin.top - margin.bottom;
  const max = Math.max(...data.map((d) => d.value), 0.1);
  const barGap = Math.max(10, 24 - data.length * 2);
  const barW = Math.max(26, (chartW - barGap * (data.length + 1)) / data.length);

  return (
    <svg viewBox={`0 0 ${width} ${height}`} className="h-auto w-full min-w-[520px]">
      {[0, 0.25, 0.5, 0.75, 1].map((t) => {
        const y = margin.top + chartH - t * chartH;
        return (
          <g key={t}>
            <line
              x1={margin.left}
              y1={y}
              x2={margin.left + chartW}
              y2={y}
              stroke="#e2e8f0"
              strokeWidth={1}
            />
            <text x={margin.left - 10} y={y + 4} textAnchor="end" fontSize={11} fill="#64748b">
              {formatNumber(max * t)}
            </text>
          </g>
        );
      })}

      {data.map((d, i) => {
        const h = (d.value / max) * chartH;
        const x = margin.left + barGap + i * (barW + barGap);
        const y = margin.top + chartH - h;
        const active = activeIndex === i;
        return (
          <g key={`${d.label}-${i}`} onMouseEnter={() => onHover(i)} onMouseLeave={() => onHover(null)}>
            <rect
              x={x}
              y={y}
              width={barW}
              height={Math.max(h, 2)}
              rx={7}
              fill={COLORS[i % COLORS.length]}
              opacity={activeIndex === null || active ? 1 : 0.32}
              className="transition-opacity duration-150"
            />
            <text x={x + barW / 2} y={Math.max(y - 8, 14)} textAnchor="middle" fontSize={11} fill="#334155">
              {formatNumber(d.value)}
            </text>
            <text x={x + barW / 2} y={margin.top + chartH + 24} textAnchor="middle" fontSize={11} fill="#64748b">
              {i + 1}
            </text>
          </g>
        );
      })}
    </svg>
  );
}

function LineChart({
  data,
  activeIndex,
  onHover,
}: {
  data: { label: string; value: number }[];
  activeIndex: number | null;
  onHover: (index: number | null) => void;
}) {
  const width = 560;
  const height = 300;
  const margin = { top: 28, right: 28, bottom: 38, left: 56 };
  const chartW = width - margin.left - margin.right;
  const chartH = height - margin.top - margin.bottom;
  const max = Math.max(...data.map((d) => d.value), 0.1);
  const stepX = data.length > 1 ? chartW / (data.length - 1) : chartW / 2;

  const points = data.map((d, i) => ({
    x: margin.left + (data.length > 1 ? i * stepX : chartW / 2),
    y: margin.top + chartH - (d.value / max) * chartH,
    value: d.value,
    label: d.label,
  }));

  return (
    <svg viewBox={`0 0 ${width} ${height}`} className="h-auto w-full min-w-[520px]">
      {[0, 0.25, 0.5, 0.75, 1].map((t) => {
        const y = margin.top + chartH - t * chartH;
        return (
          <g key={t}>
            <line
              x1={margin.left}
              y1={y}
              x2={margin.left + chartW}
              y2={y}
              stroke="#e2e8f0"
              strokeWidth={1}
            />
            <text x={margin.left - 10} y={y + 4} textAnchor="end" fontSize={11} fill="#64748b">
              {formatNumber(max * t)}
            </text>
          </g>
        );
      })}
      <polyline
        fill="none"
        stroke="#0284c7"
        strokeWidth={2.5}
        strokeLinecap="round"
        strokeLinejoin="round"
        points={points.map((p) => `${p.x},${p.y}`).join(" ")}
      />
      {points.map((point, i) => {
        const active = activeIndex === i;
        return (
          <g key={`${point.label}-${i}`} onMouseEnter={() => onHover(i)} onMouseLeave={() => onHover(null)}>
            <circle
              cx={point.x}
              cy={point.y}
              r={active ? 7 : 4.5}
              fill={COLORS[i % COLORS.length]}
              stroke="#ffffff"
              strokeWidth={2}
              opacity={activeIndex === null || active ? 1 : 0.38}
              className="transition-all duration-150"
            />
            <text x={point.x} y={margin.top + chartH + 24} textAnchor="middle" fontSize={11} fill="#64748b">
              {i + 1}
            </text>
          </g>
        );
      })}
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
  const width = 300;
  const height = 300;
  const cx = 150;
  const cy = 150;
  const radius = 118;
  const total = data.reduce((sum, d) => sum + Math.max(d.value, 0), 0);

  if (total <= 0) return null;

  return (
    <svg viewBox={`0 0 ${width} ${height}`} className="h-auto w-full max-w-[320px]">
      <g transform={`translate(${cx}, ${cy})`}>
        {data.map((d, i) => {
          const prev = data.slice(0, i).reduce((sum, x) => sum + Math.max(x.value, 0), 0);
          const startAngle = (prev / total) * 2 * Math.PI - Math.PI / 2;
          const angle = (Math.max(d.value, 0) / total) * 2 * Math.PI;
          const endAngle = startAngle + angle;
          const x1 = radius * Math.cos(startAngle);
          const y1 = radius * Math.sin(startAngle);
          const x2 = radius * Math.cos(endAngle);
          const y2 = radius * Math.sin(endAngle);
          const largeArc = angle > Math.PI ? 1 : 0;
          const midAngle = startAngle + angle / 2;
          const active = activeIndex === i;
          return (
            <path
              key={`${d.label}-${i}`}
              d={`M 0 0 L ${x1} ${y1} A ${radius} ${radius} 0 ${largeArc} 1 ${x2} ${y2} Z`}
              fill={COLORS[i % COLORS.length]}
              stroke="#fff"
              strokeWidth={2}
              opacity={activeIndex === null || active ? 1 : 0.32}
              transform={active ? `translate(${Math.cos(midAngle) * 7} ${Math.sin(midAngle) * 7})` : undefined}
              onMouseEnter={() => onHover(i)}
              onMouseLeave={() => onHover(null)}
              className="cursor-pointer transition-all duration-150"
            />
          );
        })}
      </g>
    </svg>
  );
}

export default function ChartBlock({ block }: Props) {
  const data = block.data || [];
  const [activeIndex, setActiveIndex] = useState<number | null>(null);
  const total = useMemo(() => data.reduce((sum, point) => sum + point.value, 0), [data]);
  const max = useMemo(() => Math.max(...data.map((point) => point.value), 0.1), [data]);
  const activePoint = activeIndex === null ? null : data[activeIndex];

  if (data.length === 0) return null;

  return (
    <figure className="reader-panel my-6 overflow-hidden p-0">
      <div className="flex flex-wrap items-center justify-between gap-3 border-b border-slate-200 bg-slate-50 px-4 py-3">
        <figcaption className="text-base font-semibold text-slate-800">
          {block.title ?? "数据图表"}
        </figcaption>
        <div className="flex flex-wrap items-center gap-2 text-xs text-slate-500">
          {block.x_label && <span>{block.x_label}</span>}
          {block.y_label && <span className="rounded-full bg-white px-2 py-1">{block.y_label}</span>}
          <span className="rounded-full bg-slate-200/70 px-2 py-1 font-medium">
            {block.chart_type}
          </span>
        </div>
      </div>

      <div className="grid gap-0 lg:grid-cols-[minmax(0,1fr)_minmax(260px,0.42fr)]">
        <div className="min-w-0 bg-white p-4">
          <div className="md:hidden">
            <CompactValueBars
              data={data}
              max={max}
              activeIndex={activeIndex}
              onHover={setActiveIndex}
            />
          </div>
          <div className="hidden min-h-[320px] items-center justify-center overflow-x-auto md:flex">
            {block.chart_type === "line" ? (
              <LineChart data={data} activeIndex={activeIndex} onHover={setActiveIndex} />
            ) : block.chart_type === "pie" ? (
              <PieChart data={data} activeIndex={activeIndex} onHover={setActiveIndex} />
            ) : (
              <BarChart data={data} activeIndex={activeIndex} onHover={setActiveIndex} />
            )}
          </div>
        </div>

        <aside className="hidden border-t border-slate-200 bg-slate-50 p-4 md:block lg:border-l lg:border-t-0">
          <div className="mb-3 flex items-center justify-between gap-3">
            <div className="text-xs font-semibold uppercase text-slate-400">
              数据点
            </div>
            <div className="text-xs text-slate-500">max {formatNumber(max)}</div>
          </div>

          <div className="max-h-[360px] space-y-2 overflow-y-auto pr-1">
            {data.map((point, index) => {
              const selected = activeIndex === index;
              const percent = total > 0 ? (point.value / total) * 100 : 0;
              return (
                <button
                  key={`${point.label}-${index}`}
                  type="button"
                  onMouseEnter={() => setActiveIndex(index)}
                  onMouseLeave={() => setActiveIndex(null)}
                  onFocus={() => setActiveIndex(index)}
                  onBlur={() => setActiveIndex(null)}
                  className={cn(
                    "grid w-full grid-cols-[28px_minmax(0,1fr)_auto] gap-2 rounded-lg border p-2.5 text-left text-xs transition",
                    selected
                      ? "border-sky-200 bg-white shadow-sm"
                      : "border-transparent bg-white/55 hover:border-slate-200 hover:bg-white",
                  )}
                >
                  <span
                    className="flex h-6 w-6 items-center justify-center rounded-full text-[11px] font-semibold text-white"
                    style={{ backgroundColor: COLORS[index % COLORS.length] }}
                  >
                    {index + 1}
                  </span>
                  <span className="min-w-0">
                    <span className="block break-words leading-5 text-slate-700">
                      {point.label}
                    </span>
                    {total > 0 && (
                      <span className="mt-1 block text-[11px] text-slate-400">
                        {percent.toFixed(1)}%
                      </span>
                    )}
                  </span>
                  <span className="font-mono text-sm font-semibold text-slate-900">
                    {formatNumber(point.value)}
                  </span>
                </button>
              );
            })}
          </div>

          {activePoint && total > 0 && (
            <div className="mt-4 rounded-lg border border-slate-200 bg-white p-3 text-xs text-slate-600">
              <div className="font-semibold leading-5 text-slate-900">
                {activePoint.label}
              </div>
              <div className="mt-1">
                当前占比 {((activePoint.value / total) * 100).toFixed(1)}%
              </div>
            </div>
          )}
        </aside>
      </div>
    </figure>
  );
}

function CompactValueBars({
  data,
  max,
  activeIndex,
  onHover,
}: {
  data: { label: string; value: number }[];
  max: number;
  activeIndex: number | null;
  onHover: (index: number | null) => void;
}) {
  return (
    <div className="space-y-3">
      {data.map((point, index) => {
        const selected = activeIndex === index;
        const width = `${Math.max(4, (point.value / max) * 100)}%`;
        return (
          <button
            key={`${point.label}-${index}`}
            type="button"
            onMouseEnter={() => onHover(index)}
            onMouseLeave={() => onHover(null)}
            onFocus={() => onHover(index)}
            onBlur={() => onHover(null)}
            className={cn(
              "w-full rounded-lg border p-3 text-left transition",
              selected
                ? "border-sky-200 bg-sky-50"
                : "border-slate-200 bg-white hover:border-sky-200",
            )}
          >
            <div className="flex items-start justify-between gap-3">
              <div className="min-w-0">
                <div className="flex items-center gap-2">
                  <span
                    className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full text-[11px] font-semibold text-white"
                    style={{ backgroundColor: COLORS[index % COLORS.length] }}
                  >
                    {index + 1}
                  </span>
                  <span className="break-words text-sm font-medium leading-5 text-slate-700">
                    {point.label}
                  </span>
                </div>
              </div>
              <span className="font-mono text-sm font-semibold text-slate-900">
                {formatNumber(point.value)}
              </span>
            </div>
            <div className="mt-3 h-2 overflow-hidden rounded-full bg-slate-100">
              <div
                className="h-full rounded-full transition-all"
                style={{
                  width,
                  backgroundColor: COLORS[index % COLORS.length],
                }}
              />
            </div>
          </button>
        );
      })}
    </div>
  );
}
