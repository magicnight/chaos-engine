import Link from 'next/link';
import { notFound } from 'next/navigation';
import { db } from '@/lib/db';
import { markets, trades, users } from '@/lib/db/schema';
import { eq, desc } from 'drizzle-orm';
import { getPrice } from '@/lib/market-engine';
import { MarketDetailClient } from './client';
import { T } from '@/components/i18n-text';

export const dynamic = 'force-dynamic';

export default async function MarketDetailPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = await params;

  const [market] = await db.select().from(markets).where(eq(markets.id, id));
  if (!market) notFound();

  const price = getPrice(
    Number(market.yesShares),
    Number(market.noShares),
    Number(market.liquidityParam)
  );

  const recentTrades = await db
    .select({
      id: trades.id,
      side: trades.side,
      shares: trades.shares,
      price: trades.price,
      cost: trades.cost,
      createdAt: trades.createdAt,
      userName: users.name,
    })
    .from(trades)
    .leftJoin(users, eq(trades.userId, users.id))
    .where(eq(trades.marketId, id))
    .orderBy(desc(trades.createdAt))
    .limit(20);

  const tradeList = recentTrades.map((t: any) => ({
    id: t.id,
    side: t.side as 'YES' | 'NO',
    shares: Number(t.shares),
    price: Number(t.price),
    cost: Number(t.cost),
    createdAt: t.createdAt.toISOString(),
    userName: t.userName || 'Anonymous',
  }));

  const priceHistory = tradeList
    .slice()
    .reverse()
    .map((t: any) => ({
      price: t.price,
      time: new Date(t.createdAt).toLocaleTimeString('en-US', {
        hour: '2-digit',
        minute: '2-digit',
      }),
    }));

  return (
    <div>
      <div className="px-4 pt-4 pb-2 flex items-center gap-3">
        <Link href="/markets" className="text-[var(--muted)] hover:text-[var(--foreground)]">
          &larr; <T k="common.back" />
        </Link>
        <span className="text-[10px] text-[var(--accent)] font-medium uppercase tracking-wide">
          {market.category}
        </span>
      </div>

      <div className="px-4 mb-4">
        <h1 className="text-lg font-bold leading-tight mb-1">{market.question}</h1>
        <p className="text-xs text-[var(--muted)]">
          {market.traderCount.toLocaleString()} traders &middot;{' '}
          {Number(market.volume) >= 1000
            ? `$${(Number(market.volume) / 1000).toFixed(1)}K`
            : `$${Number(market.volume).toFixed(0)}`}{' '}
          <T k="common.volume" />
        </p>
      </div>

      <div className="px-4 mb-4 flex items-center gap-4">
        <div className="flex-1 text-center py-3 rounded-xl bg-[var(--success)]/10">
          <p className="text-lg font-bold text-[var(--success)]">
            ${price.yes.toFixed(2)}
          </p>
          <p className="text-[10px] text-[var(--muted)] uppercase">Yes</p>
        </div>
        <div className="flex-1 text-center py-3 rounded-xl bg-[var(--danger)]/10">
          <p className="text-lg font-bold text-[var(--danger)]">
            ${price.no.toFixed(2)}
          </p>
          <p className="text-[10px] text-[var(--muted)] uppercase">No</p>
        </div>
      </div>

      <MarketDetailClient
        marketId={market.id}
        yesPrice={price.yes}
        noPrice={price.no}
        volume={Number(market.volume)}
        traderCount={market.traderCount}
        closeAt={market.closeAt.toISOString()}
        resolutionCriteria={market.resolutionCriteria}
        resolutionSource={market.resolutionSource}
        priceHistory={priceHistory}
        recentTrades={tradeList}
        status={market.status}
      />
    </div>
  );
}
