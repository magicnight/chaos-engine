import { NextRequest, NextResponse } from 'next/server';
import { db } from '@/lib/db';
import { comments, users, markets } from '@/lib/db/schema';
import { eq, desc } from 'drizzle-orm';
import { redis } from '@/lib/redis';

const CHAOS_URL = process.env.CHAOS_API_URL || 'http://localhost:3117';
const CHAOS_KEY = process.env.CHAOS_API_KEY || '';

export async function GET(request: NextRequest) {
  try {
    const marketId = request.nextUrl.searchParams.get('marketId');
    if (!marketId) {
      return NextResponse.json({ error: 'Missing marketId' }, { status: 400 });
    }

    // Check cache (5 min TTL)
    const cacheKey = `sentiment:${marketId}`;
    try {
      const cached = await redis.get(cacheKey);
      if (cached) return NextResponse.json(cached);
    } catch {}

    // Fetch market question
    const [market] = await db.select({ question: markets.question }).from(markets).where(eq(markets.id, marketId));
    if (!market) {
      return NextResponse.json({ error: 'Market not found' }, { status: 404 });
    }

    // Fetch recent comments
    const rows = await db
      .select({ content: comments.content, userName: users.name })
      .from(comments)
      .innerJoin(users, eq(comments.userId, users.id))
      .where(eq(comments.marketId, marketId))
      .orderBy(desc(comments.createdAt))
      .limit(30);

    if (rows.length < 3) {
      const result = { summary: null, sentiment: 'neutral', commentCount: rows.length, reason: 'Not enough comments for analysis' };
      return NextResponse.json(result);
    }

    // Build prompt for CHAOS LLM
    const commentText = rows.map((r: any, i: number) => `${i + 1}. ${r.userName || 'Anonymous'}: ${r.content}`).join('\n');
    const prompt = `Analyze the sentiment of these comments about the prediction market question: "${market.question}"

Comments:
${commentText}

Respond in JSON format:
{
  "sentiment": "bullish" | "bearish" | "neutral" | "mixed",
  "confidence": 0.0-1.0,
  "summary": "2-3 sentence summary of the community consensus in the same language as the market question",
  "bullishCount": number,
  "bearishCount": number,
  "neutralCount": number
}`;

    // Call CHAOS Engine LLM query endpoint
    const headers: Record<string, string> = { 'Content-Type': 'application/json' };
    if (CHAOS_KEY) headers['X-CHAOS-Key'] = CHAOS_KEY;

    const llmRes = await fetch(`${CHAOS_URL}/api/v1/query`, {
      method: 'POST',
      headers,
      body: JSON.stringify({ query: prompt }),
    });

    if (!llmRes.ok) {
      return NextResponse.json({ summary: null, sentiment: 'neutral', commentCount: rows.length, reason: 'LLM unavailable' });
    }

    const llmData = await llmRes.json();
    const answer = llmData.answer || llmData.response || '';

    // Parse LLM JSON response
    let analysis;
    try {
      const jsonMatch = answer.match(/\{[\s\S]*\}/);
      analysis = jsonMatch ? JSON.parse(jsonMatch[0]) : null;
    } catch {
      analysis = null;
    }

    const result = {
      sentiment: analysis?.sentiment || 'neutral',
      confidence: analysis?.confidence || 0,
      summary: analysis?.summary || null,
      bullishCount: analysis?.bullishCount || 0,
      bearishCount: analysis?.bearishCount || 0,
      neutralCount: analysis?.neutralCount || 0,
      commentCount: rows.length,
    };

    // Cache result
    try { await redis.set(cacheKey, JSON.stringify(result), { ex: 300 }); } catch {}

    return NextResponse.json(result);
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : 'Failed to analyze sentiment';
    return NextResponse.json({ error: message }, { status: 500 });
  }
}
