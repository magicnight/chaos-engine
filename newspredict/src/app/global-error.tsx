'use client';

import * as Sentry from '@sentry/nextjs';
import { useEffect } from 'react';

export default function GlobalError({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  useEffect(() => {
    Sentry.captureException(error);
  }, [error]);

  return (
    <html>
      <body style={{ background: '#0b1220', color: '#e2e8f0', fontFamily: 'system-ui' }}>
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', minHeight: '100vh', padding: '1rem', textAlign: 'center' }}>
          <h2 style={{ fontSize: '1.25rem', fontWeight: 700, marginBottom: '0.5rem' }}>Something went wrong</h2>
          <p style={{ fontSize: '0.875rem', color: '#64748b', marginBottom: '1.5rem' }}>
            An unexpected error occurred.
          </p>
          {error.digest && (
            <p style={{ fontSize: '0.625rem', color: '#64748b', marginBottom: '1rem', fontFamily: 'monospace' }}>
              Error ID: {error.digest}
            </p>
          )}
          <button
            onClick={reset}
            style={{ padding: '0.5rem 1.5rem', borderRadius: '9999px', background: '#00d4ff', color: 'black', fontWeight: 600, fontSize: '0.875rem', border: 'none', cursor: 'pointer' }}
          >
            Try again
          </button>
        </div>
      </body>
    </html>
  );
}
