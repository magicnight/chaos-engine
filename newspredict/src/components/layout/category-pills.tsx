'use client';
import { useState } from 'react';
import { useLocale } from '@/lib/i18n/context';
import { CategoryIcon } from '@/components/ui/category-icon';

// Display key → i18n key, filter value, DB category (for icon)
const categories = [
  { key: 'all',      value: 'All',      db: '' },
  { key: 'markets',  value: 'Markets',  db: 'economics' },
  { key: 'politics', value: 'Politics', db: 'politics' },
  { key: 'tech',     value: 'Tech',     db: 'technology' },
  { key: 'conflict', value: 'Conflict', db: 'geopolitics' },
  { key: 'climate',  value: 'Climate',  db: 'environment' },
  { key: 'health',   value: 'Health',   db: 'health' },
  { key: 'cyber',    value: 'Cyber',    db: 'technology' },
  { key: 'space',    value: 'Space',    db: 'science' },
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
            onClick={() => { setActive(cat.value); onSelect?.(cat.value); }}
            className={`flex items-center gap-1.5 px-3.5 py-1.5 rounded-full text-xs font-medium whitespace-nowrap transition-all ${
              active === cat.value
                ? 'bg-[var(--accent)] text-black shadow-md shadow-[var(--accent)]/20'
                : 'bg-[var(--card)] text-[var(--muted)] border border-[var(--border-subtle)] hover:border-[var(--border)] hover:text-[var(--foreground)]'
            }`}
          >
            {cat.db && <CategoryIcon category={cat.db} size="sm" />}
            {label}
          </button>
        );
      })}
    </div>
  );
}
