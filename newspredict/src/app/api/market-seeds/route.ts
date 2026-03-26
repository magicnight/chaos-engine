import { NextRequest, NextResponse } from 'next/server';
import { db } from '@/lib/db';
import { markets } from '@/lib/db/schema';
import { eq } from 'drizzle-orm';
import { chaosClient } from '@/lib/chaos-client';

interface RawSeed {
  id: string;
  question: string;
  description?: string;
  category: string;
  closeAt?: string;
  suggested_end_time?: string;
  resolutionCriteria?: string;
  resolution_criteria?: string;
  resolutionSource?: string;
  resolution_source?: string;
  tags?: string[];
  relatedSources?: string[];
}

function parseCloseAt(seed: RawSeed): Date {
  const raw = seed.closeAt || seed.suggested_end_time || '';
  const d = new Date(raw);
  if (!isNaN(d.getTime())) return d;
  // Fallback: 7 days from now
  return new Date(Date.now() + 7 * 24 * 60 * 60 * 1000);
}

export async function GET(request: NextRequest) {
  try {
    const cronSecret = request.headers.get('x-cron-secret');
    if (cronSecret !== process.env.CRON_SECRET && cronSecret !== process.env.CHAOS_API_KEY) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }
    const data = await chaosClient.getMarketSeeds();
    const seeds = (data as { seeds: RawSeed[] }).seeds || [];

    const created: string[] = [];
    const errors: string[] = [];

    for (const seed of seeds) {
      try {
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
            closeAt: parseCloseAt(seed),
            resolutionCriteria: seed.resolutionCriteria || seed.resolution_criteria || seed.question,
            resolutionSource: seed.resolutionSource || seed.resolution_source || null,
            crucixSeedId: seed.id,
            creatorType: 'system',
            tags: seed.tags || null,
            relatedSources: seed.relatedSources || null,
          })
          .returning();

        created.push(market.id);
      } catch (e: unknown) {
        const msg = e instanceof Error ? e.message : String(e);
        errors.push(`${seed.id}: ${msg}`);
      }
    }

    return NextResponse.json({ created: created.length, ids: created, seedCount: seeds.length, errors });
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : 'Failed to sync market seeds';
    return NextResponse.json({ error: message }, { status: 500 });
  }
}
