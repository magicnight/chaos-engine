'use client';
import { useState } from 'react';

const categories = ['All', 'Markets', 'Politics', 'Tech', 'Conflict', 'Climate', 'Health', 'Cyber', 'Space'];

export function CategoryPills({ onSelect }: { onSelect?: (cat: string) => void }) {
  const [active, setActive] = useState('All');
  return (
    <div className="flex gap-2 px-4 py-2 overflow-x-auto no-scrollbar">
      {categories.map((cat: any) => (
        <button
          key={cat}
          onClick={() => { setActive(cat); onSelect?.(cat); }}
          className={`px-3 py-1.5 rounded-full text-sm whitespace-nowrap transition-colors ${
            active === cat
              ? 'bg-white text-black font-medium'
              : 'bg-[var(--card)] text-[var(--muted)] hover:text-[var(--foreground)]'
          }`}
        >
          {cat}
        </button>
      ))}
    </div>
  );
}
