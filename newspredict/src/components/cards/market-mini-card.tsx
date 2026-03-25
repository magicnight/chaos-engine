'use client';

import Link from 'next/link';

interface MarketMiniCardProps {
  label: string;
  yesPercent: number;
  price: string;
  change: string;
  icon?: string;
  href?: string;
}

export function MarketMiniCard({
  label,
  yesPercent,
  price,
  change,
  icon,
  href,
}: MarketMiniCardProps) {
  const noPercent = 100 - yesPercent;
  const isPositive = change.startsWith('+') || change.startsWith('$');
  const Wrapper = href ? Link : 'div';
  const wrapperProps = href ? { href } : {};

  return (
    <Wrapper {...wrapperProps as any} className="w-[130px] shrink-0 card p-3 animate-fade-in block">
      <p className="text-[10px] text-[var(--muted)] mb-2 truncate font-medium">
        {icon && <span className="mr-1 text-[var(--accent)]">{icon}</span>}
        {label}
      </p>
      <div className="relative w-14 h-14 mx-auto mb-2">
        <svg viewBox="0 0 36 36" className="w-full h-full -rotate-90">
          <circle cx="18" cy="18" r="15.5" fill="none" stroke="var(--border-subtle)" strokeWidth="3" />
          <circle
            cx="18" cy="18" r="15.5" fill="none"
            stroke="var(--accent)" strokeWidth="3"
            strokeDasharray={`${yesPercent} ${noPercent}`}
            strokeLinecap="round"
            className="transition-all duration-700"
          />
        </svg>
        <span className="absolute inset-0 flex items-center justify-center text-[10px] font-bold font-mono">
          {yesPercent}%
        </span>
      </div>
      <div className="flex items-center justify-between text-xs">
        <span className="font-bold font-mono">${price}</span>
        <span className={`font-mono text-[10px] ${isPositive ? 'text-[var(--success)]' : 'text-[var(--danger)]'}`}>
          {change}
        </span>
      </div>
    </Wrapper>
  );
}
