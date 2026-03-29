'use client';

import { useEffect, useState } from 'react';
import { useLocale } from '@/lib/i18n/context';

interface SentimentData {
  sentiment: string;
  confidence: number;
  summary: string | null;
  bullishCount: number;
  bearishCount: number;
  neutralCount: number;
  commentCount: number;
}

const SENTIMENT_CONFIG: Record<string, { color: string; label: string; labelZh: string }> = {
  bullish: { color: 'var(--success)', label: 'Bullish', labelZh: '看涨' },
  bearish: { color: 'var(--danger)', label: 'Bearish', labelZh: '看跌' },
  mixed: { color: 'var(--warning)', label: 'Mixed', labelZh: '分歧' },
  neutral: { color: 'var(--muted)', label: 'Neutral', labelZh: '中性' },
};

export function SentimentBadge({ marketId }: { marketId: string }) {
  const [data, setData] = useState<SentimentData | null>(null);
  const { locale } = useLocale();

  useEffect(() => {
    fetch(`/api/market-sentiment?marketId=${marketId}`)
      .then((r) => r.json())
      .then((d) => { if (d.summary) setData(d); })
      .catch(() => {});
  }, [marketId]);

  if (!data || !data.summary) return null;

  const config = SENTIMENT_CONFIG[data.sentiment] || SENTIMENT_CONFIG.neutral;
  const label = locale === 'zh' ? config.labelZh : config.label;

  return (
    <div className="bg-[var(--card)] rounded-xl p-4">
      <div className="flex items-center gap-2 mb-2">
        <h3 className="text-sm font-semibold">{locale === 'zh' ? '社区情绪' : 'Community Sentiment'}</h3>
        <span
          className="text-[10px] font-bold px-2 py-0.5 rounded-full"
          style={{ backgroundColor: `${config.color}20`, color: config.color }}
        >
          {label} {Math.round(data.confidence * 100)}%
        </span>
      </div>
      <p className="text-xs text-[var(--foreground-dim)] leading-relaxed mb-2">{data.summary}</p>
      <div className="flex gap-3 text-[10px] text-[var(--muted)]">
        <span style={{ color: 'var(--success)' }}>{data.bullishCount} {locale === 'zh' ? '看涨' : 'bullish'}</span>
        <span style={{ color: 'var(--danger)' }}>{data.bearishCount} {locale === 'zh' ? '看跌' : 'bearish'}</span>
        <span>{data.neutralCount} {locale === 'zh' ? '中性' : 'neutral'}</span>
        <span className="ml-auto">{data.commentCount} {locale === 'zh' ? '条评论' : 'comments'}</span>
      </div>
    </div>
  );
}
