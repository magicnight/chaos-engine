import { http, createConfig } from 'wagmi';
import { bsc, bscTestnet } from 'wagmi/chains';

// BSC chain config
export const chains = [bsc, bscTestnet] as const;

export const wagmiConfig = createConfig({
  chains,
  transports: {
    [bsc.id]: http(),
    [bscTestnet.id]: http(),
  },
});

// Reown project ID (get from https://cloud.reown.com)
export const REOWN_PROJECT_ID = process.env.NEXT_PUBLIC_REOWN_PROJECT_ID || '';
