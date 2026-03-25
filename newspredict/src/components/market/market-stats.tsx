'use client';

import { useLocale } from '@/lib/i18n/context';

interface MarketStatsProps {
  volume: number;
  traderCount: number;
  closeAt: string;
  resolutionCriteria: string;
  resolutionSource?: string | null;
}

function formatVolume(v: number): string {
  if (v >= 1_000_000) return `$${(v / 1_000_000).toFixed(1)}M`;
  if (v >= 1_000) return `$${(v / 1_000).toFixed(1)}K`;
  return `$${v.toFixed(0)}`;
}

export function MarketStats({
  volume,
  traderCount,
  closeAt,
  resolutionCriteria,
  resolutionSource,
}: MarketStatsProps) {
  const { t, locale } = useLocale();

  const formattedDate = new Date(closeAt).toLocaleDateString(
    locale === 'zh' ? 'zh-CN' : 'en-US',
    { month: 'short', day: 'numeric', year: 'numeric' }
  );

  return (
    <div className="bg-[var(--card)] rounded-xl p-4 space-y-3">
      <div className="flex justify-between text-sm">
        <span className="text-[var(--muted)]">{t('common.volume')}</span>
        <span className="font-semibold">{formatVolume(volume)}</span>
      </div>
      <div className="flex justify-between text-sm">
        <span className="text-[var(--muted)]">{t('common.traders')}</span>
        <span className="font-semibold">{traderCount.toLocaleString()}</span>
      </div>
      <div className="flex justify-between text-sm">
        <span className="text-[var(--muted)]">{t('market.closesAt')}</span>
        <span className="font-semibold">{formattedDate}</span>
      </div>
      <div className="border-t border-[var(--border)] pt-3">
        <p className="text-xs text-[var(--muted)] mb-1">{t('market.resolutionCriteria')}</p>
        <p className="text-sm">{resolutionCriteria}</p>
        {resolutionSource && (
          <p className="text-xs text-[var(--accent)] mt-1">{t('market.resolutionSource')}: {resolutionSource}</p>
        )}
      </div>
    </div>
  );
}
