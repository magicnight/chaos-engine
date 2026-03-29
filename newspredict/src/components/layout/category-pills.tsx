'use client';
import { useState } from 'react';
import { useLocale } from '@/lib/i18n/context';
import { CategoryIcon } from '@/components/ui/category-icon';

// Display key → i18n key, filter value, DB category (for icon)
const categories = [
  { key: 'all',            db: 'All' },
  { key: 'markets',        db: 'economics' },
  { key: 'politics',       db: 'politics' },
  { key: 'tech',           db: 'technology' },
  { key: 'conflict',       db: 'geopolitics' },
  { key: 'climate',        db: 'environment' },
  { key: 'health',         db: 'health' },
  { key: 'science',        db: 'science' },
  { key: 'entertainment',  db: 'entertainment' },
  { key: 'sports',         db: 'sports' },
  { key: 'other',          db: 'other' },
];

export function CategoryPills({ onSelect }: { onSelect?: (cat: string) => void }) {
  const [active, setActive] = useState('All');
  const { t } = useLocale();
  return (
    <div className="flex gap-2 px-4 py-2 overflow-x-auto no-scrollbar">
      {categories.map((cat) => {
        const label = t(`categories.${cat.key}`);
        return (
          <button
            key={cat.key}
            onClick={() => { setActive(cat.db); onSelect?.(cat.db); }}
            className={`flex items-center gap-1.5 px-3.5 py-1.5 rounded-full text-xs font-medium whitespace-nowrap transition-all ${
              active === cat.db
                ? 'bg-[var(--accent)] text-black shadow-md shadow-[var(--accent)]/20'
                : 'bg-[var(--card)] text-[var(--muted)] border border-[var(--border-subtle)] hover:border-[var(--border)] hover:text-[var(--foreground)]'
            }`}
          >
            {cat.key !== 'all' && <CategoryIcon category={cat.db} size="sm" />}
            {label}
          </button>
        );
      })}
    </div>
  );
}
