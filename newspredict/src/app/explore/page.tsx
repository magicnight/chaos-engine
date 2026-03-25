'use client';

import { useState } from 'react';
import { CategoryPills } from '@/components/layout/category-pills';
import { NewsPredictionCard } from '@/components/cards/news-prediction-card';

const MOCK_MARKETS = [
  { title: 'Will the Fed cut rates in June 2026?', category: 'Markets', yesPercent: 62, noPercent: 38 },
  { title: 'Will China invade Taiwan by 2027?', category: 'Conflict', yesPercent: 12, noPercent: 88 },
  { title: 'Will GPT-5 launch before September?', category: 'Tech', yesPercent: 74, noPercent: 26, isHot: true },
  { title: 'Will oil prices exceed $100/barrel?', category: 'Markets', yesPercent: 45, noPercent: 55 },
  { title: 'Will WHO declare new pandemic by 2027?', category: 'Health', yesPercent: 18, noPercent: 82 },
  { title: 'Will SpaceX Starship reach orbit this quarter?', category: 'Space', yesPercent: 81, noPercent: 19, isHot: true },
  { title: 'Will EU pass AI Act enforcement rules?', category: 'Politics', yesPercent: 67, noPercent: 33 },
  { title: 'Major ransomware attack on US infrastructure?', category: 'Cyber', yesPercent: 39, noPercent: 61 },
];

export default function ExplorePage() {
  const [query, setQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState('All');

  const filtered = MOCK_MARKETS.filter((m: any) => {
    const matchesQuery = !query || m.title.toLowerCase().includes(query.toLowerCase());
    const matchesCategory = selectedCategory === 'All' || m.category === selectedCategory;
    return matchesQuery && matchesCategory;
  });

  return (
    <div>
      <div className="px-4 pt-4 pb-2">
        <h1 className="text-xl font-bold mb-3">Explore</h1>
        <input
          type="text"
          placeholder="Search markets..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          className="w-full rounded-xl bg-[var(--card)] border border-[var(--border)] px-4 py-2.5 text-sm text-[var(--foreground)] placeholder-[var(--muted)] outline-none focus:border-[var(--accent)] transition-colors"
        />
      </div>
      <CategoryPills onSelect={setSelectedCategory} />
      <div className="px-4 space-y-3 mt-2">
        {filtered.length === 0 && (
          <p className="text-sm text-[var(--muted)] text-center py-8">No markets found</p>
        )}
        {filtered.map((m: any) => (
          <NewsPredictionCard key={m.title} {...m} />
        ))}
      </div>
    </div>
  );
}
