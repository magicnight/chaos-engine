import Link from 'next/link';

export default function NotFound() {
  return (
    <div className="flex flex-col items-center justify-center min-h-[60vh] px-4 text-center">
      <p className="text-6xl font-bold text-gradient mb-4">404</p>
      <h2 className="text-lg font-bold mb-2">Page not found</h2>
      <p className="text-sm text-[var(--muted)] mb-6">
        The page you&apos;re looking for doesn&apos;t exist.
      </p>
      <Link
        href="/"
        className="px-6 py-2.5 rounded-full bg-[var(--accent)] text-black text-sm font-semibold"
      >
        Back to Home
      </Link>
    </div>
  );
}
