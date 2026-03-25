import { NextRequest, NextResponse } from 'next/server';
import { db } from '@/lib/db';
import { markets, trades, users } from '@/lib/db/schema';
import { eq, desc } from 'drizzle-orm';
import { getPrice } from '@/lib/market-engine';

export async function GET(
  _request: NextRequest,
  { params }: { params: Promise<{ id: string }> }
) {
  try {
    const { id } = await params;
    const [market] = await db.select().from(markets).where(eq(markets.id, id));
    if (!market) {
      return NextResponse.json({ error: 'Market not found' }, { status: 404 });
    }

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

    return NextResponse.json({
      id: market.id,
      question: market.question,
      description: market.description,
      category: market.category,
      imageUrl: market.imageUrl,
      status: market.status,
      creatorType: market.creatorType,
      yesPrice: price.yes,
      noPrice: price.no,
      yesShares: Number(market.yesShares),
      noShares: Number(market.noShares),
      volume: Number(market.volume),
      traderCount: market.traderCount,
      resolutionCriteria: market.resolutionCriteria,
      resolutionSource: market.resolutionSource,
      resolutionResult: market.resolutionResult,
      resolvedAt: market.resolvedAt?.toISOString() || null,
      closeAt: market.closeAt.toISOString(),
      createdAt: market.createdAt.toISOString(),
      tags: market.tags,
      relatedSources: market.relatedSources,
      recentTrades: recentTrades.map((t: any) => ({
        id: t.id,
        side: t.side,
        shares: Number(t.shares),
        price: Number(t.price),
        cost: Number(t.cost),
        createdAt: t.createdAt.toISOString(),
        userName: t.userName || 'Anonymous',
      })),
    });
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : 'Failed to fetch market';
    return NextResponse.json({ error: message }, { status: 500 });
  }
}
