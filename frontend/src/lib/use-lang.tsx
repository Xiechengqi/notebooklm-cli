'use client';

import { createContext, useCallback, useContext, useEffect, useState } from 'react';
import type { Lang } from './i18n';

interface LangContextValue {
  lang: Lang;
  setLang: (next: Lang) => void;
  toggle: () => void;
}

const LangContext = createContext<LangContextValue | null>(null);

export function LangProvider({ children }: { children: React.ReactNode }) {
  const [lang, setLangState] = useState<Lang>('en');

  useEffect(() => {
    const stored = localStorage.getItem('lang') as Lang | null;
    if (stored === 'zh' || stored === 'en') setLangState(stored);
  }, []);

  const setLang = useCallback((next: Lang) => {
    setLangState(next);
    localStorage.setItem('lang', next);
    document.cookie = `lang=${next};path=/;max-age=31536000`;
  }, []);

  const toggle = useCallback(() => {
    setLangState((prev) => {
      const next = prev === 'en' ? 'zh' : 'en';
      localStorage.setItem('lang', next);
      document.cookie = `lang=${next};path=/;max-age=31536000`;
      return next;
    });
  }, []);

  return (
    <LangContext.Provider value={{ lang, setLang, toggle }}>
      {children}
    </LangContext.Provider>
  );
}

export function useLang() {
  const ctx = useContext(LangContext);
  if (!ctx) throw new Error('useLang must be used within LangProvider');
  return ctx;
}
