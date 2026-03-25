'use client';
import { useAccount, useChainId } from 'wagmi';
import { bsc, bscTestnet } from 'wagmi/chains';

export function ChainBadge() {
  const { isConnected } = useAccount();
  const chainId = useChainId();

  if (!isConnected) return null;

  const chain = chainId === bsc.id ? bsc : chainId === bscTestnet.id ? bscTestnet : null;
  const isTestnet = chainId === bscTestnet.id;

  return (
    <span className={`text-xs px-2 py-0.5 rounded-full ${
      isTestnet ? 'bg-yellow-900/50 text-yellow-400' : 'bg-green-900/50 text-green-400'
    }`}>
      {chain?.name || `Chain ${chainId}`}
    </span>
  );
}
