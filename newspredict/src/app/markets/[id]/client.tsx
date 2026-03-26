'use client';

import { useState, useRef, useEffect } from 'react';
import { OrderPanel } from '@/components/market/order-panel';
import { PriceChart } from '@/components/market/price-chart';
import { MarketStats } from '@/components/market/market-stats';
import { useLocale } from '@/lib/i18n/context';
import { useLivePrice } from '@/hooks/use-live-price';
import { CommentsSection } from '@/components/market/comments-section';

interface TradeItem {
  id: string;
  side: 'YES' | 'NO';
  shares: number;
  price: number;
  cost: number;
  createdAt: string;
  userName: string;
}

interface MarketDetailClientProps {
  marketId: string;
  yesPrice: number;
  noPrice: number;
  volume: number;
  traderCount: number;
  closeAt: string;
  resolutionCriteria: string;
  resolutionSource?: string | null;
  priceHistory: { price: number; time: string }[];
  recentTrades: TradeItem[];
  status: string;
}

function useTimeAgo() {
  const { t } = useLocale();
  return function timeAgo(iso: string): string {
    const diff = Date.now() - new Date(iso).getTime();
    const mins = Math.floor(diff / 60000);
    if (mins < 1) return t('common.justNow');
    if (mins < 60) return t('common.mAgo', { n: mins });
    const hrs = Math.floor(mins / 60);
    if (hrs < 24) return t('common.hAgo', { n: hrs });
    return t('common.dAgo', { n: Math.floor(hrs / 24) });
  };
}

export function MarketDetailClient({
  marketId,
  yesPrice,
  noPrice,
  volume,
  traderCount,
  closeAt,
  resolutionCriteria,
  resolutionSource,
  priceHistory,
  recentTrades,
  status,
}: MarketDetailClientProps) {
  const [tradeSuccess, setTradeSuccess] = useState(false);
  const { t } = useLocale();
  const timeAgo = useTimeAgo();
  const live = useLivePrice(marketId, { yesPrice, noPrice, volume, traderCount });

  // Price flash animation
  const prevYes = useRef(yesPrice);
  const [flashClass, setFlashClass] = useState('');
  useEffect(() => {
    if (live.yesPrice !== prevYes.current) {
      setFlashClass(live.yesPrice > prevYes.current ? 'animate-flash-green' : 'animate-flash-red');
      prevYes.current = live.yesPrice;
      const timer = setTimeout(() => setFlashClass(''), 600);
      return () => clearTimeout(timer);
    }
  }, [live.yesPrice]);

  return (
    <div className="px-4 space-y-4 pb-8">
      <div className={flashClass}>
        <PriceChart data={priceHistory} />
      </div>

      {status === 'open' && (
        <OrderPanel
          marketId={marketId}
          yesPrice={live.yesPrice}
          noPrice={live.noPrice}
          onTrade={(result) => {
            if (result.success) setTradeSuccess(true);
          }}
        />
      )}

      {tradeSuccess && (
        <div className="bg-[var(--success)]/15 text-[var(--success)] text-sm p-3 rounded-lg text-center">
          {t('market.tradePlacedSuccess')}
        </div>
      )}

      <MarketStats
        volume={live.volume}
        traderCount={live.traderCount}
        closeAt={closeAt}
        resolutionCriteria={resolutionCriteria}
        resolutionSource={resolutionSource}
      />

      <div className="bg-[var(--card)] rounded-xl p-4">
        <h3 className="text-sm font-semibold mb-3">{t('market.recentActivity')}</h3>
        {recentTrades.length === 0 ? (
          <p className="text-xs text-[var(--muted)]">{t('market.noTradesYet')}</p>
        ) : (
          <div className="space-y-2">
            {recentTrades.map((tr: any) => (
              <div key={tr.id} className="flex items-center justify-between text-xs">
                <span>
                  <span className="text-[var(--foreground)] font-medium">{tr.userName}</span>{' '}
                  <span className="text-[var(--muted)]">{t('market.bought')}</span>{' '}
                  <span
                    className={
                      tr.side === 'YES' ? 'text-[var(--success)]' : 'text-[var(--danger)]'
                    }
                  >
                    {tr.side}
                  </span>{' '}
                  <span className="text-[var(--muted)]">@ ${tr.price.toFixed(2)}</span>
                </span>
                <span className="text-[var(--muted)]">{timeAgo(tr.createdAt)}</span>
              </div>
            ))}
          </div>
        )}
      </div>

      <CommentsSection marketId={marketId} />
    </div>
  );
}
