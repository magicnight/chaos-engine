'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';

interface ActivityItem {
  id: string;
  type: 'trade' | 'resolved';
  side?: string;
  shares?: number;
  price?: number;
  cost?: number;
  pnl?: number;
  won?: boolean;
  createdAt: string;
  market: {
    id: string;
    question: string;
  };
}

export default function ActivityPage() {
  const [items, setItems] = useState<ActivityItem[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetch('/api/portfolio')
      .then((r) => r.json())
      .then((data) => {
        if (data.error || !data.positions) {
          setLoading(false);
          return;
        }

        const activities: ActivityItem[] = data.positions.map(
          (p: {
            id: string;
            side: string;
            shares: number;
            avgPrice: number;
            realizedPnl: number;
            market: {
              id: string;
              question: string;
              status: string;
              resolutionResult?: string | null;
            };
          }) => {
            if (p.market.status === 'resolved') {
              const won =
                (p.market.resolutionResult === 'YES' && p.side === 'YES') ||
                (p.market.resolutionResult === 'NO' && p.side === 'NO');
              return {
                id: p.id,
                type: 'resolved' as const,
                won,
                pnl: p.realizedPnl,
                createdAt: new Date().toISOString(),
                market: { id: p.market.id, question: p.market.question },
              };
            }
            return {
              id: p.id,
              type: 'trade' as const,
              side: p.side,
              shares: p.shares,
              price: p.avgPrice,
              createdAt: new Date().toISOString(),
              market: { id: p.market.id, question: p.market.question },
            };
          }
        );

        setItems(activities);
        setLoading(false);
      })
      .catch(() => setLoading(false));
  }, []);

  if (loading) {
    return (
      <div className="p-4">
        <h1 className="text-2xl font-bold mb-4">Activity</h1>
        <div className="space-y-3">
          {[1, 2, 3].map((i: any) => (
            <div key={i} className="h-16 rounded-xl bg-[var(--card)] animate-pulse" />
          ))}
        </div>
      </div>
    );
  }

  return (
    <div className="p-4">
      <h1 className="text-2xl font-bold mb-4">Activity</h1>

      {items.length === 0 ? (
        <p className="text-[var(--muted)] text-center py-8">No activity yet</p>
      ) : (
        <div className="space-y-3">
          {items.map((item: any) => (
            <Link
              key={item.id}
              href={`/markets/${item.market.id}`}
              className="block rounded-xl bg-[var(--card)] p-4 active:bg-[var(--card-hover)]"
            >
              {item.type === 'trade' ? (
                <>
                  <div className="flex items-start gap-2 mb-1">
                    <span className="text-sm">Your trade on</span>
                  </div>
                  <p className="text-sm font-medium mb-1">
                    &quot;{item.market.question}&quot;
                  </p>
                  <p className="text-xs text-[var(--muted)]">
                    {item.shares?.toFixed(1)} shares{' '}
                    <span
                      className={
                        item.side === 'YES'
                          ? 'text-[var(--success)]'
                          : 'text-[var(--danger)]'
                      }
                    >
                      {item.side}
                    </span>{' '}
                    @ ${item.price?.toFixed(2)}
                  </p>
                </>
              ) : (
                <>
                  <div className="flex items-start gap-2 mb-1">
                    <span className="text-sm">
                      Market resolved
                    </span>
                  </div>
                  <p className="text-sm font-medium mb-1">
                    &quot;{item.market.question}&quot;
                  </p>
                  <p
                    className={`text-xs font-bold ${
                      item.won ? 'text-[var(--success)]' : 'text-[var(--danger)]'
                    }`}
                  >
                    {item.won ? 'Won' : 'Lost'}: {item.won ? '+' : ''}$
                    {item.pnl?.toFixed(2)}
                  </p>
                </>
              )}
            </Link>
          ))}
        </div>
      )}
    </div>
  );
}
