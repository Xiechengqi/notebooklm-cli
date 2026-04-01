'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { Globe, LogOut, History, Eye } from 'lucide-react';
import { clsx } from 'clsx';
import { useLang } from '@/lib/use-lang';
import { t } from '@/lib/i18n';
import * as api from '@/lib/api';

export function Nav({ authenticated }: { authenticated: boolean }) {
  const pathname = usePathname();
  const { lang, toggle: toggleLang } = useLang();
  const tr = t(lang);

  const navLink = (href: string, label: string) => (
    <Link
      href={href}
      className={clsx(
        'px-3 py-1.5 rounded-lg text-sm font-medium transition-all duration-200',
        pathname === href
          ? 'bg-brand-50 text-brand-600'
          : 'text-slate-600 hover:bg-brand-50 hover:text-brand-600',
      )}
    >
      {label}
    </Link>
  );

  const handleLogout = async () => {
    await api.logout();
    window.location.href = '/login';
  };

  return (
    <nav className="sticky top-0 z-50 bg-white/85 backdrop-blur-xl border-b border-slate-200">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 flex items-center justify-between h-14">
        <div className="flex items-center gap-1">
          <Link href="/" className="text-lg font-extrabold gradient-text mr-4">
            notebooklm-cli
          </Link>
          {authenticated ? (
            <>
              {navLink('/commands', tr.nav.commands)}
              {navLink('/mcp', tr.nav.mcp)}
              {navLink('/cdp', tr.nav.cdp)}
              {navLink('/skills', tr.nav.skills)}
              {navLink('/docs', tr.nav.docs)}
              {navLink('/settings', tr.nav.settings)}
              <Link
                href="/history"
                className={clsx(
                  'flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-sm font-medium transition-all duration-200',
                  pathname === '/history'
                    ? 'bg-brand-50 text-brand-600'
                    : 'text-slate-600 hover:bg-brand-50 hover:text-brand-600',
                )}
              >
                <History className="h-3.5 w-3.5" />
                {tr.nav.history}
              </Link>
              <Link
                href="/preview"
                className={clsx(
                  'flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-sm font-medium transition-all duration-200',
                  pathname === '/preview'
                    ? 'bg-brand-50 text-brand-600'
                    : 'text-slate-600 hover:bg-brand-50 hover:text-brand-600',
                )}
              >
                <Eye className="h-3.5 w-3.5" />
                {tr.nav.preview}
              </Link>
            </>
          ) : (
            <>
              {navLink('/login', tr.nav.login)}
              {navLink('/setup/password', tr.nav.setup_password)}
            </>
          )}
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={toggleLang}
            className="p-2 rounded-lg text-slate-500 hover:bg-slate-100 transition-colors text-xs font-semibold"
          >
            <Globe className="h-4 w-4" />
          </button>
          {authenticated && (
            <button
              onClick={handleLogout}
              className="p-2 rounded-lg text-slate-500 hover:bg-red-50 hover:text-red-600 transition-colors"
              title={tr.nav.logout}
            >
              <LogOut className="h-4 w-4" />
            </button>
          )}
        </div>
      </div>
    </nav>
  );
}
