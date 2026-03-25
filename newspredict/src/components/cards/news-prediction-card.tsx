'use client';

interface NewsPredictionCardProps {
  title: string;
  category: string;
  yesPercent: number;
  noPercent: number;
  isHot?: boolean;
  imageUrl?: string;
}

export function NewsPredictionCard({
  title,
  category,
  yesPercent,
  noPercent,
  isHot,
  imageUrl,
}: NewsPredictionCardProps) {
  return (
    <div className="flex gap-3 bg-[var(--card)] rounded-xl p-3">
      <div
        className="w-[72px] h-[72px] rounded-lg shrink-0"
        style={{
          background: imageUrl
            ? `url(${imageUrl}) center/cover`
            : 'linear-gradient(135deg, #1a2332, #0b1220)',
        }}
      />
      <div className="flex-1 min-w-0">
        <p className="text-[10px] text-[var(--accent)] font-medium uppercase tracking-wide mb-0.5">
          {category}
        </p>
        <h4 className="text-sm font-semibold leading-tight line-clamp-2 mb-1.5">{title}</h4>
        <div className="flex items-center gap-2">
          <span className="text-xs font-semibold text-[var(--success)]">YES {yesPercent}%</span>
          <span className="text-[var(--border)]">|</span>
          <span className="text-xs font-semibold text-[var(--danger)]">NO {noPercent}%</span>
          {isHot && <span className="text-xs ml-auto">Hot</span>}
        </div>
      </div>
    </div>
  );
}
