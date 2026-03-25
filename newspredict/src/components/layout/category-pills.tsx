'use client';
import { useState } from 'react';
import { useLocale } from '@/lib/i18n/context';

const categoryKeys = ['all', 'markets', 'politics', 'tech', 'conflict', 'climate', 'health', 'cyber', 'space'];
const categoryValues = ['All', 'Markets', 'Politics', 'Tech', 'Conflict', 'Climate', 'Health', 'Cyber', 'Space'];

export function CategoryPills({ onSelect }: { onSelect?: (cat: string) => void }) {
  const [active, setActive] = useState('All');
  const { t } = useLocale();
  return (
    <div className="flex gap-2 px-4 py-2 overflow-x-auto no-scrollbar">
      {categoryKeys.map((key, i) => {
        const value = categoryValues[i];
        const label = t(`categories.${key}`);
        return (
          <button
            key={key}
            onClick={() => { setActive(value); onSelect?.(value); }}
            className={`px-3.5 py-1.5 rounded-full text-xs font-medium whitespace-nowrap transition-all ${
              active === value
                ? 'bg-[var(--accent)] text-black shadow-md shadow-[var(--accent)]/20'
                : 'bg-[var(--card)] text-[var(--muted)] border border-[var(--border-subtle)] hover:border-[var(--border)] hover:text-[var(--foreground)]'
            }`}
          >
            {label}
          </button>
        );
      })}
    </div>
  );
}
