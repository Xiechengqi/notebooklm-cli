'use client';

import { LangProvider } from '@/lib/use-lang';

export function Providers({ children }: { children: React.ReactNode }) {
  return <LangProvider>{children}</LangProvider>;
}
