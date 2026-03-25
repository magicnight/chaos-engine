'use client';

import Link from 'next/link';
import { useLocale } from '@/lib/i18n/context';

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
  const { t } = useLocale();
  const Wrapper = href ? Link : 'div';
  const wrapperProps = href ? { href } : {};
  return (
    <Wrapper {...wrapperProps as any} className="block rounded-2xl overflow-hidden card-glow animate-fade-in">
      <div
        className="relative h-44 flex items-end p-4"
        style={{
          background: imageUrl
            ? `url(${imageUrl}) center/cover`
            : 'linear-gradient(135deg, #0f2027 0%, #203a43 40%, #2c5364 100%)',
        }}
      >
        <div className="absolute inset-0 bg-gradient-to-t from-black/90 via-black/40 to-transparent" />
        <div className="relative z-10 w-full">
          <div className="flex items-center gap-2 mb-1.5">
            {isLive && (
              <span className="badge badge-live">
                <span className="w-1.5 h-1.5 rounded-full bg-[var(--danger)] animate-pulse" />
                {t('common.live')}
              </span>
            )}
            <span className="badge badge-new">{category}</span>
          </div>
          <h3 className="text-base font-bold leading-tight line-clamp-2">{title}</h3>
        </div>
      </div>
      <div className="bg-[var(--card)] px-4 py-3 space-y-2.5">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <span className="text-sm font-bold text-[var(--success)]">
              YES {yesPercent}%
            </span>
            <span className="text-[var(--border)]">|</span>
            <span className="text-sm font-bold text-[var(--danger)]">
              NO {noPercent}%
            </span>
          </div>
          <div className="flex items-center gap-2 text-[10px] text-[var(--muted)] font-mono">
            {viewerCount && <span>{viewerCount}</span>}
            <span>{volume} vol</span>
          </div>
        </div>
        <div className="pct-bar">
          <div
            className="pct-bar-fill bg-[var(--success)]"
            style={{ width: `${yesPercent}%` }}
          />
        </div>
      </div>
    </Wrapper>
  );
}
