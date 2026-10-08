import React, { createContext, useState, useCallback } from 'react';
import english from './messages/en.json';
import german from './messages/de.json';
import type { ReactNode } from 'react';

// ─── Types ────────────────────────────────────────────────────────────────────

type MessageCatalog = Record<string, string>;

export interface LocaleContextValue {
  /** Current locale code (e.g. "en", "de") */
  locale: string;
  /**
   * Look up a translated string by dotted key.
   * Supports simple {placeholder} interpolation with the second argument.
   *
   * @example
   *   t("nav.dashboard")                // → "Dashboard"
   *   t("stories.shown", { count: 12 }) // → "12 shown"
   */
  t: (key: string, params?: Record<string, string | number>) => string;
  /** Switch the active locale and reload messages */
  setLocale: (locale: string) => void;
}

// ─── Locale registry — add new locales here ───────────────────────────────────

const LOCALE_MESSAGES: Record<string, MessageCatalog> = { en: english, de: german };

// ─── Context ───────────────────────────────────────────────────────────────────

export const LocaleContext = createContext<LocaleContextValue>({
  locale: 'en',
  t: (key: string) => key,
  setLocale: () => {},
});

// ─── Provider ──────────────────────────────────────────────────────────────────

interface LocaleProviderProps {
  children: ReactNode;
  /** Initial locale (defaults to "en") */
  defaultLocale?: string;
}

/**
 * Provides locale state and a `t()` translation function to the component tree.
 *
 * Messages are imported as browser-compatible modules.
 * Falls back to the message key itself when a translation is missing.
 */
export function LocaleProvider({ children, defaultLocale = 'en' }: LocaleProviderProps) {
  const [locale, setLocaleState] = useState(defaultLocale);
  const [messages, setMessages] = useState<MessageCatalog>(() => LOCALE_MESSAGES[defaultLocale] ?? english);

  const switchLocale = useCallback((next: string) => {
    setMessages(LOCALE_MESSAGES[next] ?? {});
    setLocaleState(next);
  }, []);

  const t = useCallback(
    (key: string, params?: Record<string, string | number>): string => {
      let value = messages[key];
      if (value === undefined) {
        // Fall back to the key itself so the UI doesn't break
        if (import.meta.env.DEV) {
          console.warn(`[i18n] Missing translation for "${key}" in locale "${locale}"`);
        }
        value = key;
      }
      if (params) {
        for (const [k, v] of Object.entries(params)) {
          value = value.replace(`{${k}}`, String(v));
        }
      }
      return value;
    },
    [messages, locale],
  );

  return (
    <LocaleContext.Provider value={{ locale, t, setLocale: switchLocale }}>
      {children}
    </LocaleContext.Provider>
  );
}
