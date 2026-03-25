'use client';

import { useLocale } from '@/lib/i18n/context';

export default function OfflinePage() {
  const { t } = useLocale();
  return (
    <div className="flex flex-col items-center justify-center min-h-[60vh] px-4 text-center">
      <div className="text-4xl mb-4">📡</div>
      <h1 className="text-xl font-bold mb-2">{t('offline.title')}</h1>
      <p className="text-sm text-[var(--muted)] mb-6 max-w-xs">
        {t('offline.message')}
      </p>
      <button
        onClick={() => window.location.reload()}
        className="px-6 py-2.5 rounded-full bg-[var(--accent)] text-black text-sm font-semibold"
      >
        Try Again
      </button>
    </div>
  );
}
