'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';

const CATEGORIES = [
  { value: 'economics', label: 'Markets' },
  { value: 'politics', label: 'Politics' },
  { value: 'technology', label: 'Tech' },
  { value: 'geopolitics', label: 'Conflict' },
  { value: 'environment', label: 'Climate' },
  { value: 'health', label: 'Health' },
  { value: 'science', label: 'Science' },
  { value: 'entertainment', label: 'Entertainment' },
  { value: 'sports', label: 'Sports' },
  { value: 'other', label: 'Other' },
];

export default function CreateMarketPage() {
  const router = useRouter();
  const [question, setQuestion] = useState('');
  const [category, setCategory] = useState('economics');
  const [closeAt, setCloseAt] = useState('');
  const [resolutionCriteria, setResolutionCriteria] = useState('');
  const [resolutionSource, setResolutionSource] = useState('');
  const [description, setDescription] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!question || !closeAt || !resolutionCriteria) {
      setError('Please fill in all required fields');
      return;
    }

    setLoading(true);
    setError('');

    try {
      const res = await fetch('/api/markets', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          question,
          category,
          closeAt,
          resolutionCriteria,
          resolutionSource: resolutionSource || undefined,
          description: description || undefined,
        }),
      });

      const data = await res.json();
      if (!res.ok) {
        setError(data.error || 'Failed to create market');
        return;
      }

      router.push(`/markets/${data.id}`);
    } catch {
      setError('Network error');
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="px-4 pt-4 pb-8">
      <h1 className="text-xl font-bold mb-4">Create Market</h1>

      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label className="text-xs text-[var(--muted)] mb-1 block">
            Question <span className="text-[var(--danger)]">*</span>
          </label>
          <textarea
            value={question}
            onChange={(e) => setQuestion(e.target.value)}
            placeholder="Will X happen by Y date?"
            rows={2}
            className="w-full rounded-xl bg-[var(--card)] border border-[var(--border)] px-4 py-2.5 text-sm text-[var(--foreground)] placeholder-[var(--muted)] outline-none focus:border-[var(--accent)] transition-colors resize-none"
          />
        </div>

        <div>
          <label className="text-xs text-[var(--muted)] mb-1 block">Description</label>
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="Additional context for this market..."
            rows={2}
            className="w-full rounded-xl bg-[var(--card)] border border-[var(--border)] px-4 py-2.5 text-sm text-[var(--foreground)] placeholder-[var(--muted)] outline-none focus:border-[var(--accent)] transition-colors resize-none"
          />
        </div>

        <div>
          <label className="text-xs text-[var(--muted)] mb-1 block">
            Category <span className="text-[var(--danger)]">*</span>
          </label>
          <select
            value={category}
            onChange={(e) => setCategory(e.target.value)}
            className="w-full rounded-xl bg-[var(--card)] border border-[var(--border)] px-4 py-2.5 text-sm text-[var(--foreground)] outline-none focus:border-[var(--accent)] transition-colors"
          >
            {CATEGORIES.map((c: any) => (
              <option key={c.value} value={c.value}>
                {c.label}
              </option>
            ))}
          </select>
        </div>

        <div>
          <label className="text-xs text-[var(--muted)] mb-1 block">
            Close Date <span className="text-[var(--danger)]">*</span>
          </label>
          <input
            type="datetime-local"
            value={closeAt}
            onChange={(e) => setCloseAt(e.target.value)}
            className="w-full rounded-xl bg-[var(--card)] border border-[var(--border)] px-4 py-2.5 text-sm text-[var(--foreground)] outline-none focus:border-[var(--accent)] transition-colors"
          />
        </div>

        <div>
          <label className="text-xs text-[var(--muted)] mb-1 block">
            Resolution Criteria <span className="text-[var(--danger)]">*</span>
          </label>
          <textarea
            value={resolutionCriteria}
            onChange={(e) => setResolutionCriteria(e.target.value)}
            placeholder="How will this market be resolved? (e.g., 'BTC price above $100K on CoinGecko')"
            rows={2}
            className="w-full rounded-xl bg-[var(--card)] border border-[var(--border)] px-4 py-2.5 text-sm text-[var(--foreground)] placeholder-[var(--muted)] outline-none focus:border-[var(--accent)] transition-colors resize-none"
          />
        </div>

        <div>
          <label className="text-xs text-[var(--muted)] mb-1 block">Resolution Source</label>
          <input
            type="text"
            value={resolutionSource}
            onChange={(e) => setResolutionSource(e.target.value)}
            placeholder="e.g., CoinGecko, Reuters, NOAA"
            className="w-full rounded-xl bg-[var(--card)] border border-[var(--border)] px-4 py-2.5 text-sm text-[var(--foreground)] placeholder-[var(--muted)] outline-none focus:border-[var(--accent)] transition-colors"
          />
        </div>

        {error && <p className="text-xs text-[var(--danger)]">{error}</p>}

        <button
          type="submit"
          disabled={loading}
          className="w-full py-3 rounded-xl bg-[var(--accent)] text-black font-semibold text-sm hover:opacity-90 transition-opacity disabled:opacity-50"
        >
          {loading ? 'Creating...' : 'Create Market'}
        </button>
      </form>
    </div>
  );
}
