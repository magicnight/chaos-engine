import { NextRequest, NextResponse } from 'next/server';
import { db } from '@/lib/db';
import { markets } from '@/lib/db/schema';
import { eq, desc, and } from 'drizzle-orm';
import { getPrice } from '@/lib/market-engine';
import { auth } from '@/lib/auth';

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = request.nextUrl;
    const category = searchParams.get('category');
    const status = searchParams.get('status') || 'open';
    const limit = Math.min(Number(searchParams.get('limit') || '20'), 50);

    const conditions = [eq(markets.status, status)];
    if (category && category !== 'All') {
      conditions.push(eq(markets.category, category));
    }

    const rows = await db
      .select()
      .from(markets)
      .where(and(...conditions))
      .orderBy(desc(markets.createdAt))
      .limit(limit);

    const result = rows.map((m: any) => {
      const price = getPrice(Number(m.yesShares), Number(m.noShares), Number(m.liquidityParam));
      return {
        id: m.id,
        question: m.question,
        description: m.description,
        category: m.category,
        imageUrl: m.imageUrl,
        status: m.status,
        creatorType: m.creatorType,
        yesPrice: price.yes,
        noPrice: price.no,
        volume: Number(m.volume),
        traderCount: m.traderCount,
        resolutionResult: m.resolutionResult,
        closeAt: m.closeAt.toISOString(),
        createdAt: m.createdAt.toISOString(),
        tags: m.tags,
        relatedSources: m.relatedSources,
      };
    });

    return NextResponse.json(result);
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : 'Failed to fetch markets';
    return NextResponse.json({ error: message }, { status: 500 });
  }
}

export async function POST(request: NextRequest) {
  try {
    const session = await auth();
    if (!session?.user?.id) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const body = await request.json();
    const { question, category, closeAt, resolutionCriteria, resolutionSource, description, tags, imageUrl } =
      body;

    if (!question || !category || !closeAt || !resolutionCriteria) {
      return NextResponse.json({ error: 'Missing required fields' }, { status: 400 });
    }

    if (typeof question !== 'string' || question.length > 500) {
      return NextResponse.json({ error: 'Question must be at most 500 characters' }, { status: 400 });
    }
    if (description && (typeof description !== 'string' || description.length > 2000)) {
      return NextResponse.json({ error: 'Description must be at most 2000 characters' }, { status: 400 });
    }
    if (typeof resolutionCriteria !== 'string' || resolutionCriteria.length > 1000) {
      return NextResponse.json({ error: 'Resolution criteria must be at most 1000 characters' }, { status: 400 });
    }

    if (imageUrl) {
      try {
        const url = new URL(imageUrl);
        if (!['http:', 'https:'].includes(url.protocol)) {
          return NextResponse.json({ error: 'Invalid image URL' }, { status: 400 });
        }
      } catch {
        return NextResponse.json({ error: 'Invalid image URL' }, { status: 400 });
      }
    }

    const [market] = await db
      .insert(markets)
      .values({
        question,
        description: description || null,
        category,
        closeAt: new Date(closeAt),
        resolutionCriteria,
        resolutionSource: resolutionSource || null,
        creatorId: session.user.id,
        creatorType: 'user',
        imageUrl: imageUrl || null,
        tags: tags || null,
      })
      .returning();

    const price = getPrice(0, 0);
    return NextResponse.json({
      id: market.id,
      question: market.question,
      category: market.category,
      yesPrice: price.yes,
      noPrice: price.no,
      status: market.status,
      closeAt: market.closeAt.toISOString(),
      createdAt: market.createdAt.toISOString(),
    });
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : 'Failed to create market';
    return NextResponse.json({ error: message }, { status: 500 });
  }
}
