import { NextRequest, NextResponse } from 'next/server';
import { auth } from '@/lib/auth';
import { db } from '@/lib/db';
import { comments, users, markets } from '@/lib/db/schema';
import { eq, desc } from 'drizzle-orm';
import { rateLimit } from '@/lib/rate-limit';

const UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = request.nextUrl;
    const marketId = searchParams.get('marketId');

    if (!marketId) {
      return NextResponse.json({ error: 'marketId is required' }, { status: 400 });
    }
    if (!UUID_RE.test(marketId)) {
      return NextResponse.json({ error: 'Invalid market ID' }, { status: 400 });
    }

    const rows = await db
      .select({
        comment: comments,
        user: {
          id: users.id,
          name: users.name,
          avatarUrl: users.avatarUrl,
        },
      })
      .from(comments)
      .innerJoin(users, eq(comments.userId, users.id))
      .where(eq(comments.marketId, marketId))
      .orderBy(desc(comments.createdAt))
      .limit(50);

    return NextResponse.json(
      rows.map((r: any) => ({
        id: r.comment.id,
        content: r.comment.content,
        createdAt: r.comment.createdAt.toISOString(),
        user: {
          id: r.user.id,
          name: r.user.name,
          avatarUrl: r.user.avatarUrl,
        },
      }))
    );
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : 'Failed to fetch comments';
    return NextResponse.json({ error: message }, { status: 500 });
  }
}

export async function POST(request: NextRequest) {
  try {
    const ip = request.headers.get('x-forwarded-for')?.split(',')[0]?.trim() || 'unknown';
    const { allowed } = rateLimit(ip, 'comment', 10); // 10 comments/min
    if (!allowed) return NextResponse.json({ error: 'Too many requests' }, { status: 429 });

    const session = await auth();
    if (!session?.user?.id) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const body = await request.json();
    const { marketId, content } = body;

    if (!marketId || !content) {
      return NextResponse.json({ error: 'Missing required fields' }, { status: 400 });
    }

    if (!UUID_RE.test(marketId)) {
      return NextResponse.json({ error: 'Invalid market ID' }, { status: 400 });
    }

    if (typeof content !== 'string' || content.length > 2000) {
      return NextResponse.json({ error: 'Content must be at most 2000 characters' }, { status: 400 });
    }

    // Verify market exists
    const [market] = await db.select().from(markets).where(eq(markets.id, marketId));
    if (!market) {
      return NextResponse.json({ error: 'Market not found' }, { status: 404 });
    }

    const [comment] = await db
      .insert(comments)
      .values({
        marketId,
        userId: session.user.id,
        content: content.trim(),
      })
      .returning();

    return NextResponse.json({
      id: comment.id,
      content: comment.content,
      createdAt: comment.createdAt.toISOString(),
    });
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : 'Failed to post comment';
    return NextResponse.json({ error: message }, { status: 500 });
  }
}
