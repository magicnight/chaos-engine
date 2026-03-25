'use client';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { WagmiProvider } from 'wagmi';
import { wagmiConfig } from '@/lib/web3/config';
import { useState } from 'react';
import { SessionProvider } from 'next-auth/react';
import { ChaosSSE } from './providers/chaos-sse';
import { I18nProvider } from '@/lib/i18n/context';

export function Providers({ children }: { children: React.ReactNode }) {
  const [queryClient] = useState(() => new QueryClient());

  return (
    <SessionProvider>
      <WagmiProvider config={wagmiConfig}>
        <QueryClientProvider client={queryClient}>
          <I18nProvider>
            {children}
            <ChaosSSE />
          </I18nProvider>
        </QueryClientProvider>
      </WagmiProvider>
    </SessionProvider>
  );
}
