'use client';

import Link from 'next/link';

interface BreakingBannerProps {
  headline: string;
  marketUrl?: string;
}

export function BreakingBanner({ headline, marketUrl }: BreakingBannerProps) {
  return (
    <div className="rounded-xl bg-[var(--card)] border-l-4 border-[var(--accent)] px-4 py-3 flex items-center justify-between gap-3">
      <div className="min-w-0">
        <p className="text-[10px] text-[var(--accent)] font-bold uppercase tracking-wider mb-0.5">
          Breaking
        </p>
        <p className="text-sm font-medium truncate">{headline}</p>
      </div>
      {marketUrl && (
        <Link
          href={marketUrl}
          className="shrink-0 text-xs text-[var(--accent)] font-medium"
        >
          New Market &rarr;
        </Link>
      )}
    </div>
  );
}
