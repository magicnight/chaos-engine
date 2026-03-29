import { http, createConfig, fallback } from 'wagmi';
import { bsc, bscTestnet } from 'wagmi/chains';

// BSC Mainnet RPC: Alchemy → Ankr → BSCScan (public)
const ALCHEMY_KEY = process.env.NEXT_PUBLIC_ALCHEMY_API_KEY || 'lIlW8P7vF9-Qmn8hKxbmu';
const ANKR_KEY = process.env.NEXT_PUBLIC_ANKR_API_KEY || '';

const bscTransport = fallback([
  http(`https://bnb-mainnet.g.alchemy.com/v2/${ALCHEMY_KEY}`),
  http(ANKR_KEY ? `https://rpc.ankr.com/bsc/${ANKR_KEY}` : 'https://rpc.ankr.com/bsc'),
  http('https://bsc-dataseed.bnbchain.org'),
]);

const bscTestTransport = fallback([
  http('https://data-seed-prebsc-1-s1.bnbchain.org:8545'),
  http('https://rpc.ankr.com/bsc_testnet_chapel'),
]);

export const chains = [bsc, bscTestnet] as const;

export const wagmiConfig = createConfig({
  chains,
  transports: {
    [bsc.id]: bscTransport,
    [bscTestnet.id]: bscTestTransport,
  },
});

export const REOWN_PROJECT_ID = process.env.NEXT_PUBLIC_REOWN_PROJECT_ID || '';
