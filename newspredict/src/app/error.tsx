'use client';

export default function GlobalError({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  return (
    <div className="flex flex-col items-center justify-center min-h-[60vh] px-4 text-center">
      <div className="w-14 h-14 rounded-full bg-[var(--danger-dim)] flex items-center justify-center mb-4">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="var(--danger)" strokeWidth="2" strokeLinecap="round">
          <circle cx="12" cy="12" r="10" />
          <line x1="12" y1="8" x2="12" y2="12" />
          <line x1="12" y1="16" x2="12.01" y2="16" />
        </svg>
      </div>
      <h2 className="text-lg font-bold mb-2">Something went wrong</h2>
      <p className="text-sm text-[var(--muted)] mb-6 max-w-xs">
        An unexpected error occurred. Please try again.
      </p>
      {error.digest && (
        <p className="text-[10px] text-[var(--muted)] mb-4 font-mono">Error ID: {error.digest}</p>
      )}
      <button
        onClick={reset}
        className="px-6 py-2.5 rounded-full bg-[var(--accent)] text-black text-sm font-semibold"
      >
        Try again
      </button>
    </div>
  );
}
