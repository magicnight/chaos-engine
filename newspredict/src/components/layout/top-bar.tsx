'use client';

import Link from 'next/link';
import { useLocale } from '@/lib/i18n/context';

export function TopBar({ userName }: { userName?: string }) {
  const { t, locale, setLocale } = useLocale();
  const greeting = getGreeting(t);
  const date = new Date().toLocaleDateString(locale === 'zh' ? 'zh-CN' : 'en-US', {
    weekday: 'short',
    month: 'short',
    day: 'numeric',
  });
  return (
    <header className="flex items-center justify-between px-4 pt-4 pb-2">
      <div>
        <div className="flex items-center gap-2 mb-0.5">
          <span className="text-gradient font-bold text-xs tracking-widest">C.H.A.O.S.</span>
        </div>
        <h1 className="text-lg font-bold leading-tight">
          {greeting}{userName ? `, ${userName}` : ''}
        </h1>
        <p className="text-xs text-[var(--muted)]">{date}</p>
      </div>
      <div className="flex items-center gap-2">
        <button
          onClick={() => setLocale(locale === 'en' ? 'zh' : 'en')}
          className="h-8 px-2 rounded-full bg-[var(--card)] border border-[var(--border)] flex items-center justify-center text-[10px] font-bold text-[var(--muted)] hover:border-[var(--accent)]/30 hover:text-[var(--foreground)] transition-colors"
          aria-label="Switch language"
        >
          {locale === 'en' ? '中文' : 'EN'}
        </button>
        <Link href="/activity" className="relative w-9 h-9 rounded-full bg-[var(--card)] border border-[var(--border)] flex items-center justify-center hover:border-[var(--accent)]/30 transition-colors">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18s-3-2-3-9" />
            <path d="M13.73 21a2 2 0 01-3.46 0" />
          </svg>
          <span className="absolute -top-0.5 -right-0.5 w-2.5 h-2.5 bg-[var(--danger)] rounded-full border-2 border-[var(--background)]" />
        </Link>
        <Link href="/profile" className="w-9 h-9 rounded-full bg-gradient-to-br from-[var(--accent)] to-[var(--accent-dim)] flex items-center justify-center text-sm font-bold text-black shadow-md shadow-[var(--accent)]/20">
          {userName?.[0]?.toUpperCase() || '?'}
        </Link>
      </div>
    </header>
  );
}

function getGreeting(t: (key: string) => string) {
  const h = new Date().getHours();
  if (h < 12) return t('topBar.goodMorning');
  if (h < 18) return t('topBar.goodAfternoon');
  return t('topBar.goodEvening');
}
