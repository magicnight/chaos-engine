'use client';

import { useState } from 'react';
import { OrderPanel } from '@/components/market/order-panel';
import { PriceChart } from '@/components/market/price-chart';
import { MarketStats } from '@/components/market/market-stats';

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

function timeAgo(iso: string): string {
  const diff = Date.now() - new Date(iso).getTime();
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return 'just now';
  if (mins < 60) return `${mins}m ago`;
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return `${hrs}h ago`;
  return `${Math.floor(hrs / 24)}d ago`;
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

  return (
    <div className="px-4 space-y-4 pb-8">
      <PriceChart data={priceHistory} />

      {status === 'open' && (
        <OrderPanel
          marketId={marketId}
          yesPrice={yesPrice}
          noPrice={noPrice}
          onTrade={(result) => {
            if (result.success) setTradeSuccess(true);
          }}
        />
      )}

      {tradeSuccess && (
        <div className="bg-[var(--success)]/15 text-[var(--success)] text-sm p-3 rounded-lg text-center">
          Trade placed successfully!
        </div>
      )}

      <MarketStats
        volume={volume}
        traderCount={traderCount}
        closeAt={closeAt}
        resolutionCriteria={resolutionCriteria}
        resolutionSource={resolutionSource}
      />

      <div className="bg-[var(--card)] rounded-xl p-4">
        <h3 className="text-sm font-semibold mb-3">Recent Activity</h3>
        {recentTrades.length === 0 ? (
          <p className="text-xs text-[var(--muted)]">No trades yet</p>
        ) : (
          <div className="space-y-2">
            {recentTrades.map((t: any) => (
              <div key={t.id} className="flex items-center justify-between text-xs">
                <span>
                  <span className="text-[var(--foreground)] font-medium">{t.userName}</span>{' '}
                  <span className="text-[var(--muted)]">bought</span>{' '}
                  <span
                    className={
                      t.side === 'YES' ? 'text-[var(--success)]' : 'text-[var(--danger)]'
                    }
                  >
                    {t.side}
                  </span>{' '}
                  <span className="text-[var(--muted)]">@ ${t.price.toFixed(2)}</span>
                </span>
                <span className="text-[var(--muted)]">{timeAgo(t.createdAt)}</span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
