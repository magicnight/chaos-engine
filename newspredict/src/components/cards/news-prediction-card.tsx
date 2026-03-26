'use client';

import Link from 'next/link';
import { useLocale } from '@/lib/i18n/context';
import { CategoryIcon } from '@/components/ui/category-icon';

interface NewsPredictionCardProps {
  title: string;
  category: string;
  yesPercent: number;
  noPercent: number;
  isHot?: boolean;
  imageUrl?: string;
  href?: string;
}

export function NewsPredictionCard({
  title,
  category,
  yesPercent,
  noPercent,
  isHot,
  imageUrl,
  href,
}: NewsPredictionCardProps) {
  const { t } = useLocale();
  const Wrapper = href ? Link : 'div';
  const wrapperProps = href ? { href } : {};
  return (
    <Wrapper {...wrapperProps as any} className="flex gap-3 card p-3 animate-fade-in block">
      <div
        className="w-[72px] h-[72px] rounded-lg shrink-0 relative overflow-hidden"
        style={{
          background: imageUrl
            ? `url(${imageUrl}) center/cover`
            : 'linear-gradient(135deg, #1a2332, #0b1220)',
        }}
      >
        <div className="w-full h-full rounded-lg flex items-center justify-center">
          {!imageUrl && <CategoryIcon category={category} size="lg" />}
        </div>
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-1.5 mb-0.5">
          <CategoryIcon category={category} size="sm" />
          <span className="text-[10px] text-[var(--accent)] font-semibold uppercase tracking-wide">
            {category}
          </span>
          {isHot && <span className="badge badge-hot">{t('common.hot')}</span>}
        </div>
        <h4 className="text-sm font-semibold leading-tight line-clamp-2 mb-2">{title}</h4>
        <div className="flex items-center gap-2 mb-1.5">
          <span className="text-xs font-bold text-[var(--success)] font-mono">YES {yesPercent}%</span>
          <span className="text-[var(--border)]">|</span>
          <span className="text-xs font-bold text-[var(--danger)] font-mono">NO {noPercent}%</span>
        </div>
        <div className="pct-bar">
          <div className="pct-bar-fill bg-[var(--success)]" style={{ width: `${yesPercent}%` }} />
        </div>
      </div>
    </Wrapper>
  );
}
