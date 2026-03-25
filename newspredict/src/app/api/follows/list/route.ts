import { NextRequest, NextResponse } from 'next/server';
import { db } from '@/lib/db';
import { follows, users } from '@/lib/db/schema';
import { eq, sql } from 'drizzle-orm';

const UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = request.nextUrl;
    const userId = searchParams.get('userId');
    const type = searchParams.get('type') || 'followers';

    if (!userId || !UUID_RE.test(userId)) {
      return NextResponse.json({ error: 'Invalid userId' }, { status: 400 });
    }

    if (type === 'followers') {
      const rows = await db
        .select({
          id: users.id,
          name: users.name,
          avatarUrl: users.avatarUrl,
        })
        .from(follows)
        .innerJoin(users, eq(follows.followerId, users.id))
        .where(eq(follows.followingId, userId))
        .limit(50);

      return NextResponse.json({ users: rows });
    }

    // type === 'following'
    const rows = await db
      .select({
        id: users.id,
        name: users.name,
        avatarUrl: users.avatarUrl,
      })
      .from(follows)
      .innerJoin(users, eq(follows.followingId, users.id))
      .where(eq(follows.followerId, userId))
      .limit(50);

    return NextResponse.json({ users: rows });
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : 'Failed to fetch follows';
    return NextResponse.json({ error: message }, { status: 500 });
  }
}
