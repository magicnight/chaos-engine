'use client';

interface SummaryBarProps {
  totalPnl: number;
  activePositions: number;
  winRate: number;
}

export function SummaryBar({ totalPnl, activePositions, winRate }: SummaryBarProps) {
  const isPositive = totalPnl >= 0;
  return (
    <div className="mx-4 my-2 flex items-center justify-between rounded-xl bg-[var(--card)] px-4 py-3">
      <div className="flex items-center gap-1.5">
        <span className="text-xs text-[var(--muted)]">Portfolio</span>
        <span
          className={`text-sm font-bold ${
            isPositive ? 'text-[var(--success)]' : 'text-[var(--danger)]'
          }`}
        >
          {isPositive ? '+' : '-'}${Math.abs(totalPnl)}
          <span className="ml-0.5 text-[10px]">{isPositive ? '\u2191' : '\u2193'}</span>
        </span>
      </div>
      <div className="h-4 w-px bg-[var(--border)]" />
      <span className="text-xs text-[var(--muted)]">
        <span className="text-[var(--foreground)] font-medium">{activePositions}</span> Active
      </span>
      <div className="h-4 w-px bg-[var(--border)]" />
      <span className="text-xs text-[var(--muted)]">
        Win <span className="text-[var(--foreground)] font-medium">{winRate}%</span>
      </span>
    </div>
  );
}
