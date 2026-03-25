'use client';
import { useAccount, useConnect, useDisconnect } from 'wagmi';
import { injected } from 'wagmi/connectors';

export function ConnectWalletButton() {
  const { address, isConnected } = useAccount();
  const { connect } = useConnect();
  const { disconnect } = useDisconnect();

  if (isConnected && address) {
    return (
      <div className="flex items-center gap-2">
        <span className="text-xs text-[var(--accent)] font-mono">
          {address.slice(0, 6)}...{address.slice(-4)}
        </span>
        <button
          onClick={() => disconnect()}
          className="text-xs px-2 py-1 rounded bg-[var(--card)] text-[var(--muted)] hover:text-[var(--danger)]"
        >
          Disconnect
        </button>
      </div>
    );
  }

  return (
    <button
      onClick={() => connect({ connector: injected() })}
      className="px-4 py-2 rounded-lg bg-[var(--accent)] text-black font-medium text-sm hover:opacity-90 transition-opacity"
    >
      Connect Wallet
    </button>
  );
}
