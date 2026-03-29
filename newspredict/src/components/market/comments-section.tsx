'use client';

import { useState } from 'react';
import useSWR from 'swr';
import { useSession } from 'next-auth/react';
import { useLocale } from '@/lib/i18n/context';

const fetcher = (url: string) => fetch(url).then((r) => r.json());

interface Comment {
  id: string;
  content: string;
  createdAt: string;
  user: { id: string; name: string | null; avatarUrl: string | null };
}

function timeAgoShort(iso: string): string {
  const diff = Date.now() - new Date(iso).getTime();
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return '<1m';
  if (mins < 60) return `${mins}m`;
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return `${hrs}h`;
  return `${Math.floor(hrs / 24)}d`;
}

export function CommentsSection({ marketId }: { marketId: string }) {
  const { t } = useLocale();
  const { data: session } = useSession();
  const [input, setInput] = useState('');
  const [posting, setPosting] = useState(false);
  const [error, setError] = useState('');

  const { data: comments, mutate } = useSWR<Comment[]>(
    `/api/comments?marketId=${marketId}`,
    fetcher,
    { refreshInterval: 30000, fallbackData: [] }
  );

  async function handlePost() {
    if (!input.trim() || posting) return;
    setPosting(true);
    setError('');
    try {
      const res = await fetch('/api/comments', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ marketId, content: input.trim() }),
      });
      if (res.ok) {
        setInput('');
        mutate();
      } else {
        const data = await res.json().catch(() => ({}));
        setError(data.error || t('common.error'));
      }
    } catch {
      setError(t('common.networkError'));
    } finally {
      setPosting(false);
    }
  }

  const list = Array.isArray(comments) ? comments : [];

  return (
    <div className="bg-[var(--card)] rounded-xl p-4">
      <h3 className="text-sm font-semibold mb-3">{t('comments.title')}</h3>

      {session?.user ? (
        <div className="flex gap-2 mb-3">
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handlePost()}
            placeholder={t('comments.placeholder')}
            maxLength={2000}
            className="flex-1 rounded-lg bg-[var(--background)] border border-[var(--border)] px-3 py-2 text-xs text-[var(--foreground)] placeholder-[var(--muted)] outline-none focus:border-[var(--accent)] transition-colors"
          />
          <button
            onClick={handlePost}
            disabled={posting || !input.trim()}
            className="px-3 py-2 rounded-lg bg-[var(--accent)] text-black text-xs font-semibold hover:opacity-90 transition-opacity disabled:opacity-50"
          >
            {t('comments.submit')}
          </button>
        </div>
      ) : (
        <p className="text-xs text-[var(--muted)] mb-3">{t('comments.signInToComment')}</p>
      )}

      {error && <p className="text-xs text-[var(--danger)] mb-2">{error}</p>}

      {list.length === 0 ? (
        <p className="text-xs text-[var(--muted)]">{t('comments.noComments')}</p>
      ) : (
        <div className="space-y-2.5">
          {list.map((c) => (
            <div key={c.id} className="flex gap-2">
              <div className="w-6 h-6 rounded-full bg-[var(--accent-dim)] flex items-center justify-center text-[10px] shrink-0">
                {c.user.name?.[0]?.toUpperCase() || '?'}
              </div>
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="text-xs font-medium">{c.user.name || 'Anonymous'}</span>
                  <span className="text-[10px] text-[var(--muted)]">{timeAgoShort(c.createdAt)}</span>
                </div>
                <p className="text-xs text-[var(--foreground-dim)] leading-relaxed">{c.content}</p>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
