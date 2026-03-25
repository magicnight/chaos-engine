interface MarketStatsProps {
  volume: number;
  traderCount: number;
  closeAt: string;
  resolutionCriteria: string;
  resolutionSource?: string | null;
}

function formatVolume(v: number): string {
  if (v >= 1_000_000) return `$${(v / 1_000_000).toFixed(1)}M`;
  if (v >= 1_000) return `$${(v / 1_000).toFixed(1)}K`;
  return `$${v.toFixed(0)}`;
}

function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  });
}

export function MarketStats({
  volume,
  traderCount,
  closeAt,
  resolutionCriteria,
  resolutionSource,
}: MarketStatsProps) {
  return (
    <div className="bg-[var(--card)] rounded-xl p-4 space-y-3">
      <div className="flex justify-between text-sm">
        <span className="text-[var(--muted)]">Volume</span>
        <span className="font-semibold">{formatVolume(volume)}</span>
      </div>
      <div className="flex justify-between text-sm">
        <span className="text-[var(--muted)]">Traders</span>
        <span className="font-semibold">{traderCount.toLocaleString()}</span>
      </div>
      <div className="flex justify-between text-sm">
        <span className="text-[var(--muted)]">Closes</span>
        <span className="font-semibold">{formatDate(closeAt)}</span>
      </div>
      <div className="border-t border-[var(--border)] pt-3">
        <p className="text-xs text-[var(--muted)] mb-1">Resolution criteria</p>
        <p className="text-sm">{resolutionCriteria}</p>
        {resolutionSource && (
          <p className="text-xs text-[var(--accent)] mt-1">Source: {resolutionSource}</p>
        )}
      </div>
    </div>
  );
}
