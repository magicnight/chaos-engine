'use client';

import { useEffect, useState } from 'react';
import { useParams } from 'next/navigation';
import Link from 'next/link';

interface UserProfile {
  user: {
    id: string;
    name: string | null;
    avatarUrl: string | null;
    createdAt: string;
    totalTrades: number;
    winRate: number;
    pnl: number;
  };
  recentTrades: {
    id: string;
    side: string;
    shares: number;
    price: number;
    cost: number;
    createdAt: string;
    market: { id: string; question: string; category: string };
  }[];
  positions: {
    id: string;
    side: string;
    shares: number;
    currentPrice: number;
    market: { id: string; question: string; category: string };
  }[];
}

export default function UserProfilePage() {
  const params = useParams();
  const userId = params.userId as string;
  const [data, setData] = useState<UserProfile | null>(null);
  const [loading, setLoading] = useState(true);
  const [following, setFollowing] = useState(false);
  const [actionLoading, setActionLoading] = useState(false);

  useEffect(() => {
    fetch(`/api/users/${userId}`)
      .then((r) => r.json())
      .then((d) => {
        if (!d.error) setData(d);
      })
      .catch(() => {})
      .finally(() => setLoading(false));
  }, [userId]);

  async function handleFollow() {
    setActionLoading(true);
    try {
      const action = following ? 'unfollow' : 'follow';
      const res = await fetch('/api/follows', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ targetUserId: userId, action }),
      });
      const result = await res.json();
      if (result.success) {
        setFollowing(!following);
      }
    } catch {
      // Silently fail
    } finally {
      setActionLoading(false);
    }
  }

  if (loading) {
    return (
      <div className="p-4">
        <div className="h-40 rounded-xl bg-[var(--card)] animate-pulse" />
      </div>
    );
  }

  if (!data) {
    return (
      <div className="p-4">
        <p className="text-[var(--muted)] text-center py-8">User not found</p>
      </div>
    );
  }

  const joinDate = new Date(data.user.createdAt).toLocaleDateString('en-US', {
    month: 'short',
    year: 'numeric',
  });
  const isPositive = data.user.pnl >= 0;

  return (
    <div className="p-4">
      <div className="flex items-center gap-4 mb-4">
        <div className="w-16 h-16 rounded-full bg-[var(--accent-dim)] flex items-center justify-center text-2xl">
          {data.user.name?.[0]?.toUpperCase() || '?'}
        </div>
        <div className="flex-1">
          <h1 className="text-xl font-bold">{data.user.name || 'Anonymous'}</h1>
          <p className="text-sm text-[var(--muted)]">Joined {joinDate}</p>
        </div>
        <button
          onClick={handleFollow}
          disabled={actionLoading}
          className={`px-4 py-2 rounded-full text-sm font-medium transition-colors ${
            following
              ? 'bg-[var(--card)] text-[var(--foreground)] border border-[var(--border)]'
              : 'bg-[var(--accent)] text-black'
          }`}
        >
          {actionLoading ? '...' : following ? 'Following' : 'Follow'}
        </button>
      </div>

      <div className="flex items-center gap-4 mb-6">
        <span className="text-sm text-[var(--muted)]">
          {data.user.totalTrades} trades
        </span>
        <span className="text-sm text-[var(--muted)]">{data.user.winRate}% win</span>
        <span
          className={`text-sm font-bold ${
            isPositive ? 'text-[var(--success)]' : 'text-[var(--danger)]'
          }`}
        >
          {isPositive ? '+' : ''}${data.user.pnl.toFixed(0)}
        </span>
      </div>

      {data.recentTrades.length > 0 && (
        <section className="mb-6">
          <h2 className="text-lg font-bold mb-3">Recent Activity</h2>
          <div className="space-y-2">
            {data.recentTrades.map((t: any) => (
              <Link
                key={t.id}
                href={`/markets/${t.market.id}`}
                className="block rounded-xl bg-[var(--card)] p-3 text-sm active:bg-[var(--card-hover)]"
              >
                <span className={t.side === 'YES' ? 'text-[var(--success)]' : 'text-[var(--danger)]'}>
                  {t.side}
                </span>{' '}
                on &quot;{t.market.question.slice(0, 50)}...&quot; @ ${t.price.toFixed(2)}
              </Link>
            ))}
          </div>
        </section>
      )}

      {data.positions.length > 0 && (
        <section>
          <h2 className="text-lg font-bold mb-3">Open Positions</h2>
          <div className="space-y-2">
            {data.positions.map((p: any) => (
              <Link
                key={p.id}
                href={`/markets/${p.market.id}`}
                className="block rounded-xl bg-[var(--card)] p-3 active:bg-[var(--card-hover)]"
              >
                <p className="text-sm font-medium mb-1">{p.market.question}</p>
                <p className="text-xs text-[var(--muted)]">
                  <span className={p.side === 'YES' ? 'text-[var(--success)]' : 'text-[var(--danger)]'}>
                    {p.side}
                  </span>{' '}
                  {p.shares.toFixed(1)} shares @ ${p.currentPrice.toFixed(2)}
                </p>
              </Link>
            ))}
          </div>
        </section>
      )}
    </div>
  );
}
