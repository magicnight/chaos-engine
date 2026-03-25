'use client';

import Link from 'next/link';

interface PositionItem {
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

interface PositionListProps {
  positions: PositionItem[];
  filter: 'active' | 'resolved';
}

export function PositionList({ positions, filter }: PositionListProps) {
  const filtered = positions.filter((p: any) =>
    filter === 'active' ? p.market.status === 'open' : p.market.status === 'resolved'
  );

  if (filtered.length === 0) {
    return (
      <p className="text-center text-sm text-[var(--muted)] py-6">
        No {filter} positions
      </p>
    );
  }

  return (
    <div className="space-y-3">
      {filtered.map((p: any) => {
        const pnl = filter === 'resolved' ? p.realizedPnl : p.unrealizedPnl;
        const isPositive = pnl >= 0;
        const won =
          filter === 'resolved' &&
          ((p.market.resolutionResult === 'YES' && p.side === 'YES') ||
            (p.market.resolutionResult === 'NO' && p.side === 'NO'));

        return (
          <Link
            key={p.id}
            href={`/markets/${p.marketId}`}
            className="block rounded-xl bg-[var(--card)] p-4 active:bg-[var(--card-hover)]"
          >
            <div className="flex items-start justify-between mb-2">
              <p className="text-sm font-medium leading-tight flex-1 mr-3">
                {p.market.question}
              </p>
              <span className="text-xs px-2 py-0.5 rounded-full bg-[var(--border)] text-[var(--muted)]">
                {p.market.category}
              </span>
            </div>
            <div className="flex items-center justify-between text-xs">
              <div className="flex items-center gap-3">
                <span className={p.side === 'YES' ? 'text-[var(--success)]' : 'text-[var(--danger)]'}>
                  {p.side}: {p.shares.toFixed(1)} shares
                </span>
                <span className="text-[var(--muted)]">
                  @ ${p.avgPrice.toFixed(2)}
                </span>
                {filter === 'active' && (
                  <span className="text-[var(--muted)]">
                    Now: ${p.currentPrice.toFixed(2)}
                  </span>
                )}
              </div>
              <span
                className={`font-bold ${
                  filter === 'resolved'
                    ? won
                      ? 'text-[var(--success)]'
                      : 'text-[var(--danger)]'
                    : isPositive
                    ? 'text-[var(--success)]'
                    : 'text-[var(--danger)]'
                }`}
              >
                {filter === 'resolved' ? (won ? '+' : '') : isPositive ? '+' : ''}
                ${pnl.toFixed(2)}
              </span>
            </div>
          </Link>
        );
      })}
    </div>
  );
}
