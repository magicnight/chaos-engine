import { NextRequest, NextResponse } from 'next/server';
import { db } from '@/lib/db';
import { users, trades } from '@/lib/db/schema';
import { desc, sql, eq, gte, and } from 'drizzle-orm';
import { redis } from '@/lib/redis';

interface RankingEntry {
  userId: string;
  name: string | null;
  avatar: string | null;
  pnl: number;
  winRate: number;
  totalTrades: number;
}

function getPeriodStart(period: string): Date {
  const now = new Date();
  if (period === 'daily') return new Date(now.getTime() - 24 * 60 * 60 * 1000);
  if (period === 'weekly') return new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000);
  return new Date(0); // alltime
}

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = request.nextUrl;
    const period = searchParams.get('period') || 'weekly';
    const cacheKey = `leaderboard:${period}`;

    try {
      const cached = await redis.get<RankingEntry[]>(cacheKey);
      if (cached) {
        return NextResponse.json({ rankings: cached, period, cached: true });
      }
    } catch {}

    const periodStart = getPeriodStart(period);

    if (period === 'alltime') {
      // All-time: use user balance directly (most efficient)
      const rows = await db
        .select({
          id: users.id,
          name: users.name,
          avatarUrl: users.avatarUrl,
          balance: users.balance,
          totalTrades: users.totalTrades,
          wins: users.wins,
        })
        .from(users)
        .where(sql`${users.totalTrades} > 0`)
        .orderBy(desc(sql`cast(${users.balance} as numeric)`))
        .limit(50);

      const rankings: RankingEntry[] = rows.map((u: any) => ({
        userId: u.id,
        name: u.name,
        avatar: u.avatarUrl,
        pnl: Number(u.balance) - 1000,
        winRate: u.totalTrades > 0 ? Math.round((u.wins / u.totalTrades) * 100) : 0,
        totalTrades: u.totalTrades,
      }));

      try { await redis.set(cacheKey, rankings, { ex: 300 }); } catch {}
      return NextResponse.json({ rankings, period, cached: false });
    }

    // Daily/Weekly: aggregate trades within the period
    const rows = await db
      .select({
        userId: trades.userId,
        totalCost: sql<number>`sum(cast(${trades.cost} as numeric))`,
        tradeCount: sql<number>`count(*)::int`,
      })
      .from(trades)
      .where(gte(trades.createdAt, periodStart))
      .groupBy(trades.userId)
      .orderBy(desc(sql`sum(cast(${trades.cost} as numeric))`))
      .limit(50);

    const userIds = rows.map((r: any) => r.userId);
    if (userIds.length === 0) {
      return NextResponse.json({ rankings: [], period, cached: false });
    }

    // Fetch user info
    const userRows = await db
      .select({ id: users.id, name: users.name, avatarUrl: users.avatarUrl, balance: users.balance, wins: users.wins, totalTrades: users.totalTrades })
      .from(users)
      .where(sql`${users.id} IN ${userIds}`);

    const userMap = new Map(userRows.map((u: any) => [u.id, u]));

    const rankings: RankingEntry[] = rows.map((r: any) => {
      const u = userMap.get(r.userId);
      return {
        userId: r.userId,
        name: u?.name || null,
        avatar: u?.avatarUrl || null,
        pnl: Number(u?.balance || 1000) - 1000,
        winRate: u?.totalTrades > 0 ? Math.round((u.wins / u.totalTrades) * 100) : 0,
        totalTrades: r.tradeCount,
      };
    });

    try { await redis.set(cacheKey, rankings, { ex: 300 }); } catch {}
    return NextResponse.json({ rankings, period, cached: false });
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : 'Failed to fetch leaderboard';
    return NextResponse.json({ error: message }, { status: 500 });
  }
}
