'use client';

import Link from 'next/link';
import { useLocale } from '@/lib/i18n/context';

interface BreakingBannerProps {
  headline: string;
  marketUrl?: string;
}

export function BreakingBanner({ headline, marketUrl }: BreakingBannerProps) {
  const { t } = useLocale();
  return (
    <div className="rounded-xl bg-[var(--card)] border border-[var(--danger)]/20 px-4 py-3 flex items-center justify-between gap-3 animate-fade-in">
      <div className="min-w-0">
        <div className="flex items-center gap-2 mb-1">
          <span className="w-1.5 h-1.5 rounded-full bg-[var(--danger)] animate-pulse" />
          <span className="text-[10px] text-[var(--danger)] font-bold uppercase tracking-wider">
            {t('breaking.prefix')}
          </span>
        </div>
        <p className="text-sm font-medium leading-tight line-clamp-2">{headline}</p>
      </div>
      {marketUrl && (
        <Link
          href={marketUrl}
          className="shrink-0 px-3 py-1.5 rounded-full bg-[var(--accent-glow)] text-[var(--accent)] text-xs font-semibold hover:bg-[var(--accent)]/20 transition-colors"
        >
          Trade &rarr;
        </Link>
      )}
    </div>
  );
}
