import { NextResponse } from 'next/server';
import { auth } from '@/lib/auth';
import { db } from '@/lib/db';
import { users, positions, markets } from '@/lib/db/schema';
import { eq } from 'drizzle-orm';
import { getPrice } from '@/lib/market-engine';

export async function GET() {
  try {
    const session = await auth();
    if (!session?.user?.id) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const [user] = await db.select().from(users).where(eq(users.id, session.user.id));
    if (!user) {
      return NextResponse.json({ error: 'User not found' }, { status: 404 });
    }

    const rows = await db
      .select({
        position: positions,
        market: markets,
      })
      .from(positions)
      .innerJoin(markets, eq(positions.marketId, markets.id))
      .where(eq(positions.userId, session.user.id));

    let totalPnl = 0;
    let activeCount = 0;

    const positionList = rows.map((row: any) => {
      const p = row.position;
      const m = row.market;
      const shares = Number(p.shares);
      const avgPrice = Number(p.avgPrice);
      const realized = Number(p.realizedPnl);

      let currentPrice = 0;
      let unrealizedPnl = 0;

      if (m.status === 'resolved') {
        const won =
          (m.resolutionResult === 'YES' && p.side === 'YES') ||
          (m.resolutionResult === 'NO' && p.side === 'NO');
        currentPrice = won ? 1 : 0;
        unrealizedPnl = realized;
      } else {
        const price = getPrice(Number(m.yesShares), Number(m.noShares), Number(m.liquidityParam));
        currentPrice = p.side === 'YES' ? price.yes : price.no;
        unrealizedPnl = (currentPrice - avgPrice) * shares;
        activeCount++;
      }

      totalPnl += unrealizedPnl;

      return {
        id: p.id,
        marketId: m.id,
        side: p.side,
        shares,
        avgPrice,
        currentPrice,
        unrealizedPnl,
        realizedPnl: realized,
        market: {
          id: m.id,
          question: m.question,
          status: m.status,
          category: m.category,
          resolutionResult: m.resolutionResult,
        },
      };
    });

    const winRate =
      user.totalTrades > 0 ? Math.round((user.wins / user.totalTrades) * 100) : 0;

    return NextResponse.json({
      userId: user.id,
      userName: user.name,
      createdAt: user.createdAt.toISOString(),
      balance: Number(user.balance),
      totalPnl,
      positions: positionList,
      activeCount,
      winRate,
    });
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : 'Failed to fetch portfolio';
    return NextResponse.json({ error: message }, { status: 500 });
  }
}
