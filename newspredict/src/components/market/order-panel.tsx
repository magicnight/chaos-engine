'use client';

import { useState } from 'react';
import { useAccount } from 'wagmi';
import { formatUnits } from 'viem';
import { useChaosBalance, useApproveToken, useBuyShares, useSellShares } from '@/lib/web3/hooks';
import { CONTRACTS } from '@/lib/web3/contracts';
import { useLocale } from '@/lib/i18n/context';

type TradeMode = 'virtual' | 'onchain';

interface OrderPanelProps {
  marketId: string;
  yesPrice: number;
  noPrice: number;
  onchainMarketId?: number;
  onTrade?: (result: { success: boolean; shares?: number; cost?: number; error?: string }) => void;
}

function TxStatus({ hash, isConfirming, isSuccess, chainId }: {
  hash?: `0x${string}`;
  isConfirming: boolean;
  isSuccess: boolean;
  chainId?: number;
}) {
  const { t } = useLocale();
  if (!hash) return null;
  const explorer = chainId === 56
    ? 'https://bscscan.com/tx/'
    : 'https://testnet.bscscan.com/tx/';

  return (
    <div className="text-xs mt-2 p-2 rounded bg-[var(--background)]">
      {isConfirming && <p className="text-[var(--accent)]">{t('order.confirmingTx')}</p>}
      {isSuccess && <p className="text-[var(--success)]">{t('order.txConfirmed')}</p>}
      <a
        href={`${explorer}${hash}`}
        target="_blank"
        rel="noopener noreferrer"
        className="text-[var(--accent)] hover:underline break-all"
      >
        {hash.slice(0, 10)}...{hash.slice(-8)}
      </a>
    </div>
  );
}

export function OrderPanel({ marketId, yesPrice, noPrice, onchainMarketId, onTrade }: OrderPanelProps) {
  const [side, setSide] = useState<'YES' | 'NO'>('YES');
  const [amount, setAmount] = useState(10);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [mode, setMode] = useState<TradeMode>('virtual');
  const { t } = useLocale();

  const { isConnected, chain } = useAccount();
  const { balance: cruxBalance, refetch: refetchBalance } = useChaosBalance();
  const { approve, isPending: approving, isConfirming: approveConfirming, isSuccess: approveSuccess, hash: approveHash } = useApproveToken();
  const { buy, isPending: buying, isConfirming: buyConfirming, isSuccess: buySuccess, hash: buyHash } = useBuyShares();

  const currentPrice = side === 'YES' ? yesPrice : noPrice;
  const estimatedShares = amount / currentPrice;
  const potentialProfit = estimatedShares * (1 - currentPrice);

  const formattedCruxBalance = cruxBalance
    ? parseFloat(formatUnits(cruxBalance as bigint, 18)).toFixed(2)
    : '0.00';

  const chainId = chain?.id;
  const contracts = chainId ? CONTRACTS[chainId] : null;
  const hasContracts = !!contracts?.token && !!contracts?.market;

  async function handleVirtualTrade() {
    setLoading(true);
    setError('');
    try {
      const res = await fetch('/api/trades', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ marketId, side, amount }),
      });
      const data = await res.json();
      if (!res.ok) {
        setError(data.error || 'Trade failed');
      } else {
        onTrade?.(data);
      }
    } catch {
      setError(t('common.networkError'));
    } finally {
      setLoading(false);
    }
  }

  function handleOnchainApprove() {
    if (!chainId || !contracts?.market) return;
    setError('');
    approve(contracts.market, amount.toString(), chainId);
  }

  function handleOnchainBuy() {
    if (!chainId || onchainMarketId === undefined) return;
    setError('');
    const sideNum = side === 'YES' ? 0 : 1;
    buy(onchainMarketId, sideNum as 0 | 1, estimatedShares.toFixed(4), amount.toString(), chainId);
  }

  if (buySuccess) {
    refetchBalance();
    // Save txHash to backend
    if (buyHash) {
      fetch('/api/trades', {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ tradeId: marketId, txHash: buyHash }),
      }).catch(() => {});
    }
  }

  const isOnchain = mode === 'onchain';
  const onchainBusy = approving || approveConfirming || buying || buyConfirming;

  return (
    <div className="bg-[var(--card)] rounded-xl p-4">
      {isConnected && (
        <div className="flex rounded-lg bg-[var(--background)] p-0.5 mb-4">
          <button
            onClick={() => setMode('virtual')}
            className={`flex-1 py-1.5 rounded-md text-xs font-medium transition-colors ${
              mode === 'virtual'
                ? 'bg-[var(--card)] text-[var(--foreground)] shadow-sm'
                : 'text-[var(--muted)]'
            }`}
          >
            {t('order.virtualCredits')}
          </button>
          <button
            onClick={() => setMode('onchain')}
            className={`flex-1 py-1.5 rounded-md text-xs font-medium transition-colors ${
              mode === 'onchain'
                ? 'bg-[var(--card)] text-[var(--foreground)] shadow-sm'
                : 'text-[var(--muted)]'
            }`}
          >
            {t('order.onChain')}
          </button>
        </div>
      )}

      {isOnchain && (
        <div className="flex items-center justify-between mb-3 px-1">
          <span className="text-xs text-[var(--muted)]">{t('order.chaosBalance')}</span>
          <span className="text-xs font-mono text-[var(--foreground)]">{formattedCruxBalance} CHAOS</span>
        </div>
      )}

      <div className="flex gap-2 mb-4">
        <button
          onClick={() => setSide('YES')}
          aria-label={`Buy Yes at ${yesPrice.toFixed(2)}`}
          className={`flex-1 py-3 rounded-lg text-sm font-semibold transition-colors ${
            side === 'YES'
              ? 'bg-[var(--success)] text-black'
              : 'bg-[var(--success)]/15 text-[var(--success)]'
          }`}
        >
          {t('order.buyYes')} ${yesPrice.toFixed(2)}
        </button>
        <button
          onClick={() => setSide('NO')}
          aria-label={`Buy No at ${noPrice.toFixed(2)}`}
          className={`flex-1 py-3 rounded-lg text-sm font-semibold transition-colors ${
            side === 'NO'
              ? 'bg-[var(--danger)] text-white'
              : 'bg-[var(--danger)]/15 text-[var(--danger)]'
          }`}
        >
          {t('order.buyNo')} ${noPrice.toFixed(2)}
        </button>
      </div>

      <div className="mb-4">
        <label className="text-xs text-[var(--muted)] mb-1 block">
          {t('order.amount')} {isOnchain ? '(CHAOS)' : '($)'}
        </label>
        <input
          type="number"
          min={1}
          max={10000}
          value={amount}
          onChange={(e) => setAmount(Math.max(1, Number(e.target.value)))}
          className="w-full rounded-lg bg-[var(--background)] border border-[var(--border)] px-4 py-2.5 text-sm text-[var(--foreground)] outline-none focus:border-[var(--accent)] transition-colors"
        />
        <div className="flex gap-2 mt-2">
          {[5, 10, 25, 50, 100].map((v: any) => (
            <button
              key={v}
              onClick={() => setAmount(v)}
              className={`flex-1 py-1 rounded text-xs font-medium transition-colors ${
                amount === v
                  ? 'bg-[var(--accent)] text-black'
                  : 'bg-[var(--background)] text-[var(--muted)] hover:text-[var(--foreground)]'
              }`}
            >
              {isOnchain ? v : `$${v}`}
            </button>
          ))}
        </div>
      </div>

      <div className="space-y-1 mb-4 text-xs text-[var(--muted)]">
        <div className="flex justify-between">
          <span>{t('order.estShares')}</span>
          <span className="text-[var(--foreground)]">{estimatedShares.toFixed(1)}</span>
        </div>
        <div className="flex justify-between">
          <span>{t('order.avgPrice')}</span>
          <span className="text-[var(--foreground)]">${currentPrice.toFixed(2)}</span>
        </div>
        <div className="flex justify-between">
          <span>{t('order.potentialProfit')}</span>
          <span className="text-[var(--success)]">+${potentialProfit.toFixed(2)}</span>
        </div>
      </div>

      {error && (
        <p className="text-xs text-[var(--danger)] mb-2">{error}</p>
      )}

      {isOnchain ? (
        <div className="space-y-2">
          {!hasContracts && (
            <p className="text-xs text-[var(--muted)] text-center py-2">
              {t('order.notDeployed')}
            </p>
          )}
          {hasContracts && !approveSuccess && (
            <button
              onClick={handleOnchainApprove}
              disabled={onchainBusy || amount <= 0}
              className="w-full py-3 rounded-lg bg-[var(--border)] text-[var(--foreground)] font-semibold text-sm hover:opacity-90 transition-opacity disabled:opacity-50"
            >
              {approving ? t('order.approving') : approveConfirming ? t('order.confirming') : t('order.approve', { n: amount })}
            </button>
          )}
          {hasContracts && approveSuccess && (
            <button
              onClick={handleOnchainBuy}
              disabled={onchainBusy || amount <= 0 || onchainMarketId === undefined}
              className="w-full py-3 rounded-lg bg-[var(--accent)] text-black font-semibold text-sm hover:opacity-90 transition-opacity disabled:opacity-50"
            >
              {buying ? t('order.buying') : buyConfirming ? t('order.confirming') : t('order.buyShares', { side })}
            </button>
          )}
          <TxStatus hash={approveHash} isConfirming={approveConfirming} isSuccess={approveSuccess} chainId={chainId} />
          <TxStatus hash={buyHash} isConfirming={buyConfirming} isSuccess={buySuccess} chainId={chainId} />
        </div>
      ) : (
        <button
          onClick={handleVirtualTrade}
          disabled={loading || amount <= 0}
          aria-label={loading ? 'Placing trade' : `Place ${side} trade for ${amount} dollars`}
          className="w-full py-3 rounded-lg bg-[var(--accent)] text-black font-semibold text-sm hover:opacity-90 transition-opacity disabled:opacity-50"
        >
          {loading ? t('order.placing') : t('order.placeTrade', { side })}
        </button>
      )}
    </div>
  );
}
