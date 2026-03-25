'use client';

import { useState } from 'react';
import useSWR from 'swr';
import { CategoryPills } from '@/components/layout/category-pills';
import { NewsPredictionCard } from '@/components/cards/news-prediction-card';
import { useLocale } from '@/lib/i18n/context';

const fetcher = (url: string) => fetch(url).then((r) => r.json());

export default function ExplorePage() {
  const [query, setQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState('All');
  const { t } = useLocale();

  const { data: markets, isLoading } = useSWR('/api/markets?status=open&limit=50', fetcher, {
    refreshInterval: 30000,
    fallbackData: [],
  });

  const filtered = (Array.isArray(markets) ? markets : []).filter((m: any) => {
    const matchesQuery = !query || m.question?.toLowerCase().includes(query.toLowerCase());
    const matchesCategory = selectedCategory === 'All' || m.category === selectedCategory;
    return matchesQuery && matchesCategory;
  }).map((m: any) => ({
    title: m.question || '',
    category: m.category || 'General',
    yesPercent: Math.round((m.yesPrice || 0.5) * 100),
    noPercent: Math.round((m.noPrice || 0.5) * 100),
    isHot: m.volume > 100,
    href: `/markets/${m.id}`,
  }));

  return (
    <div>
      <div className="px-4 pt-4 pb-2">
        <h1 className="text-xl font-bold mb-3">{t('explore.title')}</h1>
        <input
          type="text"
          placeholder={t('explore.searchPlaceholder')}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          className="w-full rounded-xl bg-[var(--card)] border border-[var(--border)] px-4 py-2.5 text-sm text-[var(--foreground)] placeholder-[var(--muted)] outline-none focus:border-[var(--accent)] transition-colors"
        />
      </div>
      <CategoryPills onSelect={setSelectedCategory} />
      <div className="px-4 space-y-3 mt-2">
        {isLoading && (
          <p className="text-sm text-[var(--muted)] text-center py-8">{t('explore.loadingMarkets')}</p>
        )}
        {!isLoading && filtered.length === 0 && (
          <p className="text-sm text-[var(--muted)] text-center py-8">{t('explore.noMarketsFound')}</p>
        )}
        {filtered.map((m: any) => (
          <NewsPredictionCard key={m.title} {...m} />
        ))}
      </div>
    </div>
  );
}
