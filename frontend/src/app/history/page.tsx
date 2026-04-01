'use client';

import { useEffect, useState } from 'react';
import { Nav } from '@/components/nav';
import { Card } from '@/components/card';
import { ExecutionTable } from '@/components/execution-table';
import { Spinner } from '@/components/spinner';
import { useLang } from '@/lib/use-lang';
import { t } from '@/lib/i18n';
import * as api from '@/lib/api';
import type { ExecutionRecord } from '@/lib/types';

export default function HistoryPage() {
  const { lang } = useLang();
  const tr = t(lang).history;
  const [records, setRecords] = useState<ExecutionRecord[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    (async () => {
      try {
        const bs = await api.bootstrap();
        if (bs.password_required) {
          window.location.href = '/setup/password';
          return;
        }
        const histRes = await api.getHistory();
        setRecords(histRes.data);
      } catch {
        // 401 handled by api wrapper
      } finally {
        setLoading(false);
      }
    })();
  }, []);

  if (loading) {
    return (
      <>
        <Nav authenticated />
        <main className="max-w-5xl mx-auto px-4 sm:px-6 py-16 flex justify-center">
          <Spinner />
        </main>
      </>
    );
  }

  const allRecords = [...records].reverse();

  return (
    <>
      <Nav authenticated />
      <main className="max-w-5xl mx-auto px-4 sm:px-6 py-16 space-y-8">
        <div>
          <h1 className="text-2xl font-bold text-slate-900 mb-2">{tr.title}</h1>
          <p className="text-sm text-slate-500">{tr.description}</p>
        </div>

        <Card hover={false}>
          <ExecutionTable records={allRecords} showAll />
        </Card>
      </main>
    </>
  );
}
