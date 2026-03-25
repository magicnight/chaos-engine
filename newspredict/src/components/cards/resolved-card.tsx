'use client';

interface ResolvedCardProps {
  title: string;
  result: 'win' | 'loss';
  amount: string;
}

export function ResolvedCard({ title, result, amount }: ResolvedCardProps) {
  const isWin = result === 'win';
  return (
    <div
      className={`flex items-center gap-3 rounded-xl px-4 py-3 animate-fade-in ${
        isWin
          ? 'bg-[var(--success-dim)] border border-[var(--success)]/20'
          : 'bg-[var(--danger-dim)] border border-[var(--danger)]/20'
      }`}
    >
      <div className={`w-8 h-8 rounded-full flex items-center justify-center shrink-0 ${
        isWin ? 'bg-[var(--success)]/20' : 'bg-[var(--danger)]/20'
      }`}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"
          className={isWin ? 'text-[var(--success)]' : 'text-[var(--danger)]'}
        >
          {isWin
            ? <polyline points="20 6 9 17 4 12" />
            : <><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></>
          }
        </svg>
      </div>
      <span className="flex-1 text-sm font-medium truncate">{title}</span>
      <span
        className={`text-sm font-bold font-mono ${
          isWin ? 'text-[var(--success)]' : 'text-[var(--danger)]'
        }`}
      >
        {isWin ? '+' : '-'}${amount}
      </span>
    </div>
  );
}
