'use client';
import { WagmiProvider } from 'wagmi';
import { wagmiConfig } from '@/lib/web3/config';
import { SessionProvider } from 'next-auth/react';
import { ChaosSSE } from './providers/chaos-sse';
import { I18nProvider } from '@/lib/i18n/context';

export function Providers({ children }: { children: React.ReactNode }) {
  return (
    <SessionProvider>
      <WagmiProvider config={wagmiConfig}>
        <I18nProvider>
          {children}
          <ChaosSSE />
        </I18nProvider>
      </WagmiProvider>
    </SessionProvider>
  );
}
