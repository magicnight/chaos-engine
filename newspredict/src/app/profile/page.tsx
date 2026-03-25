'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';

interface ProfileData {
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
}

export default function ProfilePage() {
  const [data, setData] = useState<ProfileData | null>(null);
  const [loading, setLoading] = useState(true);
  const [followerCount, setFollowerCount] = useState(0);
  const [followingCount, setFollowingCount] = useState(0);

  useEffect(() => {
    fetch('/api/portfolio')
      .then((r) => r.json())
      .then((portfolio) => {
        if (portfolio.error) {
          setLoading(false);
          return;
        }
        // Build profile from portfolio data
        setData({
          user: {
            id: '',
            name: null,
            avatarUrl: null,
            createdAt: new Date().toISOString(),
            totalTrades: portfolio.positions?.length || 0,
            winRate: portfolio.winRate || 0,
            pnl: portfolio.totalPnl || 0,
          },
          recentTrades: [],
        });
        setLoading(false);
      })
      .catch(() => setLoading(false));
  }, []);

  if (loading) {
    return (
      <div className="p-4">
        <h1 className="text-2xl font-bold mb-4">Profile</h1>
        <div className="h-40 rounded-xl bg-[var(--card)] animate-pulse" />
      </div>
    );
  }

  if (!data) {
    return (
      <div className="p-4">
        <h1 className="text-2xl font-bold mb-4">Profile</h1>
        <p className="text-[var(--muted)] text-center py-8">
          <Link href="/sign-in" className="text-[var(--accent)] underline">
            Sign in
          </Link>{' '}
          to view your profile
        </p>
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
      <div className="flex items-center gap-4 mb-6">
        <div className="w-16 h-16 rounded-full bg-[var(--accent-dim)] flex items-center justify-center text-2xl">
          {data.user.name?.[0]?.toUpperCase() || '?'}
        </div>
        <div>
          <h1 className="text-xl font-bold">{data.user.name || 'Anonymous'}</h1>
          <p className="text-sm text-[var(--muted)]">Joined {joinDate}</p>
        </div>
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

      <div className="flex gap-6 mb-6">
        <div className="text-center">
          <p className="text-lg font-bold">{followingCount}</p>
          <p className="text-xs text-[var(--muted)]">Following</p>
        </div>
        <div className="text-center">
          <p className="text-lg font-bold">{followerCount}</p>
          <p className="text-xs text-[var(--muted)]">Followers</p>
        </div>
      </div>

      <section>
        <h2 className="text-lg font-bold mb-3">Quick Links</h2>
        <div className="space-y-2">
          <Link
            href="/portfolio"
            className="block rounded-xl bg-[var(--card)] p-4 text-sm active:bg-[var(--card-hover)]"
          >
            View Portfolio
          </Link>
          <Link
            href="/leaderboard"
            className="block rounded-xl bg-[var(--card)] p-4 text-sm active:bg-[var(--card-hover)]"
          >
            Leaderboard
          </Link>
          <Link
            href="/activity"
            className="block rounded-xl bg-[var(--card)] p-4 text-sm active:bg-[var(--card-hover)]"
          >
            Activity
          </Link>
        </div>
      </section>
    </div>
  );
}
