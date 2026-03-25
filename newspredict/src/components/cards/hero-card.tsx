'use client';

import Link from 'next/link';

interface HeroCardProps {
  title: string;
  category: string;
  yesPercent: number;
  noPercent: number;
  volume: string;
  isLive?: boolean;
  viewerCount?: string;
  imageUrl?: string;
  href?: string;
}

export function HeroCard({
  title,
  category,
  yesPercent,
  noPercent,
  volume,
  isLive,
  viewerCount,
  imageUrl,
  href,
}: HeroCardProps) {
  const Wrapper = href ? Link : 'div';
  const wrapperProps = href ? { href } : {};
  return (
    <Wrapper {...wrapperProps as any} className="block rounded-2xl overflow-hidden bg-[var(--card)] hover:ring-1 hover:ring-[var(--accent)]/30 transition-all">
      <div
        className="relative h-40 flex items-end p-4"
        style={{
          background: imageUrl
            ? `url(${imageUrl}) center/cover`
            : 'linear-gradient(135deg, #1a2332 0%, #0b1220 100%)',
        }}
      >
        <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-black/30 to-transparent" />
        <div className="relative z-10 w-full">
          {isLive && (
            <span className="inline-flex items-center gap-1 text-xs font-semibold text-[var(--danger)] mb-1">
              <span className="w-1.5 h-1.5 rounded-full bg-[var(--danger)] animate-pulse" />
              LIVE
            </span>
          )}
          <p className="text-xs text-[var(--accent)] font-medium uppercase tracking-wide mb-0.5">
            {category}
          </p>
          <h3 className="text-base font-bold leading-tight">{title}</h3>
        </div>
      </div>
      <div className="px-4 py-3 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <span className="text-sm font-semibold text-[var(--success)]">
            YES {yesPercent}%
          </span>
          <span className="text-[var(--border)]">|</span>
          <span className="text-sm font-semibold text-[var(--danger)]">
            NO {noPercent}%
          </span>
        </div>
        <div className="flex items-center gap-2 text-xs text-[var(--muted)]">
          {viewerCount && <span>{viewerCount} watching</span>}
          <span>${volume} vol</span>
        </div>
      </div>
    </Wrapper>
  );
}
