import { useState, useEffect, useRef } from 'react';

interface LivePrice {
  yesPrice: number;
  noPrice: number;
  volume: number;
  traderCount: number;
}

export function useLivePrice(marketId: string, initialData: LivePrice) {
  const [data, setData] = useState<LivePrice>(initialData);
  const esRef = useRef<EventSource | null>(null);

  useEffect(() => {
    const es = new EventSource(`/api/sse/prices?id=${marketId}`);
    esRef.current = es;

    es.onmessage = (event) => {
      try {
        const parsed = JSON.parse(event.data);
        if (parsed.error) return;
        setData({
          yesPrice: parsed.yesPrice ?? initialData.yesPrice,
          noPrice: parsed.noPrice ?? initialData.noPrice,
          volume: parsed.volume ?? initialData.volume,
          traderCount: parsed.traderCount ?? initialData.traderCount,
        });
      } catch {
        // ignore parse errors
      }
    };

    es.onerror = () => {
      // EventSource auto-reconnects
    };

    return () => {
      es.close();
      esRef.current = null;
    };
  }, [marketId, initialData.yesPrice, initialData.noPrice, initialData.volume, initialData.traderCount]);

  return data;
}
