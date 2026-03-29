import { Suspense } from 'react';
import { TopBar } from '@/components/layout/top-bar';
import { CategoryPills } from '@/components/layout/category-pills';
import { SummaryBar } from '@/components/portfolio/summary-bar';
import { HeroCard } from '@/components/cards/hero-card';
import { MarketMiniCard } from '@/components/cards/market-mini-card';
import { NewsPredictionCard } from '@/components/cards/news-prediction-card';
import { QuickPollCard } from '@/components/cards/quick-poll-card';
import { BreakingBanner } from '@/components/cards/breaking-banner';
import { ResolvedCard } from '@/components/cards/resolved-card';
import { chaosClient } from '@/lib/chaos-client';
import { T } from '@/components/i18n-text';
import { getCategoryImage } from '@/lib/category-image';
import { db } from '@/lib/db';
import { markets } from '@/lib/db/schema';
import { eq, desc } from 'drizzle-orm';
import { getPrice } from '@/lib/market-engine';

export const revalidate = 15;

async function getMarkets(status: string, limit: number) {
  try {
    const rows = await db
      .select()
      .from(markets)
      .where(eq(markets.status, status))
      .orderBy(desc(markets.createdAt))
      .limit(limit);
    return rows.map((m: any) => {
      const price = getPrice(Number(m.yesShares), Number(m.noShares), Number(m.liquidityParam));
      return {
        id: m.id, question: m.question, category: m.category, status: m.status,
        imageUrl: m.imageUrl, yesPrice: price.yes, noPrice: price.no,
        volume: Number(m.volume), traderCount: m.traderCount,
        resolutionResult: m.resolutionResult,
      };
    });
  } catch {
    return [];
  }
}

async function getBreakingNews() {
  try {
    const data = (await chaosClient.getEvents()) as any;
    const events = data?.events || [];
    if (events.length === 0) return [];
    return events.slice(0, 3).map((e: any) => ({
      headline: e.title || e.headline || 'Intelligence event',
      marketUrl: '/explore',
    }));
  } catch {
    return [];
  }
}

export default async function HomePage() {
  const [openMarkets, resolvedMarkets, breaking] = await Promise.all([
    getMarkets('open', 20),
    getMarkets('resolved', 3),
    getBreakingNews(),
  ]);

  const marketList = Array.isArray(openMarkets) ? openMarkets : [];
  const resolved = Array.isArray(resolvedMarkets) ? resolvedMarkets : [];

  const hero = marketList[0] || null;
  const miniCards = marketList.slice(0, 5).map((m: any) => ({
    label: (m.question || '').substring(0, 20),
    yesPercent: Math.round((m.yesPrice || 0.5) * 100),
    price: (m.yesPrice || 0.5).toFixed(2),
    change: m.volume > 0 ? `$${m.volume}` : 'New',
    icon: (m.category || 'M')[0],
    href: `/markets/${m.id}`,
  }));
  const trending = marketList.slice(0, 4).map((m: any) => ({
    title: m.question || '',
    category: m.category || 'General',
    yesPercent: Math.round((m.yesPrice || 0.5) * 100),
    noPercent: Math.round((m.noPrice || 0.5) * 100),
    isHot: m.volume > 100,
    href: `/markets/${m.id}`,
    imageUrl: m.imageUrl || getCategoryImage(m.category || 'other', m.id),
  }));
  const quickPoll = marketList[1] || marketList[0] || null;
  const resolvedCard = resolved[0] || null;

  const hasData = marketList.length > 0;

  return (
    <div>
      <TopBar />
      <SummaryBar totalPnl={0} activePositions={marketList.length} winRate={0} />
      <CategoryPills />

      <section className="px-4 mb-6">
        <div className="flex justify-between items-center mb-3">
          <h2 className="text-lg font-bold"><T k="home.topStories" /></h2>
          <span className="text-xs text-[var(--muted)]">
            {hasData ? <T k="home.activeMarkets" vars={{ n: marketList.length }} /> : <T k="home.waitingForData" />}
          </span>
        </div>
        {hero ? (
          <HeroCard
            title={hero.question}
            category={hero.category || 'General'}
            yesPercent={Math.round((hero.yesPrice || 0.5) * 100)}
            noPercent={Math.round((hero.noPrice || 0.5) * 100)}
            volume={hero.volume > 0 ? `$${hero.volume}` : '$0'}
            isLive={hero.status === 'open'}
            imageUrl={hero.imageUrl || getCategoryImage(hero.category || 'other', hero.id)}
            href={`/markets/${hero.id}`}
          />
        ) : (
          <div className="rounded-xl border border-[var(--border)] p-6 text-center text-[var(--muted)] text-sm">
            <T k="home.noMarketsYet" /> <code className="text-xs">curl -X GET localhost:3000/api/market-seeds</code>
          </div>
        )}
      </section>

      {miniCards.length > 0 && (
        <section className="mb-6">
          <h2 className="text-lg font-bold px-4 mb-3"><T k="home.marketsSection" /></h2>
          <div className="flex gap-3 px-4 overflow-x-auto no-scrollbar">
            {miniCards.map((m: any) => (
              <MarketMiniCard key={m.label} {...m} />
            ))}
          </div>
        </section>
      )}

      {trending.length > 0 && (
        <section className="px-4 mb-6">
          <h2 className="text-lg font-bold mb-3"><T k="home.trending" /></h2>
          <div className="space-y-3">
            {trending.map((t: any) => (
              <NewsPredictionCard key={t.title} {...t} />
            ))}
          </div>
        </section>
      )}

      {quickPoll && (
        <section className="px-4 mb-6">
          <QuickPollCard
            question={quickPoll.question}
            yesPrice={quickPoll.yesPrice || 0.5}
            noPrice={quickPoll.noPrice || 0.5}
            traderCount={`${quickPoll.traderCount || 0}`}
          />
        </section>
      )}

      {breaking.length > 0 && (
        <section className="px-4 mb-6 space-y-2">
          {breaking.map((b: any, i: number) => (
            <BreakingBanner key={i} headline={b.headline} marketUrl={b.marketUrl} />
          ))}
        </section>
      )}

      {resolvedCard && (
        <section className="px-4 mb-6">
          <h2 className="text-lg font-bold mb-3"><T k="home.justResolved" /></h2>
          <ResolvedCard
            title={resolvedCard.question}
            result={resolvedCard.resolutionResult === 'YES' ? 'win' : 'loss'}
            amount={resolvedCard.volume > 0 ? Math.round(resolvedCard.volume).toString() : '0'}
          />
        </section>
      )}
    </div>
  );
}
