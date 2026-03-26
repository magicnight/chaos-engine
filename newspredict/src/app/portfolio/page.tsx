'use client';

import { useEffect, useState } from 'react';
import { SummaryBar } from '@/components/portfolio/summary-bar';
import { PositionList } from '@/components/portfolio/position-list';
import { useLocale } from '@/lib/i18n/context';

interface PositionData {
  id: string;
  marketId: string;
  side: string;
  shares: number;
  avgPrice: number;
  currentPrice: number;
  unrealizedPnl: number;
  realizedPnl: number;
  market: {
    id: string;
    question: string;
    status: string;
    category: string;
    resolutionResult?: string | null;
  };
}

interface PortfolioData {
  balance: number;
  totalPnl: number;
  positions: PositionData[];
  activeCount: number;
  winRate: number;
}

export default function PortfolioPage() {
  const [data, setData] = useState<PortfolioData | null>(null);
  const [loading, setLoading] = useState(true);
  const { t } = useLocale();

  useEffect(() => {
    fetch('/api/portfolio')
      .then((r) => r.json())
      .then((d) => setData(d))
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  if (loading) {
    return (
      <div className="p-4">
        <h1 className="text-2xl font-bold mb-4">{t('portfolio.title')}</h1>
        <div className="space-y-3">
          {[1, 2, 3].map((i: any) => (
            <div key={i} className="h-20 rounded-xl bg-[var(--card)] animate-pulse" />
          ))}
        </div>
      </div>
    );
  }

  if (!data || data.positions === undefined) {
    return (
      <div className="p-4">
        <h1 className="text-2xl font-bold mb-4">{t('portfolio.title')}</h1>
        <p className="text-[var(--muted)] text-center py-8">
          {t('portfolio.signInToView')}
        </p>
      </div>
    );
  }

  const resolvedCount = data.positions.filter((p: any) => p.market.status === 'resolved').length;

  return (
    <div>
      <div className="p-4 pb-2">
        <h1 className="text-2xl font-bold">{t('portfolio.title')}</h1>
      </div>

      <SummaryBar
        totalPnl={data.totalPnl}
        activePositions={data.activeCount}
        winRate={data.winRate}
      />

      <div className="px-4 py-2">
        <div className="flex items-center justify-between rounded-xl bg-[var(--card)] p-5 border border-[var(--border-subtle)]">
          <div>
            <p className="text-[10px] text-[var(--muted)] uppercase tracking-wide mb-1">{t('portfolio.balance')}</p>
            <p className="text-2xl font-bold font-mono">${data.balance.toFixed(2)}</p>
          </div>
          <div className="text-right">
            <p className="text-[10px] text-[var(--muted)] uppercase tracking-wide mb-1">{t('portfolio.unrealizedPnl')}</p>
            <div className="flex items-center justify-end gap-1.5">
              <svg className={`w-4 h-4 ${data.totalPnl >= 0 ? 'text-[var(--success)]' : 'text-[var(--danger)]'}`} viewBox="0 0 12 12" fill="currentColor">
                {data.totalPnl >= 0
                  ? <path d="M6 2L10 8H2L6 2Z" />
                  : <path d="M6 10L2 4H10L6 10Z" />
                }
              </svg>
              <p
                className={`text-2xl font-bold font-mono ${
                  data.totalPnl >= 0 ? 'text-[var(--success)]' : 'text-[var(--danger)]'
                }`}
              >
                {data.totalPnl >= 0 ? '+' : ''}${data.totalPnl.toFixed(2)}
              </p>
            </div>
          </div>
        </div>
      </div>

      <section className="px-4 mt-4">
        <h2 className="text-lg font-bold mb-3">
          {t('portfolio.activePositions', { n: data.activeCount })}
        </h2>
        <PositionList positions={data.positions} filter="active" />
      </section>

      {resolvedCount > 0 && (
        <section className="px-4 mt-6 mb-6">
          <h2 className="text-lg font-bold mb-3">{t('portfolio.resolvedPositions', { n: resolvedCount })}</h2>
          <PositionList positions={data.positions} filter="resolved" />
        </section>
      )}
    </div>
  );
}
