import { NextRequest, NextResponse } from 'next/server';
import { db } from '@/lib/db';
import { markets, positions, users } from '@/lib/db/schema';
import { eq, and, sql } from 'drizzle-orm';
import { auth } from '@/lib/auth';

const UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

export async function POST(request: NextRequest) {
  try {
    const session = await auth();
    if (!session?.user?.id) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const body = await request.json();
    const { marketId, result } = body;

    if (!marketId || !result) {
      return NextResponse.json({ error: 'Missing required fields' }, { status: 400 });
    }

    if (!UUID_RE.test(marketId)) {
      return NextResponse.json({ error: 'Invalid market ID' }, { status: 400 });
    }

    if (result !== 'YES' && result !== 'NO' && result !== 'CANCELLED') {
      return NextResponse.json({ error: 'Result must be YES, NO, or CANCELLED' }, { status: 400 });
    }

    const [market] = await db.select().from(markets).where(eq(markets.id, marketId));
    if (!market) {
      return NextResponse.json({ error: 'Market not found' }, { status: 404 });
    }
    if (market.creatorId !== session.user.id) {
      return NextResponse.json({ error: 'Only the market creator can resolve this market' }, { status: 403 });
    }
    if (market.status !== 'open' && market.status !== 'closed') {
      return NextResponse.json({ error: 'Market already resolved' }, { status: 400 });
    }

    await db
      .update(markets)
      .set({
        status: 'resolved',
        resolutionResult: result,
        resolvedAt: sql`now()`,
      })
      .where(eq(markets.id, marketId));

    if (result === 'CANCELLED') {
      // Batch refund: credit all position holders based on avg_price * shares
      await db.execute(sql`
        UPDATE users SET balance = balance + (
          SELECT CAST(p.avg_price AS DECIMAL) * CAST(p.shares AS DECIMAL) FROM positions p
          WHERE p.user_id = users.id AND p.market_id = ${marketId}
        )
        WHERE id IN (
          SELECT user_id FROM positions WHERE market_id = ${marketId}
        )
      `);

      const allPositions = await db
        .select()
        .from(positions)
        .where(eq(positions.marketId, marketId));

      return NextResponse.json({ resolved: true, result: 'CANCELLED', refunded: allPositions.length });
    }

    const winningSide = result;

    // Batch payout: credit all winning position holders with their shares value
    await db.execute(sql`
      UPDATE users SET balance = balance + (
        SELECT CAST(p.shares AS DECIMAL) FROM positions p
        WHERE p.user_id = users.id AND p.market_id = ${marketId} AND p.side = ${winningSide}
      ), wins = wins + 1
      WHERE id IN (
        SELECT user_id FROM positions WHERE market_id = ${marketId} AND side = ${winningSide}
      )
    `);

    // Batch update realized PnL for winning positions
    await db.execute(sql`
      UPDATE positions SET realized_pnl = CAST(shares AS DECIMAL) - (CAST(avg_price AS DECIMAL) * CAST(shares AS DECIMAL))
      WHERE market_id = ${marketId} AND side = ${winningSide}
    `);

    const winningPositions = await db
      .select()
      .from(positions)
      .where(and(eq(positions.marketId, marketId), eq(positions.side, winningSide)));

    const totalPaid = winningPositions.reduce((sum: number, pos: any) => sum + Number(pos.shares), 0);

    return NextResponse.json({
      resolved: true,
      result,
      winnersCount: winningPositions.length,
      totalPaid: totalPaid.toFixed(2),
    });
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : 'Resolution failed';
    return NextResponse.json({ error: message }, { status: 500 });
  }
}
