'use client';
import { usePathname } from 'next/navigation';
import Link from 'next/link';
import { useLocale } from '@/lib/i18n/context';

function HomeIcon({ active }: { active: boolean }) {
  return (
    <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={active ? 2.5 : 1.8} strokeLinecap="round" strokeLinejoin="round">
      <path d="M3 9.5L12 3l9 6.5V20a1 1 0 01-1 1H4a1 1 0 01-1-1V9.5z" />
      <path d="M9 21V12h6v9" />
    </svg>
  );
}

function ExploreIcon({ active }: { active: boolean }) {
  return (
    <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={active ? 2.5 : 1.8} strokeLinecap="round" strokeLinejoin="round">
      <circle cx="11" cy="11" r="8" />
      <path d="M21 21l-4.35-4.35" />
    </svg>
  );
}

function ActivityIcon({ active }: { active: boolean }) {
  return (
    <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={active ? 2.5 : 1.8} strokeLinecap="round" strokeLinejoin="round">
      <polyline points="22 12 18 12 15 21 9 3 6 12 2 12" />
    </svg>
  );
}

function ProfileIcon({ active }: { active: boolean }) {
  return (
    <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={active ? 2.5 : 1.8} strokeLinecap="round" strokeLinejoin="round">
      <path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2" />
      <circle cx="12" cy="7" r="4" />
    </svg>
  );
}

function PlusIcon() {
  return (
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round">
      <line x1="12" y1="5" x2="12" y2="19" />
      <line x1="5" y1="12" x2="19" y2="12" />
    </svg>
  );
}

const tabKeys = [
  { href: '/', key: 'nav.home', Icon: HomeIcon },
  { href: '/explore', key: 'nav.explore', Icon: ExploreIcon },
  { href: '/create', key: 'nav.create', Icon: PlusIcon, isCreate: true },
  { href: '/activity', key: 'nav.activity', Icon: ActivityIcon },
  { href: '/profile', key: 'nav.profile', Icon: ProfileIcon },
];

export function BottomNav() {
  const pathname = usePathname();
  const { t } = useLocale();
  return (
    <nav role="navigation" aria-label="Main navigation" className="fixed bottom-0 left-0 right-0 glass border-t border-[var(--border)] z-50">
      <div className="flex items-center justify-around h-16 max-w-lg mx-auto">
        {tabKeys.map((tab) => {
          const active = pathname === tab.href;
          const label = t(tab.key);
          if (tab.isCreate) {
            return (
              <Link
                key={tab.href}
                href={tab.href}
                aria-label={label}
                className="bg-[var(--accent)] text-black w-12 h-12 rounded-full flex items-center justify-center -mt-4 shadow-lg shadow-[var(--accent)]/20 hover:shadow-[var(--accent)]/40 active:scale-95 transition-all"
              >
                <PlusIcon />
              </Link>
            );
          }
          return (
            <Link
              key={tab.href}
              href={tab.href}
              aria-label={label}
              className={`flex flex-col items-center gap-0.5 transition-colors ${
                active ? 'text-[var(--accent)]' : 'text-[var(--muted)] hover:text-[var(--foreground-dim)]'
              }`}
            >
              <tab.Icon active={active} />
              <span className="text-[10px] font-medium">{label}</span>
            </Link>
          );
        })}
      </div>
    </nav>
  );
}
