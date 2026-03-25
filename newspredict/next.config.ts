import type { NextConfig } from 'next';

const nextConfig: NextConfig = {
  output: 'standalone',
  reactStrictMode: true,
  webpack: (config) => {
    // Stub out modules we don't use:
    // - Solana/Coinbase: we only use BSC (EVM)
    // - Porto: experimental connector we don't need
    // - pg-native: optional native binding for pg driver
    const stubModules = [
      '@solana/kit',
      '@solana/web3.js',
      '@coinbase/cdp-sdk',
      '@coinbase/wallet-sdk',
      '@base-org/account',
      '@metamask/connect-evm',
      'porto',
      'porto/internal',
      'axios',
      'pg-native',
      '@walletconnect/ethereum-provider',
    ];

    for (const mod of stubModules) {
      config.resolve.alias[mod] = false;
    }

    return config;
  },
};

export default nextConfig;
