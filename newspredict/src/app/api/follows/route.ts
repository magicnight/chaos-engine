import { NextRequest, NextResponse } from 'next/server';
import { auth } from '@/lib/auth';
import { db } from '@/lib/db';
import { follows, users } from '@/lib/db/schema';
import { and, eq, sql } from 'drizzle-orm';

const UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

export async function POST(request: NextRequest) {
  try {
    const session = await auth();
    if (!session?.user?.id) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const body = await request.json();
    const { targetUserId, action } = body;

    if (!targetUserId || !action) {
      return NextResponse.json({ error: 'Missing required fields' }, { status: 400 });
    }

    if (!UUID_RE.test(targetUserId)) {
      return NextResponse.json({ error: 'Invalid user ID' }, { status: 400 });
    }

    if (action !== 'follow' && action !== 'unfollow') {
      return NextResponse.json({ error: 'Action must be follow or unfollow' }, { status: 400 });
    }

    if (targetUserId === session.user.id) {
      return NextResponse.json({ error: 'Cannot follow yourself' }, { status: 400 });
    }

    // Verify target user exists
    const [target] = await db.select().from(users).where(eq(users.id, targetUserId));
    if (!target) {
      return NextResponse.json({ error: 'User not found' }, { status: 404 });
    }

    if (action === 'follow') {
      await db
        .insert(follows)
        .values({ followerId: session.user.id, followingId: targetUserId })
        .onConflictDoNothing();
    } else {
      await db
        .delete(follows)
        .where(
          and(
            eq(follows.followerId, session.user.id),
            eq(follows.followingId, targetUserId)
          )
        );
    }

    // Return updated counts
    const [followerCount] = await db
      .select({ count: sql<number>`count(*)::int` })
      .from(follows)
      .where(eq(follows.followingId, targetUserId));

    const [followingCount] = await db
      .select({ count: sql<number>`count(*)::int` })
      .from(follows)
      .where(eq(follows.followerId, targetUserId));

    return NextResponse.json({
      success: true,
      action,
      followers: followerCount.count,
      following: followingCount.count,
    });
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : 'Failed to process follow';
    return NextResponse.json({ error: message }, { status: 500 });
  }
}
