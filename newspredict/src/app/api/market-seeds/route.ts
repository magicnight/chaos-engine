import { NextRequest, NextResponse } from 'next/server';
import { db } from '@/lib/db';
import { markets } from '@/lib/db/schema';
import { eq } from 'drizzle-orm';
import { chaosClient } from '@/lib/chaos-client';

interface MarketSeed {
  id: string;
  question: string;
  description?: string;
  category: string;
  closeAt: string;
  resolutionCriteria: string;
  resolutionSource?: string;
  tags?: string[];
  relatedSources?: string[];
}

export async function GET(request: NextRequest) {
  try {
    const cronSecret = request.headers.get('x-cron-secret');
    if (cronSecret !== process.env.CRON_SECRET && cronSecret !== process.env.CHAOS_API_KEY) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }
    const data = await chaosClient.getMarketSeeds();
    const seeds = (data as { seeds: MarketSeed[] }).seeds || [];

    const created: string[] = [];

    for (const seed of seeds) {
      const [existing] = await db
        .select({ id: markets.id })
        .from(markets)
        .where(eq(markets.crucixSeedId, seed.id));

      if (existing) continue;

      const [market] = await db
        .insert(markets)
        .values({
          question: seed.question,
          description: seed.description || null,
          category: seed.category,
          closeAt: new Date(seed.closeAt),
          resolutionCriteria: seed.resolutionCriteria,
          resolutionSource: seed.resolutionSource || null,
          crucixSeedId: seed.id,
          creatorType: 'system',
          tags: seed.tags || null,
          relatedSources: seed.relatedSources || null,
        })
        .returning();

      created.push(market.id);
    }

    return NextResponse.json({ created: created.length, ids: created });
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : 'Failed to sync market seeds';
    return NextResponse.json({ error: message }, { status: 500 });
  }
}
