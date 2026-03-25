import Link from 'next/link';
import { db } from '@/lib/db';
import { markets } from '@/lib/db/schema';
import { eq, desc } from 'drizzle-orm';
import { getPrice } from '@/lib/market-engine';
import { T } from '@/components/i18n-text';

export const dynamic = 'force-dynamic';

function formatVolume(v: number): string {
  if (v >= 1_000_000) return `$${(v / 1_000_000).toFixed(1)}M`;
  if (v >= 1_000) return `$${(v / 1_000).toFixed(1)}K`;
  return `$${v.toFixed(0)}`;
}

export default async function MarketsPage() {
  const rows = await db
    .select()
    .from(markets)
    .where(eq(markets.status, 'open'))
    .orderBy(desc(markets.createdAt))
    .limit(50);

  const items = rows.map((m: any) => {
    const price = getPrice(Number(m.yesShares), Number(m.noShares), Number(m.liquidityParam));
    return {
      id: m.id,
      question: m.question,
      category: m.category,
      yesPercent: Math.round(price.yes * 100),
      noPercent: Math.round(price.no * 100),
      volume: Number(m.volume),
      traderCount: m.traderCount,
      imageUrl: m.imageUrl,
    };
  });

  return (
    <div>
      <div className="px-4 pt-4 pb-2 flex justify-between items-center">
        <h1 className="text-xl font-bold"><T k="home.marketsSection" /></h1>
        <Link
          href="/create"
          className="px-3 py-1.5 rounded-full bg-[var(--accent)] text-black text-xs font-semibold"
        >
          + <T k="nav.create" />
        </Link>
      </div>

      {items.length === 0 && (
        <p className="text-sm text-[var(--muted)] text-center py-12"><T k="explore.noMarketsFound" /></p>
      )}

      <div className="px-4 space-y-3 mt-2">
        {items.map((m: any) => (
          <Link key={m.id} href={`/markets/${m.id}`} className="block">
            <div className="flex gap-3 bg-[var(--card)] rounded-xl p-3 hover:bg-[var(--card-hover)] transition-colors">
              <div
                className="w-[72px] h-[72px] rounded-lg shrink-0"
                style={{
                  background: m.imageUrl
                    ? `url(${m.imageUrl}) center/cover`
                    : 'linear-gradient(135deg, #1a2332, #0b1220)',
                }}
              />
              <div className="flex-1 min-w-0">
                <p className="text-[10px] text-[var(--accent)] font-medium uppercase tracking-wide mb-0.5">
                  {m.category}
                </p>
                <h4 className="text-sm font-semibold leading-tight line-clamp-2 mb-1.5">
                  {m.question}
                </h4>
                <div className="flex items-center gap-2">
                  <span className="text-xs font-semibold text-[var(--success)]">
                    YES {m.yesPercent}%
                  </span>
                  <span className="text-[var(--border)]">|</span>
                  <span className="text-xs font-semibold text-[var(--danger)]">
                    NO {m.noPercent}%
                  </span>
                  <span className="text-xs text-[var(--muted)] ml-auto">
                    {formatVolume(m.volume)} vol
                  </span>
                </div>
              </div>
            </div>
          </Link>
        ))}
      </div>
    </div>
  );
}
