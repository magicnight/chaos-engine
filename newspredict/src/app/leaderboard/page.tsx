'use client';

import { useEffect, useState } from 'react';
import { useLocale } from '@/lib/i18n/context';

interface RankingEntry {
  userId: string;
  name: string | null;
  avatar: string | null;
  pnl: number;
  winRate: number;
  totalTrades: number;
}

const PERIODS = ['daily', 'weekly', 'alltime'] as const;
type Period = (typeof PERIODS)[number];

const PERIOD_KEYS: Record<Period, string> = {
  daily: 'leaderboard.daily',
  weekly: 'leaderboard.weekly',
  alltime: 'leaderboard.allTime',
};

export default function LeaderboardPage() {
  const [period, setPeriod] = useState<Period>('weekly');
  const [rankings, setRankings] = useState<RankingEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const { t } = useLocale();

  useEffect(() => {
    setLoading(true);
    fetch(`/api/leaderboard?period=${period}`)
      .then((r) => r.json())
      .then((d) => setRankings(d.rankings || []))
      .catch(() => setRankings([]))
      .finally(() => setLoading(false));
  }, [period]);

  return (
    <div className="p-4">
      <h1 className="text-2xl font-bold mb-4">{t('leaderboard.title')}</h1>

      <div className="flex gap-2 mb-6">
        {PERIODS.map((p) => (
          <button
            key={p}
            onClick={() => setPeriod(p)}
            className={`px-4 py-2 rounded-full text-sm font-medium transition-colors ${
              period === p
                ? 'bg-[var(--accent)] text-black'
                : 'bg-[var(--card)] text-[var(--muted)]'
            }`}
          >
            {t(PERIOD_KEYS[p])}
          </button>
        ))}
      </div>

      {loading ? (
        <div className="space-y-3">
          {[1, 2, 3, 4, 5].map((i: any) => (
            <div key={i} className="h-16 rounded-xl bg-[var(--card)] animate-pulse" />
          ))}
        </div>
      ) : rankings.length === 0 ? (
        <p className="text-[var(--muted)] text-center py-8">{t('leaderboard.noTraders')}</p>
      ) : (
        <div className="space-y-2">
          {rankings.map((entry, i) => {
            const rank = i + 1;
            const medal = rank === 1 ? '1st' : rank === 2 ? '2nd' : rank === 3 ? '3rd' : null;
            const isPositive = entry.pnl >= 0;

            return (
              <div
                key={entry.userId}
                className={`flex items-center gap-3 rounded-xl p-4 ${
                  rank <= 3 ? 'bg-[var(--card)] border border-[var(--warning)]/10' : 'bg-[var(--card)]'
                }`}
              >
                <span
                  className={`w-8 text-center font-bold text-sm ${
                    rank <= 3 ? 'text-[var(--warning)]' : 'text-[var(--muted)]'
                  }`}
                >
                  {medal || rank}
                </span>
                <div className="w-8 h-8 rounded-full bg-[var(--accent-dim)] flex items-center justify-center text-sm flex-shrink-0">
                  {entry.name?.[0]?.toUpperCase() || '?'}
                </div>
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium truncate">
                    {entry.name || t('common.anonymous')}
                  </p>
                  <p className="text-xs text-[var(--muted)]">
                    {entry.totalTrades} {t('common.trades')}
                  </p>
                </div>
                <div className="text-right">
                  <p
                    className={`text-sm font-bold ${
                      isPositive ? 'text-[var(--success)]' : 'text-[var(--danger)]'
                    }`}
                  >
                    {isPositive ? '+' : ''}${entry.pnl.toFixed(0)}
                  </p>
                  <p className="text-xs text-[var(--muted)]">{entry.winRate}% {t('leaderboard.winRateShort')}</p>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
