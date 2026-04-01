'use client';

import { useCallback, useEffect, useState } from 'react';

type Theme = 'auto' | 'dark' | 'light';

function applyTheme(theme: Theme) {
  const isDark =
    theme === 'dark' ||
    (theme === 'auto' && window.matchMedia('(prefers-color-scheme: dark)').matches);
  document.documentElement.classList.toggle('dark', isDark);
}

export function useTheme() {
  const [theme, setThemeState] = useState<Theme>('auto');

  useEffect(() => {
    const stored = localStorage.getItem('theme') as Theme | null;
    const initial = stored && ['auto', 'dark', 'light'].includes(stored) ? stored : 'auto';
    setThemeState(initial);
    applyTheme(initial);

    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    const handler = () => {
      if ((localStorage.getItem('theme') || 'auto') === 'auto') applyTheme('auto');
    };
    mq.addEventListener('change', handler);
    return () => mq.removeEventListener('change', handler);
  }, []);

  const cycle = useCallback(() => {
    setThemeState((prev) => {
      const next: Theme = prev === 'auto' ? 'dark' : prev === 'dark' ? 'light' : 'auto';
      localStorage.setItem('theme', next);
      applyTheme(next);
      return next;
    });
  }, []);

  return { theme, cycle };
}
