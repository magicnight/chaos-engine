import { NextRequest } from 'next/server';
import { db } from '@/lib/db';
import { markets } from '@/lib/db/schema';
import { eq } from 'drizzle-orm';
import { getPrice } from '@/lib/market-engine';

export const dynamic = 'force-dynamic';

export async function GET(request: NextRequest) {
  const marketId = request.nextUrl.searchParams.get('id');
  if (!marketId) {
    return new Response('Missing id parameter', { status: 400 });
  }

  const id = marketId;
  const encoder = new TextEncoder();
  let closed = false;

  const stream = new ReadableStream({
    async start(controller) {
      let lastYes = -1;
      let lastNo = -1;

      async function poll() {
        if (closed) return;
        try {
          const [market] = await db.select().from(markets).where(eq(markets.id, id));
          if (!market) {
            controller.enqueue(encoder.encode('data: {"error":"not_found"}\n\n'));
            controller.close();
            return;
          }
          const price = getPrice(Number(market.yesShares), Number(market.noShares), Number(market.liquidityParam));
          // Only send if price changed
          if (price.yes !== lastYes || price.no !== lastNo) {
            lastYes = price.yes;
            lastNo = price.no;
            const data = JSON.stringify({
              yesPrice: price.yes,
              noPrice: price.no,
              volume: Number(market.volume),
              traderCount: market.traderCount,
              status: market.status,
            });
            controller.enqueue(encoder.encode(`data: ${data}\n\n`));
          }
        } catch {
          // DB error — skip this tick
        }
        if (!closed) {
          setTimeout(poll, 3000);
        }
      }

      // Send initial data immediately
      poll();
    },
    cancel() {
      closed = true;
    },
  });

  return new Response(stream, {
    headers: {
      'Content-Type': 'text/event-stream',
      'Cache-Control': 'no-cache, no-transform',
      Connection: 'keep-alive',
    },
  });
}
