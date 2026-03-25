'use client';

import { useLocale } from '@/lib/i18n/context';

interface SummaryBarProps {
  totalPnl: number;
  activePositions: number;
  winRate: number;
}

export function SummaryBar({ totalPnl, activePositions, winRate }: SummaryBarProps) {
  const { t } = useLocale();
  const isPositive = totalPnl >= 0;
  return (
    <div className="mx-4 my-2 flex items-center justify-between glass rounded-xl px-4 py-3 border border-[var(--border-subtle)]">
      <div className="flex items-center gap-1.5">
        <span className="text-[10px] text-[var(--muted)] uppercase tracking-wide">{t('summary.pnl')}</span>
        <span
          className={`text-sm font-bold font-mono ${
            isPositive ? 'text-[var(--success)]' : 'text-[var(--danger)]'
          }`}
        >
          {isPositive ? '+' : '-'}${Math.abs(totalPnl).toFixed(0)}
          <svg className="inline ml-0.5 w-3 h-3" viewBox="0 0 12 12" fill="currentColor">
            {isPositive
              ? <path d="M6 2L10 8H2L6 2Z" />
              : <path d="M6 10L2 4H10L6 10Z" />
            }
          </svg>
        </span>
      </div>
      <div className="h-4 w-px bg-[var(--border)]" />
      <div className="text-center">
        <span className="text-sm font-bold text-[var(--foreground)] font-mono">{activePositions}</span>
        <span className="text-[10px] text-[var(--muted)] ml-1">{t('summary.active')}</span>
      </div>
      <div className="h-4 w-px bg-[var(--border)]" />
      <div className="text-center">
        <span className="text-sm font-bold text-[var(--foreground)] font-mono">{winRate}%</span>
        <span className="text-[10px] text-[var(--muted)] ml-1">{t('summary.winRate')}</span>
      </div>
    </div>
  );
}
