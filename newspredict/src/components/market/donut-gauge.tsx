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

  return (
    <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`}>
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
