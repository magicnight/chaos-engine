import Link from 'next/link';

export function TopBar({ userName }: { userName?: string }) {
  const greeting = getGreeting();
  const date = new Date().toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
  return (
    <header className="flex items-center justify-between px-4 pt-4 pb-2">
      <div>
        <h1 className="text-xl font-bold">
          {greeting}{userName ? `, ${userName}` : ''}
        </h1>
        <p className="text-sm text-[var(--muted)]">{date}</p>
      </div>
      <div className="flex items-center gap-3">
        <Link href="/activity" className="text-xl relative">
          <span className="absolute -top-1 -right-1 w-2 h-2 bg-[var(--danger)] rounded-full" />
        </Link>
        <Link href="/profile" className="w-8 h-8 rounded-full bg-[var(--accent-dim)] flex items-center justify-center text-sm">
          {userName?.[0]?.toUpperCase() || '?'}
        </Link>
      </div>
    </header>
  );
}

function getGreeting() {
  const h = new Date().getHours();
  if (h < 12) return 'Good morning';
  if (h < 18) return 'Good afternoon';
  return 'Good evening';
}
