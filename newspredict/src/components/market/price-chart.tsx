'use client';

interface PricePoint {
  price: number;
  time: string;
}

interface PriceChartProps {
  data: PricePoint[];
  height?: number;
}

export function PriceChart({ data, height = 120 }: PriceChartProps) {
  if (data.length < 2) {
    return (
      <div
        className="flex items-center justify-center text-xs text-[var(--muted)] bg-[var(--card)] rounded-xl"
        style={{ height }}
      >
        Not enough data for chart
      </div>
    );
  }

  const padding = { top: 8, right: 8, bottom: 20, left: 8 };
  const width = 320;
  const innerW = width - padding.left - padding.right;
  const innerH = height - padding.top - padding.bottom;

  const prices = data.map((d: any) => d.price);
  const minP = Math.max(0, Math.min(...prices) - 0.05);
  const maxP = Math.min(1, Math.max(...prices) + 0.05);
  const range = maxP - minP || 0.1;

  const points = data.map((d, i) => {
    const x = padding.left + (i / (data.length - 1)) * innerW;
    const y = padding.top + innerH - ((d.price - minP) / range) * innerH;
    return { x, y };
  });

  const linePath = points.map((p, i) => `${i === 0 ? 'M' : 'L'}${p.x},${p.y}`).join(' ');

  const areaPath =
    linePath +
    ` L${points[points.length - 1].x},${padding.top + innerH}` +
    ` L${points[0].x},${padding.top + innerH} Z`;

  const lastPrice = prices[prices.length - 1];
  const firstPrice = prices[0];
  const isUp = lastPrice >= firstPrice;
  const color = isUp ? 'var(--success)' : 'var(--danger)';

  return (
    <div className="bg-[var(--card)] rounded-xl p-3">
      <svg viewBox={`0 0 ${width} ${height}`} className="w-full" style={{ height }} aria-label="Price history chart" role="img">
        <defs>
          <linearGradient id="chartGrad" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor={color} stopOpacity="0.3" />
            <stop offset="100%" stopColor={color} stopOpacity="0" />
          </linearGradient>
        </defs>
        <path d={areaPath} fill="url(#chartGrad)" />
        <path d={linePath} fill="none" stroke={color} strokeWidth="2" strokeLinecap="round" />
        <circle cx={points[points.length - 1].x} cy={points[points.length - 1].y} r="3" fill={color} />
        <text
          x={padding.left}
          y={height - 4}
          fontSize="9"
          fill="var(--muted)"
        >
          {data[0].time}
        </text>
        <text
          x={width - padding.right}
          y={height - 4}
          fontSize="9"
          fill="var(--muted)"
          textAnchor="end"
        >
          {data[data.length - 1].time}
        </text>
      </svg>
    </div>
  );
}
