'use client';

import Link from 'next/link';
import { signIn } from 'next-auth/react';
import { WalletSignIn } from '@/components/web3/wallet-sign-in';

export default function SignInPage() {

  return (
    <div className="min-h-screen flex items-center justify-center px-4">
      <div className="w-full max-w-sm">
        <div className="text-center mb-8">
          <h1 className="text-3xl font-bold mb-2">NewsPredict</h1>
          <p className="text-[var(--muted)]">Predict the future</p>
        </div>

        <div className="space-y-3 mb-6">
          <button
            onClick={() => signIn('guest', { callbackUrl: '/' })}
            className="w-full flex items-center justify-center gap-2 rounded-xl bg-[var(--accent)] text-black px-4 py-3 text-sm font-bold hover:opacity-90 transition-opacity"
          >
            Start Trading (Guest — $1,000 free)
          </button>
          <button
            onClick={() => signIn('google')}
            className="w-full flex items-center justify-center gap-2 rounded-xl bg-[var(--card)] px-4 py-3 text-sm font-medium hover:bg-[var(--card-hover)] transition-colors"
          >
            Continue with Google
          </button>
          <button
            onClick={() => signIn('github')}
            className="w-full flex items-center justify-center gap-2 rounded-xl bg-[var(--card)] px-4 py-3 text-sm font-medium hover:bg-[var(--card-hover)] transition-colors"
          >
            Continue with GitHub
          </button>
        </div>

        <div className="text-center text-sm text-[var(--muted)]">— or —</div>

        <div className="mb-6">
          <WalletSignIn />
        </div>

        <p className="text-center text-sm text-[var(--muted)]">
          New here?{' '}
          <Link href="/sign-up" className="text-[var(--accent)] hover:underline">
            Sign up
          </Link>
        </p>
      </div>
    </div>
  );
}
