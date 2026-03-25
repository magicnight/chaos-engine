'use client';

interface MarketMiniCardProps {
  label: string;
  yesPercent: number;
  price: string;
  change: string;
  icon?: string;
}

export function MarketMiniCard({
  label,
  yesPercent,
  price,
  change,
  icon,
}: MarketMiniCardProps) {
  const noPercent = 100 - yesPercent;
  const isPositive = change.startsWith('+');

  return (
    <div className="w-[130px] shrink-0 bg-[var(--card)] rounded-xl p-3">
      <p className="text-xs text-[var(--muted)] mb-2 truncate">
        {icon && <span className="mr-1">{icon}</span>}
        {label}
      </p>
      <div className="relative w-14 h-14 mx-auto mb-2">
        <svg viewBox="0 0 36 36" className="w-full h-full -rotate-90">
          <circle
            cx="18"
            cy="18"
            r="15.5"
            fill="none"
            stroke="var(--border)"
            strokeWidth="3"
          />
          <circle
            cx="18"
            cy="18"
            r="15.5"
            fill="none"
            stroke="var(--accent)"
            strokeWidth="3"
            strokeDasharray={`${yesPercent} ${noPercent}`}
            strokeLinecap="round"
          />
        </svg>
        <span className="absolute inset-0 flex items-center justify-center text-[10px] font-bold">
          {yesPercent}/{noPercent}
        </span>
      </div>
      <div className="flex items-center justify-between text-xs">
        <span className="font-semibold">${price}</span>
        <span className={isPositive ? 'text-[var(--success)]' : 'text-[var(--danger)]'}>
          {change}
        </span>
      </div>
    </div>
  );
}
