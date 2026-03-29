import { NextRequest, NextResponse } from 'next/server';
import { auth } from '@/lib/auth';
import { db } from '@/lib/db';
import { copySettings, users } from '@/lib/db/schema';
import { eq, and } from 'drizzle-orm';

const UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

// GET — list copy-trade settings for current user
export async function GET() {
  try {
    const session = await auth();
    if (!session?.user?.id) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const copying = await db
      .select({
        id: copySettings.id,
        leaderId: copySettings.leaderId,
        maxAmount: copySettings.maxAmount,
        active: copySettings.active,
        leaderName: users.name,
      })
      .from(copySettings)
      .innerJoin(users, eq(copySettings.leaderId, users.id))
      .where(eq(copySettings.followerId, session.user.id));

    return NextResponse.json({
      copyTrades: copying.map((c: any) => ({
        id: c.id,
        leaderId: c.leaderId,
        leaderName: c.leaderName,
        maxAmount: c.maxAmount,
        active: c.active === 1,
      })),
    });
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : 'Failed to fetch copy settings';
    return NextResponse.json({ error: message }, { status: 500 });
  }
}

// POST — create, update, or delete copy-trade setting
export async function POST(request: NextRequest) {
  try {
    const session = await auth();
    if (!session?.user?.id) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const body = await request.json();
    const { action, leaderId, maxAmount } = body;

    if (!leaderId || !UUID_RE.test(leaderId)) {
      return NextResponse.json({ error: 'Invalid leader ID' }, { status: 400 });
    }

    if (leaderId === session.user.id) {
      return NextResponse.json({ error: 'Cannot copy yourself' }, { status: 400 });
    }

    if (action === 'start') {
      const amt = Math.min(Math.max(Number(maxAmount) || 50, 1), 1000);
      await db
        .insert(copySettings)
        .values({ followerId: session.user.id, leaderId, maxAmount: amt })
        .onConflictDoNothing();
      return NextResponse.json({ success: true, action: 'start' });
    }

    if (action === 'stop') {
      await db
        .delete(copySettings)
        .where(and(eq(copySettings.followerId, session.user.id), eq(copySettings.leaderId, leaderId)));
      return NextResponse.json({ success: true, action: 'stop' });
    }

    if (action === 'update') {
      const amt = Math.min(Math.max(Number(maxAmount) || 50, 1), 1000);
      await db
        .update(copySettings)
        .set({ maxAmount: amt })
        .where(and(eq(copySettings.followerId, session.user.id), eq(copySettings.leaderId, leaderId)));
      return NextResponse.json({ success: true, action: 'update' });
    }

    if (action === 'pause') {
      await db
        .update(copySettings)
        .set({ active: 0 })
        .where(and(eq(copySettings.followerId, session.user.id), eq(copySettings.leaderId, leaderId)));
      return NextResponse.json({ success: true, action: 'pause' });
    }

    if (action === 'resume') {
      await db
        .update(copySettings)
        .set({ active: 1 })
        .where(and(eq(copySettings.followerId, session.user.id), eq(copySettings.leaderId, leaderId)));
      return NextResponse.json({ success: true, action: 'resume' });
    }

    return NextResponse.json({ error: 'Invalid action' }, { status: 400 });
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : 'Failed to update copy settings';
    return NextResponse.json({ error: message }, { status: 500 });
  }
}
