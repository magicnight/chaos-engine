import { NextRequest, NextResponse } from 'next/server';
import { db } from '@/lib/db';
import { markets, positions, users } from '@/lib/db/schema';
import { eq, and, lte, sql } from 'drizzle-orm';
import { chaosClient } from '@/lib/chaos-client';

/**
 * GET /api/auto-resolve
 * Automatically resolve expired markets using CHAOS Engine data.
 * Protected by CRON_SECRET — intended to be called periodically.
 */
export async function GET(request: NextRequest) {
  try {
    const cronSecret = request.headers.get('x-cron-secret');
    if (cronSecret !== process.env.CRON_SECRET && cronSecret !== process.env.CHAOS_API_KEY) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    // Find open markets past their close date
    const expiredMarkets = await db
      .select()
      .from(markets)
      .where(and(eq(markets.status, 'open'), lte(markets.closeAt, new Date())));

    if (expiredMarkets.length === 0) {
      return NextResponse.json({ checked: 0, resolved: 0, message: 'No expired markets' });
    }

    const results: { id: string; question: string; result: string; method: string }[] = [];

    for (const market of expiredMarkets) {
      let resolution: 'YES' | 'NO' | null = null;
      let method = 'unknown';

      // Try CHAOS resolve-check if we have resolution source and criteria
      if (market.resolutionSource) {
        try {
          const checkResult = await checkWithChaos(
            market.resolutionSource,
            market.resolutionCriteria
          );
          if (checkResult !== null) {
            resolution = checkResult ? 'YES' : 'NO';
            method = 'chaos-resolve-check';
          }
        } catch {
          // CHAOS check failed, try heuristic
        }
      }

      // Heuristic fallback: if market expired and no CHAOS check available,
      // check based on keyword matching in latest CHAOS data
      if (!resolution) {
        try {
          resolution = await heuristicResolve(market);
          if (resolution) method = 'heuristic';
        } catch {
          // Skip this market
        }
      }

      if (!resolution) continue;

      // Resolve the market (single atomic update)
      await db
        .update(markets)
        .set({
          status: 'resolved',
          resolutionResult: resolution,
          resolvedAt: sql`now()`,
        })
        .where(eq(markets.id, market.id));

      // Payout winners
      const winningSide = resolution;
      await db.execute(sql`
        UPDATE users SET balance = balance + COALESCE((
          SELECT CAST(p.shares AS DECIMAL) FROM positions p
          WHERE p.user_id = users.id AND p.market_id = ${market.id} AND p.side = ${winningSide}
        ), 0), wins = wins + 1
        WHERE id IN (
          SELECT user_id FROM positions WHERE market_id = ${market.id} AND side = ${winningSide}
        )
      `);

      results.push({
        id: market.id,
        question: market.question,
        result: resolution,
        method,
      });
    }

    return NextResponse.json({
      checked: expiredMarkets.length,
      resolved: results.length,
      results,
    });
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : 'Auto-resolve failed';
    return NextResponse.json({ error: message }, { status: 500 });
  }
}

/**
 * Check market condition using CHAOS resolve-check API
 */
async function checkWithChaos(source: string, criteria: string): Promise<boolean | null> {
  // Parse source format: "yfinance:BTC-USD" or "usgs" or "fred:VIXCLS"
  const [sourceName, sourceKey] = source.includes(':')
    ? source.split(':', 2)
    : [source, ''];

  // Map common source names to CHAOS source names
  const chaosSourceMap: Record<string, string> = {
    yfinance: 'YFinance',
    usgs: 'USGS',
    fred: 'FRED',
    acled: 'ACLED',
    'nasa-neo': 'NASA-NEO',
    'cisa-kev': 'CISA-KEV',
    coingecko: 'CoinGecko',
    who: 'WHO',
    gdelt: 'GDELT',
  };

  const chaosSource = chaosSourceMap[sourceName.toLowerCase()] || sourceName;

  // Build condition string from criteria
  // e.g. "BTC-USD price > $75000" → "BTC-USD > 75000"
  const condition = sourceKey
    ? `${sourceKey} ${extractComparison(criteria)}`
    : extractComparison(criteria);

  const CHAOS_URL = process.env.CHAOS_API_URL || 'http://localhost:3117';
  const headers: Record<string, string> = { 'Content-Type': 'application/json' };
  if (process.env.CHAOS_API_KEY) headers['X-CHAOS-Key'] = process.env.CHAOS_API_KEY;

  const res = await fetch(`${CHAOS_URL}/api/v1/resolve-check`, {
    method: 'POST',
    headers,
    body: JSON.stringify({ source: chaosSource, condition }),
  });

  if (!res.ok) return null;

  const data = await res.json();
  if (typeof data.met === 'boolean') return data.met;
  return null;
}

/**
 * Extract comparison from resolution criteria text
 * e.g. "USGS reports M6.0+ earthquake" → "> 6.0"
 * e.g. "BTC-USD price > $100K" → "> 100000"
 */
function extractComparison(criteria: string): string {
  // Look for patterns like "> 75000", "exceed 85", "above 100"
  const gtMatch = criteria.match(/(?:>|exceed|above|more than)\s*\$?([\d,.]+[KMBkmb]?)/i);
  if (gtMatch) {
    const val = parseNumericValue(gtMatch[1]);
    return `> ${val}`;
  }
  const ltMatch = criteria.match(/(?:<|below|under|less than|drop below)\s*\$?([\d,.]+[KMBkmb]?)/i);
  if (ltMatch) {
    const val = parseNumericValue(ltMatch[1]);
    return `< ${val}`;
  }
  return criteria;
}

function parseNumericValue(s: string): number {
  const clean = s.replace(/,/g, '');
  const multipliers: Record<string, number> = { k: 1e3, K: 1e3, m: 1e6, M: 1e6, b: 1e9, B: 1e9 };
  const last = clean[clean.length - 1];
  if (multipliers[last]) {
    return parseFloat(clean.slice(0, -1)) * multipliers[last];
  }
  return parseFloat(clean);
}

/**
 * Heuristic resolve: use CHAOS data to make a best-effort determination
 */
async function heuristicResolve(market: any): Promise<'YES' | 'NO' | null> {
  const data = await chaosClient.getData() as any;
  if (!data?.sources) return null;

  const q = (market.question || '').toLowerCase();
  const criteria = (market.resolutionCriteria || '').toLowerCase();

  // Earthquake markets
  if (q.includes('earthquake') || q.includes('m6')) {
    const usgs = data.sources.USGS;
    if (usgs) {
      const maxMag = usgs.maxMagnitude || 0;
      const magMatch = q.match(/m(\d+\.?\d*)\+/i);
      const threshold = magMatch ? parseFloat(magMatch[1]) : 6.0;
      return maxMag >= threshold ? 'YES' : 'NO';
    }
  }

  // BTC price markets
  if (q.includes('btc') || q.includes('bitcoin')) {
    const yf = data.sources.YFinance;
    const btcPrice = yf?.quotes?.['BTC-USD']?.price;
    if (btcPrice) {
      const priceMatch = q.match(/\$?([\d,]+[KkMm]?)/);
      if (priceMatch) {
        const threshold = parseNumericValue(priceMatch[1]);
        return btcPrice > threshold ? 'YES' : 'NO';
      }
    }
  }

  // Conflict event count markets
  if (q.includes('conflict') || q.includes('acled')) {
    const acled = data.sources.ACLED;
    if (acled) {
      const total = acled.totalEvents || 0;
      const countMatch = q.match(/exceed\s+(\d+)/i);
      if (countMatch) {
        return total > parseInt(countMatch[1]) ? 'YES' : 'NO';
      }
    }
  }

  // NEO count markets
  if (q.includes('near-earth') || q.includes('neo') || q.includes('nasa')) {
    const neo = data.sources['NASA-NEO'];
    if (neo) {
      const total = neo.elementCount || 0;
      const countMatch = q.match(/more than\s+(\d+)/i);
      if (countMatch) {
        return total > parseInt(countMatch[1]) ? 'YES' : 'NO';
      }
    }
  }

  // CISA KEV markets
  if (q.includes('cisa') || q.includes('kev')) {
    const kev = data.sources['CISA-KEV'];
    if (kev) {
      const recent = kev.recentAdditions || 0;
      const countMatch = q.match(/more than\s+(\d+)/i);
      if (countMatch) {
        return recent > parseInt(countMatch[1]) ? 'YES' : 'NO';
      }
    }
  }

  // S&P 500 direction markets
  if (q.includes('s&p') || q.includes('spy')) {
    const yf = data.sources.YFinance;
    const spy = yf?.quotes?.SPY;
    if (spy) {
      const changePct = spy.changePct || spy.change_pct || 0;
      if (q.includes('gain')) return changePct > 1.0 ? 'YES' : 'NO';
      if (q.includes('lose')) return changePct < -1.0 ? 'YES' : 'NO';
    }
  }

  return null;
}
