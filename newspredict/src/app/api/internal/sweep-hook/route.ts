import { NextRequest, NextResponse } from 'next/server';
import { auth } from '@/lib/auth';

/**
 * POST /api/internal/sweep-hook
 * Called by client-side SSE listener when CHAOS sweep completes.
 * Triggers market seeding and auto-resolution using server-side secrets.
 * Requires valid session (logged-in user) or x-cron-secret header.
 */
export async function POST(request: NextRequest) {
  const cronSecret = request.headers.get('x-cron-secret');
  const session = await auth();
  if (cronSecret !== process.env.CRON_SECRET && !session?.user?.id) {
    return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
  }

  const baseUrl = process.env.NEXTAUTH_URL || 'http://localhost:3000';
  const secret = process.env.CRON_SECRET || '';

  const results = { seeded: false, resolved: false };

  try {
    const seedRes = await fetch(`${baseUrl}/api/market-seeds`, {
      headers: { 'x-cron-secret': secret },
    });
    if (seedRes.ok) {
      const data = await seedRes.json();
      results.seeded = (data.created || 0) > 0;
    }
  } catch {}

  try {
    const resolveRes = await fetch(`${baseUrl}/api/auto-resolve`, {
      headers: { 'x-cron-secret': secret },
    });
    if (resolveRes.ok) results.resolved = true;
  } catch {}

  return NextResponse.json(results);
}
