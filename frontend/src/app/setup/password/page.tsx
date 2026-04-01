'use client';

import { useState } from 'react';
import { Nav } from '@/components/nav';
import { Card } from '@/components/card';
import { PasswordInput } from '@/components/password-input';
import { useLang } from '@/lib/use-lang';
import { t } from '@/lib/i18n';
import * as api from '@/lib/api';

export default function SetupPasswordPage() {
  const { lang } = useLang();
  const tr = t(lang).setup_password;
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setLoading(true);
    try {
      const res = await api.setupPassword(password);
      if (res.ok) {
        window.location.href = '/settings';
      } else {
        setError(res.error || 'Setup failed');
      }
    } catch {
      setError('Setup failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <>
      <Nav authenticated={false} />
      <main className="max-w-7xl mx-auto px-4 sm:px-6 py-16 flex items-center justify-center min-h-[calc(100vh-3.5rem)]">
        <Card className="max-w-md w-full" hover={false}>
          <h1 className="text-2xl font-bold text-slate-900 mb-2">{tr.title}</h1>
          <p className="text-sm text-slate-500 mb-6">{tr.description}</p>
          <form onSubmit={handleSubmit} className="space-y-4">
            <PasswordInput
              id="password"
              label={tr.password}
              value={password}
              onChange={setPassword}
              autoComplete="new-password"
            />
            {error && <p className="text-sm text-red-600">{error}</p>}
            <button type="submit" className="btn-primary w-full" disabled={loading}>
              {tr.submit}
            </button>
          </form>
        </Card>
      </main>
    </>
  );
}
