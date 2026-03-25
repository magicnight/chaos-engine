'use client';
import { usePathname } from 'next/navigation';
import Link from 'next/link';

const tabs = [
  { href: '/', label: 'Home', icon: 'H' },
  { href: '/explore', label: 'Explore', icon: 'S' },
  { href: '/create', label: '', icon: '+' },
  { href: '/activity', label: 'Activity', icon: 'A' },
  { href: '/profile', label: 'Profile', icon: 'P' },
];

export function BottomNav() {
  const pathname = usePathname();
  return (
    <nav role="navigation" aria-label="Main navigation" className="fixed bottom-0 left-0 right-0 bg-[var(--card)] border-t border-[var(--border)] z-50">
      <div className="flex items-center justify-around h-16 max-w-lg mx-auto">
        {tabs.map((tab: any) => {
          const active = pathname === tab.href;
          const isCreate = tab.href === '/create';
          return (
            <Link
              key={tab.href}
              href={tab.href}
              aria-label={isCreate ? 'Create market' : tab.label}
              className={`flex flex-col items-center gap-0.5 text-xs ${
                isCreate
                  ? 'bg-[var(--accent)] text-black w-12 h-12 rounded-full flex items-center justify-center text-xl -mt-4'
                  : active
                  ? 'text-[var(--accent)]'
                  : 'text-[var(--muted)]'
              }`}
            >
              <span className={isCreate ? 'text-xl' : 'text-lg'}>{tab.icon}</span>
              {!isCreate && <span>{tab.label}</span>}
            </Link>
          );
        })}
      </div>
    </nav>
  );
}
