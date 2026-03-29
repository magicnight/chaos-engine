'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';
import { useLocale } from '@/lib/i18n/context';

interface Notification {
  id: string;
  type: string;
  title: string;
  body: string | null;
  link: string | null;
  read: boolean;
  createdAt: string;
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

const ICON_MAP: Record<string, string> = {
  trade_confirmed: '💰',
  market_resolved: '🏁',
  new_follower: '👤',
};

export default function NotificationsPage() {
  const [items, setItems] = useState<Notification[]>([]);
  const [loading, setLoading] = useState(true);
  const { t } = useLocale();

  useEffect(() => {
    fetch('/api/notifications')
      .then((r) => r.json())
      .then((d) => setItems(d.notifications || []))
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  function markAllRead() {
    fetch('/api/notifications', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ action: 'mark-all-read' }),
    }).then(() => {
      setItems((prev) => prev.map((n) => ({ ...n, read: true })));
    }).catch(() => {});
  }

  function markRead(id: string) {
    fetch('/api/notifications', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ action: 'mark-read', notificationId: id }),
    }).then(() => {
      setItems((prev) => prev.map((n) => n.id === id ? { ...n, read: true } : n));
    }).catch(() => {});
  }

  if (loading) {
    return (
      <div className="p-4">
        <h1 className="text-2xl font-bold mb-4">{t('notifications.title')}</h1>
        <div className="space-y-3">
          {[1, 2, 3].map((i) => (
            <div key={i} className="h-16 rounded-xl bg-[var(--card)] animate-pulse" />
          ))}
        </div>
      </div>
    );
  }

  return (
    <div className="p-4">
      <div className="flex items-center justify-between mb-4">
        <h1 className="text-2xl font-bold">{t('notifications.title')}</h1>
        {items.some((n) => !n.read) && (
          <button
            onClick={markAllRead}
            className="text-xs text-[var(--accent)] hover:underline"
          >
            {t('notifications.markAllRead')}
          </button>
        )}
      </div>

      {items.length === 0 ? (
        <p className="text-[var(--muted)] text-center py-8">{t('notifications.empty')}</p>
      ) : (
        <div className="space-y-2">
          {items.map((n) => {
            const content = (
              <div
                className={`flex items-start gap-3 rounded-xl p-4 transition-colors ${
                  n.read ? 'bg-[var(--card)]' : 'bg-[var(--card)] border border-[var(--accent)]/20'
                }`}
                onClick={() => !n.read && markRead(n.id)}
              >
                <span className="text-lg shrink-0">{ICON_MAP[n.type] || '🔔'}</span>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <p className={`text-sm font-medium ${n.read ? 'text-[var(--muted)]' : ''}`}>
                      {n.title}
                    </p>
                    {!n.read && (
                      <span className="w-2 h-2 rounded-full bg-[var(--accent)] shrink-0" />
                    )}
                  </div>
                  {n.body && (
                    <p className="text-xs text-[var(--muted)] mt-0.5 truncate">{n.body}</p>
                  )}
                </div>
                <span className="text-[10px] text-[var(--muted)] shrink-0">
                  {timeAgoShort(n.createdAt)}
                </span>
              </div>
            );

            return n.link ? (
              <Link key={n.id} href={n.link} className="block">
                {content}
              </Link>
            ) : (
              <div key={n.id}>{content}</div>
            );
          })}
        </div>
      )}
    </div>
  );
}
