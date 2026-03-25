import { NextRequest, NextResponse } from 'next/server';
import { db } from '@/lib/db';
import { users, trades, positions, markets } from '@/lib/db/schema';
import { eq, desc } from 'drizzle-orm';
import { getPrice } from '@/lib/market-engine';

export async function GET(
  _request: NextRequest,
  { params }: { params: Promise<{ userId: string }> }
) {
  try {
    const { userId } = await params;

    const [user] = await db.select().from(users).where(eq(users.id, userId));
    if (!user) {
      return NextResponse.json({ error: 'User not found' }, { status: 404 });
    }

    const recentTrades = await db
      .select({
        trade: trades,
        market: markets,
      })
      .from(trades)
      .innerJoin(markets, eq(trades.marketId, markets.id))
      .where(eq(trades.userId, userId))
      .orderBy(desc(trades.createdAt))
      .limit(10);

    const userPositions = await db
      .select({
        position: positions,
        market: markets,
      })
      .from(positions)
      .innerJoin(markets, eq(positions.marketId, markets.id))
      .where(eq(positions.userId, userId));

    const winRate =
      user.totalTrades > 0 ? Math.round((user.wins / user.totalTrades) * 100) : 0;

    return NextResponse.json({
      user: {
        id: user.id,
        name: user.name,
        avatarUrl: user.avatarUrl,
        createdAt: user.createdAt.toISOString(),
        totalTrades: user.totalTrades,
        winRate,
        pnl: Number(user.balance) - 1000,
      },
      recentTrades: recentTrades.map((r: any) => ({
        id: r.trade.id,
        side: r.trade.side,
        shares: Number(r.trade.shares),
        price: Number(r.trade.price),
        cost: Number(r.trade.cost),
        createdAt: r.trade.createdAt.toISOString(),
        market: {
          id: r.market.id,
          question: r.market.question,
          category: r.market.category,
        },
      })),
      positions: userPositions
        .filter((p: any) => p.market.status === 'open')
        .map((p: any) => {
          const price = getPrice(
            Number(p.market.yesShares),
            Number(p.market.noShares),
            Number(p.market.liquidityParam)
          );
          return {
            id: p.position.id,
            side: p.position.side,
            shares: Number(p.position.shares),
            currentPrice: p.position.side === 'YES' ? price.yes : price.no,
            market: {
              id: p.market.id,
              question: p.market.question,
              category: p.market.category,
            },
          };
        }),
    });
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : 'Failed to fetch user';
    return NextResponse.json({ error: message }, { status: 500 });
  }
}
