interface DonutGaugeProps {
  yesPercent: number;
  size?: number;
}

export function DonutGauge({ yesPercent, size = 80 }: DonutGaugeProps) {
  const r = (size - 8) / 2;
  const c = size / 2;
  const circumference = 2 * Math.PI * r;
  const yesLen = (yesPercent / 100) * circumference;
  const noLen = circumference - yesLen;
  const glowColor = yesPercent >= 50 ? 'var(--success)' : 'var(--danger)';

  return (
    <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} style={{ filter: `drop-shadow(0 0 6px ${glowColor}40)` }}>
      <circle
        cx={c}
        cy={c}
        r={r}
        fill="none"
        stroke="var(--danger)"
        strokeWidth="6"
        strokeDasharray={`${noLen} ${yesLen}`}
        strokeDashoffset={circumference / 4}
        strokeLinecap="round"
        transform={`rotate(-90 ${c} ${c})`}
        className="animate-donut-draw"
        style={{ strokeDashoffset: circumference / 4 }}
      />
      <circle
        cx={c}
        cy={c}
        r={r}
        fill="none"
        stroke="var(--success)"
        strokeWidth="6"
        strokeDasharray={`${yesLen} ${noLen}`}
        strokeDashoffset={circumference / 4}
        strokeLinecap="round"
        transform={`rotate(-90 ${c} ${c})`}
        className="animate-donut-draw"
        style={{ strokeDashoffset: circumference / 4 }}
      />
      <text
        x={c}
        y={c - 4}
        textAnchor="middle"
        fontSize="14"
        fontWeight="bold"
        fill="var(--foreground)"
      >
        {yesPercent}%
      </text>
      <text
        x={c}
        y={c + 10}
        textAnchor="middle"
        fontSize="9"
        fill="var(--muted)"
      >
        YES
      </text>
    </svg>
  );
}
