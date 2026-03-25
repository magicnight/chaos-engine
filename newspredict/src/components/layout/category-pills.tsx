'use client';
import { useState } from 'react';

const categories = ['All', 'Markets', 'Politics', 'Tech', 'Conflict', 'Climate', 'Health', 'Cyber', 'Space'];

export function CategoryPills({ onSelect }: { onSelect?: (cat: string) => void }) {
  const [active, setActive] = useState('All');
  return (
    <div className="flex gap-2 px-4 py-2 overflow-x-auto no-scrollbar">
      {categories.map((cat) => (
        <button
          key={cat}
          onClick={() => { setActive(cat); onSelect?.(cat); }}
          className={`px-3.5 py-1.5 rounded-full text-xs font-medium whitespace-nowrap transition-all ${
            active === cat
              ? 'bg-[var(--accent)] text-black shadow-md shadow-[var(--accent)]/20'
              : 'bg-[var(--card)] text-[var(--muted)] border border-[var(--border-subtle)] hover:border-[var(--border)] hover:text-[var(--foreground)]'
          }`}
        >
          {cat}
        </button>
      ))}
    </div>
  );
}
