import { NextRequest, NextResponse } from 'next/server';
import { db } from '@/lib/db';
import { users } from '@/lib/db/schema';
import { desc, sql } from 'drizzle-orm';
import { redis } from '@/lib/redis';

interface RankingEntry {
  userId: string;
  name: string | null;
  avatar: string | null;
  pnl: number;
  winRate: number;
  totalTrades: number;
}

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = request.nextUrl;
    const period = searchParams.get('period') || 'weekly';
    const cacheKey = `leaderboard:${period}`;

    // Try Redis cache first
    try {
      const cached = await redis.get<RankingEntry[]>(cacheKey);
      if (cached) {
        return NextResponse.json({ rankings: cached, period, cached: true });
      }
    } catch {
      // Redis unavailable, fall through to DB
    }

    // Fallback: query DB
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

    const rankings: RankingEntry[] = rows.map((u: typeof users.$inferSelect) => ({
      userId: u.id,
      name: u.name,
      avatar: u.avatarUrl,
      pnl: Number(u.balance) - 1000,
      winRate: u.totalTrades > 0 ? Math.round((u.wins / u.totalTrades) * 100) : 0,
      totalTrades: u.totalTrades,
    }));

    // Cache to Redis for 5 minutes
    try {
      await redis.set(cacheKey, rankings, { ex: 300 });
    } catch {
      // Redis unavailable, skip caching
    }

    return NextResponse.json({ rankings, period, cached: false });
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : 'Failed to fetch leaderboard';
    return NextResponse.json({ error: message }, { status: 500 });
  }
}
