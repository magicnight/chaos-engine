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
      className={`flex items-center gap-3 rounded-xl px-4 py-3 ${
        isWin
          ? 'bg-[var(--success)]/10 border border-[var(--success)]/20'
          : 'bg-[var(--danger)]/10 border border-[var(--danger)]/20'
      }`}
    >
      <span className={`text-lg ${isWin ? 'text-[var(--success)]' : 'text-[var(--danger)]'}`}>
        {isWin ? 'v' : 'x'}
      </span>
      <span className="flex-1 text-sm font-medium truncate">{title}</span>
      <span
        className={`text-sm font-bold ${
          isWin ? 'text-[var(--success)]' : 'text-[var(--danger)]'
        }`}
      >
        {isWin ? '+' : '-'}${amount}
      </span>
    </div>
  );
}
