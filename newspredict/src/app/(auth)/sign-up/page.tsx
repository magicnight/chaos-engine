'use client';

import { useState } from 'react';
import Link from 'next/link';

export default function SignUpPage() {
  const [name, setName] = useState('');
  const [email, setEmail] = useState('');

  function handleOAuth(provider: string) {
    window.location.href = `/api/auth/signin/${provider}`;
  }

  function handleEmailSignUp(e: React.FormEvent) {
    e.preventDefault();
    if (!email) return;
    window.location.href = `/api/auth/signin/email?email=${encodeURIComponent(email)}`;
  }

  return (
    <div className="min-h-screen flex items-center justify-center px-4">
      <div className="w-full max-w-sm">
        <div className="text-center mb-8">
          <h1 className="text-3xl font-bold mb-2">NewsPredict</h1>
          <p className="text-[var(--muted)]">Join the prediction market</p>
        </div>

        <div className="space-y-3 mb-6">
          <button
            onClick={() => handleOAuth('google')}
            className="w-full flex items-center justify-center gap-2 rounded-xl bg-[var(--card)] px-4 py-3 text-sm font-medium hover:bg-[var(--card-hover)] transition-colors"
          >
            Continue with Google
          </button>
          <button
            onClick={() => handleOAuth('github')}
            className="w-full flex items-center justify-center gap-2 rounded-xl bg-[var(--card)] px-4 py-3 text-sm font-medium hover:bg-[var(--card-hover)] transition-colors"
          >
            Continue with GitHub
          </button>
        </div>

        <div className="flex items-center gap-3 mb-6">
          <div className="flex-1 h-px bg-[var(--border)]" />
          <span className="text-xs text-[var(--muted)]">or</span>
          <div className="flex-1 h-px bg-[var(--border)]" />
        </div>

        <form onSubmit={handleEmailSignUp} className="space-y-3 mb-6">
          <input
            type="text"
            placeholder="Your name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            className="w-full rounded-xl bg-[var(--card)] border border-[var(--border)] px-4 py-3 text-sm placeholder:text-[var(--muted)] focus:outline-none focus:border-[var(--accent)]"
          />
          <input
            type="email"
            placeholder="Email address"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            className="w-full rounded-xl bg-[var(--card)] border border-[var(--border)] px-4 py-3 text-sm placeholder:text-[var(--muted)] focus:outline-none focus:border-[var(--accent)]"
          />
          <button
            type="submit"
            className="w-full rounded-xl bg-[var(--accent)] px-4 py-3 text-sm font-medium text-black hover:opacity-90 transition-opacity"
          >
            Sign Up with Email
          </button>
        </form>

        <p className="text-center text-sm text-[var(--muted)]">
          Already have an account?{' '}
          <Link href="/sign-in" className="text-[var(--accent)] hover:underline">
            Sign in
          </Link>
        </p>
      </div>
    </div>
  );
}
