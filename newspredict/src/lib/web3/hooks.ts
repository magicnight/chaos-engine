import { useAccount, useBalance, useReadContract, useWriteContract, useWaitForTransactionReceipt } from 'wagmi';
import { formatUnits, parseUnits } from 'viem';
import { CRUX_TOKEN_ABI, PREDICTION_MARKET_ABI, CONTRACTS } from './contracts';

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

export function useCruxBalance() {
  const { address, chain } = useAccount();
  const contracts = chain ? CONTRACTS[chain.id] : null;

  const { data: balance, refetch } = useReadContract({
    address: contracts?.token as `0x${string}`,
    abi: CRUX_TOKEN_ABI,
    functionName: 'balanceOf',
    args: address ? [address] : undefined,
    query: { enabled: !!address && !!contracts?.token },
  });

  return { balance, refetch };
}

export function useApproveToken() {
  const { writeContract, data: hash, isPending } = useWriteContract();
  const { isLoading: isConfirming, isSuccess } = useWaitForTransactionReceipt({ hash });

  const approve = (spender: string, amount: string, chainId: number) => {
    const contracts = CONTRACTS[chainId];
    if (!contracts?.token) return;

    writeContract({
      address: contracts.token as `0x${string}`,
      abi: CRUX_TOKEN_ABI,
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
      abi: PREDICTION_MARKET_ABI,
      functionName: 'buyShares',
      args: [BigInt(marketId), side, parseUnits(shares, 18), parseUnits(maxCost, 18)],
    });
  };

  return { buy, isPending, isConfirming, isSuccess, hash };
}

export function useClaimWinnings() {
  const { writeContract, data: hash, isPending } = useWriteContract();
  const { isLoading: isConfirming, isSuccess } = useWaitForTransactionReceipt({ hash });

  const claim = (marketId: number, chainId: number) => {
    const contracts = CONTRACTS[chainId];
    if (!contracts?.market) return;

    writeContract({
      address: contracts.market as `0x${string}`,
      abi: PREDICTION_MARKET_ABI,
      functionName: 'claimWinnings',
      args: [BigInt(marketId)],
    });
  };

  return { claim, isPending, isConfirming, isSuccess, hash };
}
