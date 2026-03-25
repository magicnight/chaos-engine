import { TopBar } from '@/components/layout/top-bar';
import { CategoryPills } from '@/components/layout/category-pills';
import { SummaryBar } from '@/components/portfolio/summary-bar';
import { HeroCard } from '@/components/cards/hero-card';
import { MarketMiniCard } from '@/components/cards/market-mini-card';
import { NewsPredictionCard } from '@/components/cards/news-prediction-card';
import { QuickPollCard } from '@/components/cards/quick-poll-card';
import { BreakingBanner } from '@/components/cards/breaking-banner';
import { ResolvedCard } from '@/components/cards/resolved-card';

const MOCK = {
  hero: {
    title: 'Fed signals hawkish stance ahead of June meeting',
    category: 'Markets',
    yesPercent: 62,
    noPercent: 38,
    volume: '2.8M',
    isLive: true,
    viewerCount: '12.4K',
  },
  markets: [
    { label: 'Fed Rate', yesPercent: 65, price: '0.62', change: '+5%', icon: 'R' },
    { label: 'BTC 100K', yesPercent: 42, price: '0.41', change: '-3%', icon: 'B' },
    { label: 'NVDA Earnings', yesPercent: 78, price: '0.76', change: '+12%', icon: 'N' },
    { label: 'Trump 2028', yesPercent: 31, price: '0.30', change: '+1%', icon: 'T' },
    { label: 'Oil $90', yesPercent: 55, price: '0.54', change: '-2%', icon: 'O' },
  ],
  trending: [
    {
      title: 'Will Ukraine ceasefire hold through summer 2026?',
      category: 'Conflict',
      yesPercent: 34,
      noPercent: 66,
      isHot: true,
    },
    {
      title: 'Apple to announce AR glasses at WWDC?',
      category: 'Tech',
      yesPercent: 70,
      noPercent: 30,
    },
    {
      title: 'EU carbon tax expansion by Q3?',
      category: 'Climate',
      yesPercent: 58,
      noPercent: 42,
    },
  ],
  quickPoll: {
    question: 'Will BTC hit $120K before July 2026?',
    yesPrice: 0.67,
    noPrice: 0.33,
    traderCount: '8.2K',
  },
  breaking: {
    headline: 'Apple Q4 results beat estimates by 15%',
    marketUrl: '/markets',
  },
  resolved: {
    title: 'Election Result: Senate flip confirmed',
    result: 'win' as const,
    amount: '10',
  },
};

export default function HomePage() {
  return (
    <div>
      <TopBar userName="Alex" />
      <SummaryBar totalPnl={142} activePositions={3} winRate={68} />
      <CategoryPills />

      <section className="px-4 mb-6">
        <div className="flex justify-between items-center mb-3">
          <h2 className="text-lg font-bold">Top Stories</h2>
          <span className="text-xs text-[var(--muted)]">Updated 2m ago</span>
        </div>
        <HeroCard {...MOCK.hero} />
      </section>

      <section className="mb-6">
        <h2 className="text-lg font-bold px-4 mb-3">Markets</h2>
        <div className="flex gap-3 px-4 overflow-x-auto no-scrollbar">
          {MOCK.markets.map((m: any) => (
            <MarketMiniCard key={m.label} {...m} />
          ))}
        </div>
      </section>

      <section className="px-4 mb-6">
        <h2 className="text-lg font-bold mb-3">Trending</h2>
        <div className="space-y-3">
          {MOCK.trending.map((t: any) => (
            <NewsPredictionCard key={t.title} {...t} />
          ))}
        </div>
      </section>

      <section className="px-4 mb-6">
        <QuickPollCard {...MOCK.quickPoll} />
      </section>

      <section className="px-4 mb-6">
        <BreakingBanner {...MOCK.breaking} />
      </section>

      <section className="px-4 mb-6">
        <h2 className="text-lg font-bold mb-3">Just Resolved</h2>
        <ResolvedCard {...MOCK.resolved} />
      </section>
    </div>
  );
}
