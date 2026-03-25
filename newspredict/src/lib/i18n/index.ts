import en from './en.json';
import zh from './zh.json';

export type Locale = 'en' | 'zh';

const messages: Record<Locale, Record<string, Record<string, string>>> = { en, zh };

export function getTranslations(locale: Locale) {
  const dict = messages[locale] || messages.en;

  return function t(key: string, vars?: Record<string, string | number>): string {
    const [section, ...rest] = key.split('.');
    const field = rest.join('.');
    let val = dict[section]?.[field] ?? messages.en[section]?.[field] ?? key;
    if (vars) {
      for (const [k, v] of Object.entries(vars)) {
        val = val.replace(`{${k}}`, String(v));
      }
    }
    return val;
  };
}

export function detectLocale(): Locale {
  if (typeof document !== 'undefined') {
    const cookie = document.cookie
      .split('; ')
      .find((c) => c.startsWith('locale='));
    if (cookie) {
      const val = cookie.split('=')[1];
      if (val === 'zh' || val === 'en') return val;
    }
  }
  if (typeof navigator !== 'undefined') {
    const lang = navigator.language || '';
    if (lang.startsWith('zh')) return 'zh';
  }
  return 'en';
}

export function setLocaleCookie(locale: Locale) {
  document.cookie = `locale=${locale};path=/;max-age=${365 * 24 * 3600};samesite=lax`;
}
