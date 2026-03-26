import { NextRequest, NextResponse } from 'next/server';
import { auth } from '@/lib/auth';
import { executeTrade } from '@/lib/trade-executor';
import { db } from '@/lib/db';
import { trades } from '@/lib/db/schema';
import { eq, and } from 'drizzle-orm';

const UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
const TX_HASH_RE = /^0x[0-9a-f]{64}$/i;

export async function POST(request: NextRequest) {
  try {
    const session = await auth();
    if (!session?.user?.id) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const body = await request.json();
    const { marketId, side, amount } = body;

    if (!marketId || !side || !amount) {
      return NextResponse.json({ error: 'Missing required fields' }, { status: 400 });
    }

    if (!UUID_RE.test(marketId)) {
      return NextResponse.json({ error: 'Invalid market ID' }, { status: 400 });
    }

    if (side !== 'YES' && side !== 'NO') {
      return NextResponse.json({ error: 'Side must be YES or NO' }, { status: 400 });
    }

    if (typeof amount !== 'number' || amount <= 0 || amount > 10000) {
      return NextResponse.json({ error: 'Amount must be between 0 and 10000' }, { status: 400 });
    }

    const result = await executeTrade(session.user.id, marketId, side, amount);

    if (!result.success) {
      return NextResponse.json({ error: result.error }, { status: 400 });
    }

    return NextResponse.json(result);
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : 'Trade failed';
    return NextResponse.json({ error: message }, { status: 500 });
  }
}

export async function PATCH(request: NextRequest) {
  try {
    const session = await auth();
    if (!session?.user?.id) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const body = await request.json();
    const { tradeId, txHash } = body;

    if (!tradeId || !UUID_RE.test(tradeId)) {
      return NextResponse.json({ error: 'Invalid trade ID' }, { status: 400 });
    }
    if (!txHash || !TX_HASH_RE.test(txHash)) {
      return NextResponse.json({ error: 'Invalid transaction hash' }, { status: 400 });
    }

    await db
      .update(trades)
      .set({ txHash })
      .where(and(eq(trades.id, tradeId), eq(trades.userId, session.user.id)));

    return NextResponse.json({ success: true });
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : 'Failed to update trade';
    return NextResponse.json({ error: message }, { status: 500 });
  }
}
