'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { useLocale } from '@/lib/i18n/context';

export default function CreateMarketPage() {
  const router = useRouter();
  const { t } = useLocale();
  const [question, setQuestion] = useState('');
  const [category, setCategory] = useState('economics');
  const [closeAt, setCloseAt] = useState('');
  const [resolutionCriteria, setResolutionCriteria] = useState('');
  const [resolutionSource, setResolutionSource] = useState('');
  const [description, setDescription] = useState('');
  const [imageUrl, setImageUrl] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const CATEGORIES = [
    { value: 'economics', key: 'categories.markets' },
    { value: 'politics', key: 'categories.politics' },
    { value: 'technology', key: 'categories.tech' },
    { value: 'geopolitics', key: 'categories.conflict' },
    { value: 'environment', key: 'categories.climate' },
    { value: 'health', key: 'categories.health' },
    { value: 'science', key: 'categories.science' },
    { value: 'entertainment', key: 'categories.entertainment' },
    { value: 'sports', key: 'categories.sports' },
    { value: 'other', key: 'categories.other' },
  ];

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!question || !closeAt || !resolutionCriteria) {
      setError(t('create.required'));
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
          imageUrl: imageUrl || undefined,
        }),
      });

      const data = await res.json();
      if (!res.ok) {
        setError(data.error || 'Failed to create market');
        return;
      }

      router.push(`/markets/${data.id}`);
    } catch {
      setError(t('common.networkError'));
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="px-4 pt-4 pb-8">
      <h1 className="text-xl font-bold mb-4">{t('create.title')}</h1>

      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label className="text-xs text-[var(--muted)] mb-1 block">
            {t('create.question')} <span className="text-[var(--danger)]">*</span>
          </label>
          <textarea
            value={question}
            onChange={(e) => setQuestion(e.target.value)}
            placeholder={t('create.questionPlaceholder')}
            rows={2}
            className="w-full rounded-xl bg-[var(--card)] border border-[var(--border)] px-4 py-2.5 text-sm text-[var(--foreground)] placeholder-[var(--muted)] outline-none focus:border-[var(--accent)] transition-colors resize-none"
          />
        </div>

        <div>
          <label className="text-xs text-[var(--muted)] mb-1 block">{t('create.description')}</label>
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder={t('create.descriptionPlaceholder')}
            rows={2}
            className="w-full rounded-xl bg-[var(--card)] border border-[var(--border)] px-4 py-2.5 text-sm text-[var(--foreground)] placeholder-[var(--muted)] outline-none focus:border-[var(--accent)] transition-colors resize-none"
          />
        </div>

        <div>
          <label className="text-xs text-[var(--muted)] mb-1 block">
            {t('create.category')} <span className="text-[var(--danger)]">*</span>
          </label>
          <select
            value={category}
            onChange={(e) => setCategory(e.target.value)}
            className="w-full rounded-xl bg-[var(--card)] border border-[var(--border)] px-4 py-2.5 text-sm text-[var(--foreground)] outline-none focus:border-[var(--accent)] transition-colors"
          >
            {CATEGORIES.map((c: any) => (
              <option key={c.value} value={c.value}>
                {t(c.key)}
              </option>
            ))}
          </select>
        </div>

        <div>
          <label className="text-xs text-[var(--muted)] mb-1 block">
            {t('create.closeDate')} <span className="text-[var(--danger)]">*</span>
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
            {t('create.resolutionCriteria')} <span className="text-[var(--danger)]">*</span>
          </label>
          <textarea
            value={resolutionCriteria}
            onChange={(e) => setResolutionCriteria(e.target.value)}
            placeholder={t('create.resolutionCriteriaPlaceholder')}
            rows={2}
            className="w-full rounded-xl bg-[var(--card)] border border-[var(--border)] px-4 py-2.5 text-sm text-[var(--foreground)] placeholder-[var(--muted)] outline-none focus:border-[var(--accent)] transition-colors resize-none"
          />
        </div>

        <div>
          <label className="text-xs text-[var(--muted)] mb-1 block">{t('create.resolutionSource')}</label>
          <input
            type="text"
            value={resolutionSource}
            onChange={(e) => setResolutionSource(e.target.value)}
            placeholder={t('create.resolutionSourcePlaceholder')}
            className="w-full rounded-xl bg-[var(--card)] border border-[var(--border)] px-4 py-2.5 text-sm text-[var(--foreground)] placeholder-[var(--muted)] outline-none focus:border-[var(--accent)] transition-colors"
          />
        </div>

        <div>
          <label className="text-xs text-[var(--muted)] mb-1 block">{t('create.imageUrl')}</label>
          <input
            type="url"
            value={imageUrl}
            onChange={(e) => setImageUrl(e.target.value)}
            placeholder={t('create.imageUrlPlaceholder')}
            className="w-full rounded-xl bg-[var(--card)] border border-[var(--border)] px-4 py-2.5 text-sm text-[var(--foreground)] placeholder-[var(--muted)] outline-none focus:border-[var(--accent)] transition-colors"
          />
        </div>

        {error && <p className="text-xs text-[var(--danger)]">{error}</p>}

        <button
          type="submit"
          disabled={loading}
          className="w-full py-3 rounded-xl bg-[var(--accent)] text-black font-semibold text-sm hover:opacity-90 transition-opacity disabled:opacity-50"
        >
          {loading ? t('create.creating') : t('create.createMarket')}
        </button>
      </form>
    </div>
  );
}
