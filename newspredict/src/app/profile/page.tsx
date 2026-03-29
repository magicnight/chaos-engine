'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';
import { useSession } from 'next-auth/react';
import { useLocale } from '@/lib/i18n/context';

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
  const { t, locale } = useLocale();
  const { data: session } = useSession();

  useEffect(() => {
    fetch('/api/portfolio')
      .then((r) => r.json())
      .then((portfolio) => {
        if (portfolio.error) {
          setLoading(false);
          return;
        }
        setData({
          user: {
            id: portfolio.userId || (session?.user as any)?.id || '',
            name: portfolio.userName || session?.user?.name || null,
            avatarUrl: null,
            createdAt: portfolio.createdAt || new Date().toISOString(),
            totalTrades: portfolio.positions?.length || 0,
            winRate: portfolio.winRate || 0,
            pnl: portfolio.totalPnl || 0,
          },
          recentTrades: [],
        });
        setLoading(false);
      })
      .catch(() => setLoading(false));
  }, [session]);

  // Fetch follow counts when user data is available
  useEffect(() => {
    const userId = data?.user?.id;
    if (!userId) return;
    fetch(`/api/follows/list?userId=${userId}&type=followers`)
      .then((r) => r.json())
      .then((d) => setFollowerCount(d.users?.length || 0))
      .catch(() => {});
    fetch(`/api/follows/list?userId=${userId}&type=following`)
      .then((r) => r.json())
      .then((d) => setFollowingCount(d.users?.length || 0))
      .catch(() => {});
  }, [data?.user?.id]);

  if (loading) {
    return (
      <div className="p-4">
        <h1 className="text-2xl font-bold mb-4">{t('profile.title')}</h1>
        <div className="h-40 rounded-xl bg-[var(--card)] animate-pulse" />
      </div>
    );
  }

  if (!data) {
    return (
      <div className="p-4">
        <h1 className="text-2xl font-bold mb-4">{t('profile.title')}</h1>
        <p className="text-[var(--muted)] text-center py-8">
          <Link href="/sign-in" className="text-[var(--accent)] underline">
            {t('common.signIn')}
          </Link>{' '}
          {t('profile.signInPrompt')}
        </p>
      </div>
    );
  }

  const joinDate = new Date(data.user.createdAt).toLocaleDateString(
    locale === 'zh' ? 'zh-CN' : 'en-US',
    { month: 'short', year: 'numeric' }
  );
  const isPositive = data.user.pnl >= 0;

  return (
    <div className="p-4">
      <div className="flex items-center gap-4 mb-6">
        <div className="w-16 h-16 rounded-full bg-[var(--accent-dim)] flex items-center justify-center text-2xl">
          {data.user.name?.[0]?.toUpperCase() || '?'}
        </div>
        <div>
          <h1 className="text-xl font-bold">{data.user.name || t('common.anonymous')}</h1>
          <p className="text-sm text-[var(--muted)]">{t('profile.joined', { date: joinDate })}</p>
        </div>
      </div>

      <div className="flex items-center gap-4 mb-6">
        <span className="text-sm text-[var(--muted)]">
          {data.user.totalTrades} {t('common.trades')}
        </span>
        <span className="text-sm text-[var(--muted)]">{data.user.winRate}% {t('profile.win')}</span>
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
          <p className="text-xs text-[var(--muted)]">{t('profile.following')}</p>
        </div>
        <div className="text-center">
          <p className="text-lg font-bold">{followerCount}</p>
          <p className="text-xs text-[var(--muted)]">{t('profile.followers')}</p>
        </div>
      </div>

      <section>
        <h2 className="text-lg font-bold mb-3">{t('profile.quickLinks')}</h2>
        <div className="space-y-2">
          <Link
            href="/portfolio"
            className="block rounded-xl bg-[var(--card)] p-4 text-sm active:bg-[var(--card-hover)]"
          >
            {t('profile.viewPortfolio')}
          </Link>
          <Link
            href="/leaderboard"
            className="block rounded-xl bg-[var(--card)] p-4 text-sm active:bg-[var(--card-hover)]"
          >
            {t('profile.leaderboard')}
          </Link>
          <Link
            href="/activity"
            className="block rounded-xl bg-[var(--card)] p-4 text-sm active:bg-[var(--card-hover)]"
          >
            {t('profile.activity')}
          </Link>
        </div>
      </section>
    </div>
  );
}
