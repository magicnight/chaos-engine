'use client';
import { useAccount, useConnect, useDisconnect, useSignMessage } from 'wagmi';
import { injected } from 'wagmi/connectors';
import { SiweMessage } from 'siwe';
import { signIn } from 'next-auth/react';
import { useState, useEffect } from 'react';

export function WalletSignIn() {
  const { address, isConnected } = useAccount();
  const [mounted, setMounted] = useState(false);
  useEffect(() => setMounted(true), []);
  const { connect } = useConnect();
  const { disconnect } = useDisconnect();
  const { signMessageAsync } = useSignMessage();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleSignIn() {
    if (!address) return;
    setLoading(true);
    setError(null);

    try {
      // 1. Fetch nonce
      const nonceRes = await fetch('/api/auth/siwe/nonce');
      const { nonce } = await nonceRes.json();

      // 2. Create SIWE message
      const message = new SiweMessage({
        domain: window.location.host,
        address,
        statement: 'Sign in to NewsPredict',
        uri: window.location.origin,
        version: '1',
        chainId: 56, // BSC mainnet
        nonce,
      });
      const messageStr = message.prepareMessage();

      // 3. Request signature
      const signature = await signMessageAsync({ message: messageStr });

      // 4. Sign in via NextAuth credentials
      const result = await signIn('siwe', {
        message: messageStr,
        signature,
        redirect: false,
      });

      if (result?.error) {
        setError('Wallet sign-in failed');
      } else {
        window.location.href = '/';
      }
    } catch {
      setError('Sign-in cancelled or failed');
    } finally {
      setLoading(false);
    }
  }

  // Prevent hydration mismatch: wagmi restores wallet state on client only
  if (!mounted) {
    return (
      <button
        disabled
        className="w-full flex items-center justify-center gap-2 rounded-xl bg-[var(--card)] px-4 py-3 text-sm font-medium opacity-50"
      >
        Connect Wallet
      </button>
    );
  }

  if (!isConnected) {
    return (
      <button
        onClick={() => connect({ connector: injected() })}
        className="w-full flex items-center justify-center gap-2 rounded-xl bg-[var(--card)] px-4 py-3 text-sm font-medium hover:bg-[var(--card-hover)] transition-colors"
      >
        Connect Wallet
      </button>
    );
  }

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between rounded-xl bg-[var(--card)] px-4 py-3">
        <span className="text-sm font-mono text-[var(--accent)]">
          {address!.slice(0, 6)}...{address!.slice(-4)}
        </span>
        <button
          onClick={() => disconnect()}
          className="text-xs text-[var(--muted)] hover:text-[var(--danger)]"
        >
          Disconnect
        </button>
      </div>
      <button
        onClick={handleSignIn}
        disabled={loading}
        className="w-full rounded-xl bg-[var(--accent)] px-4 py-3 text-sm font-medium text-black hover:opacity-90 transition-opacity disabled:opacity-50"
      >
        {loading ? 'Signing in...' : 'Sign in with Wallet'}
      </button>
      {error && <p className="text-sm text-center text-red-400">{error}</p>}
    </div>
  );
}
