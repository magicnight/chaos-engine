// Copyright (c) 2026 ChaosDevOps@BKK&Estonia. All rights reserved.

import { useAccount, useBalance, useReadContract, useWriteContract, useWaitForTransactionReceipt } from 'wagmi';
import { formatUnits, parseUnits } from 'viem';
import { CHAOS_TOKEN_ABI, CHAOS_MARKET_ABI, CONTRACTS } from './contracts';

export function useWalletInfo() {
  const { address, isConnected, chain } = useAccount();
  const { data: balance } = useBalance({ address });

  return {
    address,
    isConnected,
    chain,
    balance: balance
      ? `${parseFloat(formatUnits(balance.value, balance.decimals)).toFixed(4)} ${balance.symbol}`
      : null,
  };
}

export function useChaosBalance() {
  const { address, chain } = useAccount();
  const contracts = chain ? CONTRACTS[chain.id] : null;

  const { data: balance, refetch } = useReadContract({
    address: contracts?.token as `0x${string}`,
    abi: CHAOS_TOKEN_ABI,
    functionName: 'balanceOf',
    args: address ? [address] : undefined,
    query: { enabled: !!address && !!contracts?.token },
  });

  return { balance, refetch };
}

// Backward compat alias
export const useCruxBalance = useChaosBalance;

export function useApproveToken() {
  const { writeContract, data: hash, isPending } = useWriteContract();
  const { isLoading: isConfirming, isSuccess } = useWaitForTransactionReceipt({ hash });

  const approve = (spender: string, amount: string, chainId: number) => {
    const contracts = CONTRACTS[chainId];
    if (!contracts?.token) return;

    writeContract({
      address: contracts.token as `0x${string}`,
      abi: CHAOS_TOKEN_ABI,
      functionName: 'approve',
      args: [spender as `0x${string}`, parseUnits(amount, 18)],
    });
  };

  return { approve, isPending, isConfirming, isSuccess, hash };
}

export function useBuyShares() {
  const { writeContract, data: hash, isPending } = useWriteContract();
  const { isLoading: isConfirming, isSuccess } = useWaitForTransactionReceipt({ hash });

  const buy = (marketId: number, side: 0 | 1, shares: string, maxCost: string, chainId: number) => {
    const contracts = CONTRACTS[chainId];
    if (!contracts?.market) return;

    writeContract({
      address: contracts.market as `0x${string}`,
      abi: CHAOS_MARKET_ABI,
      functionName: 'buyShares',
      args: [BigInt(marketId), side, parseUnits(shares, 18), parseUnits(maxCost, 18)],
    });
  };

  return { buy, isPending, isConfirming, isSuccess, hash };
}

export function useSellShares() {
  const { writeContract, data: hash, isPending } = useWriteContract();
  const { isLoading: isConfirming, isSuccess } = useWaitForTransactionReceipt({ hash });

  const sell = (marketId: number, side: 0 | 1, shares: string, minProceeds: string, chainId: number) => {
    const contracts = CONTRACTS[chainId];
    if (!contracts?.market) return;

    writeContract({
      address: contracts.market as `0x${string}`,
      abi: CHAOS_MARKET_ABI,
      functionName: 'sellShares',
      args: [BigInt(marketId), side, parseUnits(shares, 18), parseUnits(minProceeds, 18)],
    });
  };

  return { sell, isPending, isConfirming, isSuccess, hash };
}

export function useClaimWinnings() {
  const { writeContract, data: hash, isPending } = useWriteContract();
  const { isLoading: isConfirming, isSuccess } = useWaitForTransactionReceipt({ hash });

  const claim = (marketId: number, chainId: number) => {
    const contracts = CONTRACTS[chainId];
    if (!contracts?.market) return;

    writeContract({
      address: contracts.market as `0x${string}`,
      abi: CHAOS_MARKET_ABI,
      functionName: 'claimWinnings',
      args: [BigInt(marketId)],
    });
  };

  return { claim, isPending, isConfirming, isSuccess, hash };
}

export function useMarketPrice(marketId: number) {
  const { chain } = useAccount();
  const contracts = chain ? CONTRACTS[chain.id] : null;

  const { data: yesPrice } = useReadContract({
    address: contracts?.market as `0x${string}`,
    abi: CHAOS_MARKET_ABI,
    functionName: 'getYesPrice',
    args: [BigInt(marketId)],
    query: { enabled: !!contracts?.market },
  });

  return {
    yesPrice: yesPrice ? Number(formatUnits(yesPrice as bigint, 18)) : 0.5,
    noPrice: yesPrice ? 1 - Number(formatUnits(yesPrice as bigint, 18)) : 0.5,
  };
}

export function useBuyCost(marketId: number, side: 0 | 1, shares: string) {
  const { chain } = useAccount();
  const contracts = chain ? CONTRACTS[chain.id] : null;

  const { data: cost } = useReadContract({
    address: contracts?.market as `0x${string}`,
    abi: CHAOS_MARKET_ABI,
    functionName: 'calculateBuyCost',
    args: [BigInt(marketId), side, parseUnits(shares || '0', 18)],
    query: { enabled: !!contracts?.market && !!shares && parseFloat(shares) > 0 },
  });

  return cost ? Number(formatUnits(cost as bigint, 18)) : 0;
}
