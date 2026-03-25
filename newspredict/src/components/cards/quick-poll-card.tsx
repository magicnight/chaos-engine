'use client';

interface QuickPollCardProps {
  question: string;
  yesPrice: number;
  noPrice: number;
  traderCount: string;
  onVote?: (side: 'YES' | 'NO') => void;
}

export function QuickPollCard({
  question,
  yesPrice,
  noPrice,
  traderCount,
  onVote,
}: QuickPollCardProps) {
  const yesPercent = Math.round(yesPrice * 100);
  return (
    <div className="card p-4 animate-fade-in">
      <div className="flex items-center gap-2 mb-2">
        <span className="badge badge-new">Quick Predict</span>
        <span className="text-[10px] text-[var(--muted)] font-mono">{traderCount} traders</span>
      </div>
      <p className="text-sm font-semibold mb-3 leading-tight">{question}</p>
      <div className="flex gap-2 mb-3">
        <button
          onClick={() => onVote?.('YES')}
          className="flex-1 py-2.5 rounded-lg bg-[var(--success-dim)] text-[var(--success)] text-sm font-bold hover:bg-[var(--success)]/25 active:scale-[0.98] transition-all"
        >
          YES ${yesPrice.toFixed(2)}
        </button>
        <button
          onClick={() => onVote?.('NO')}
          className="flex-1 py-2.5 rounded-lg bg-[var(--danger-dim)] text-[var(--danger)] text-sm font-bold hover:bg-[var(--danger)]/25 active:scale-[0.98] transition-all"
        >
          NO ${noPrice.toFixed(2)}
        </button>
      </div>
      <div className="pct-bar">
        <div className="pct-bar-fill bg-[var(--success)]" style={{ width: `${yesPercent}%` }} />
      </div>
    </div>
  );
}
