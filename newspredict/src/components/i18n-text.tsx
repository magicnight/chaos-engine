'use client';

import { useLocale } from '@/lib/i18n/context';

export function T({ k, vars }: { k: string; vars?: Record<string, string | number> }) {
  const { t } = useLocale();
  return <>{t(k, vars)}</>;
}
