import useSWR from 'swr';

const fetcher = (url: string) => fetch(url).then((r) => r.json());

interface LivePrice {
  yesPrice: number;
  noPrice: number;
  volume: number;
  traderCount: number;
}

export function useLivePrice(marketId: string, initialData: LivePrice) {
  const { data } = useSWR(`/api/markets?id=${marketId}`, fetcher, {
    refreshInterval: 10000,
    fallbackData: initialData,
    revalidateOnFocus: true,
  });

  return {
    yesPrice: data?.yesPrice ?? initialData.yesPrice,
    noPrice: data?.noPrice ?? initialData.noPrice,
    volume: data?.volume ?? initialData.volume,
    traderCount: data?.traderCount ?? initialData.traderCount,
  };
}
