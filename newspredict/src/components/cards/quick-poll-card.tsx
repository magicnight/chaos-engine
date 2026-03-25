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
  return (
    <div className="bg-[var(--card)] rounded-xl p-4">
      <p className="text-xs text-[var(--accent)] font-medium mb-1">Quick Predict</p>
      <p className="text-sm font-semibold mb-3">{question}</p>
      <div className="flex gap-2 mb-2">
        <button
          onClick={() => onVote?.('YES')}
          className="flex-1 py-2.5 rounded-lg bg-[var(--success)]/15 text-[var(--success)] text-sm font-semibold hover:bg-[var(--success)]/25 transition-colors"
        >
          YES ${yesPrice.toFixed(2)}
        </button>
        <button
          onClick={() => onVote?.('NO')}
          className="flex-1 py-2.5 rounded-lg bg-[var(--danger)]/15 text-[var(--danger)] text-sm font-semibold hover:bg-[var(--danger)]/25 transition-colors"
        >
          NO ${noPrice.toFixed(2)}
        </button>
      </div>
      <p className="text-xs text-[var(--muted)] text-center">{traderCount} traders</p>
    </div>
  );
}
