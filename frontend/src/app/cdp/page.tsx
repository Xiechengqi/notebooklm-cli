'use client';

import { useEffect, useRef, useState } from 'react';
import { Plus, RefreshCw, Trash2 } from 'lucide-react';
import { Nav } from '@/components/nav';
import { Card } from '@/components/card';
import { StatusDot } from '@/components/status-dot';
import { Spinner } from '@/components/spinner';
import { useLang } from '@/lib/use-lang';
import { t } from '@/lib/i18n';
import * as api from '@/lib/api';
import type { AccountEntry } from '@/lib/types';

function relativeTime(ts: number, never: string): string {
  if (ts === 0) return never;
  const diff = Math.floor((Date.now() / 1000) - ts);
  if (diff < 60) return `${diff}s ago`;
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
  return `${Math.floor(diff / 86400)}d ago`;
}

export default function CdpPage() {
  const { lang } = useLang();
  const tr = t(lang).cdp;

  const [ports, setPorts] = useState<string[]>([]);
  const [accounts, setAccounts] = useState<AccountEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [newPort, setNewPort] = useState('');
  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const loadData = async () => {
    const [portsRes, accountsRes] = await Promise.all([
      api.getCdpPorts(),
      api.getAccounts(),
    ]);
    setPorts(portsRes.data.ports);
    setAccounts(accountsRes.data);
  };

  useEffect(() => {
    (async () => {
      try {
        await loadData();
      } catch {
        // 401 handled by api wrapper
      } finally {
        setLoading(false);
      }
    })();

    pollRef.current = setInterval(async () => {
      try {
        const accountsRes = await api.getAccounts();
        setAccounts(accountsRes.data);
      } catch {}
    }, 30_000);

    return () => {
      if (pollRef.current) clearInterval(pollRef.current);
    };
  }, []);

  const handleAddPort = async () => {
    const port = newPort.trim();
    if (!port || ports.includes(port)) return;
    try {
      const res = await api.updateCdpPorts([...ports, port]);
      setPorts(res.data.ports);
      setNewPort('');
    } catch (e) {
      console.error('add port failed:', e);
    }
  };

  const handleRemovePort = async (port: string) => {
    try {
      const res = await api.updateCdpPorts(ports.filter((p) => p !== port));
      setPorts(res.data.ports);
    } catch (e) {
      console.error('remove port failed:', e);
    }
  };

  const handleRefresh = async () => {
    setRefreshing(true);
    try {
      await api.refreshCdpPorts();
      const before = new Map(accounts.map((a) => [a.cdp_port, a.last_checked]));
      for (let i = 0; i < 15; i++) {
        await new Promise((r) => setTimeout(r, 2000));
        const res = await api.getAccounts();
        setAccounts(res.data);
        const changed = res.data.some((a) => (before.get(a.cdp_port) ?? 0) !== a.last_checked);
        if (changed) break;
      }
    } catch (e) {
      console.error('refresh failed:', e);
    } finally {
      setRefreshing(false);
    }
  };

  const accountByPort = (port: string): AccountEntry | undefined =>
    accounts.find((a) => a.cdp_port === port);

  if (loading) {
    return (
      <>
        <Nav authenticated />
        <main className="max-w-7xl mx-auto px-4 sm:px-6 py-16 flex justify-center">
          <Spinner />
        </main>
      </>
    );
  }

  return (
    <>
      <Nav authenticated />
      <main className="max-w-7xl mx-auto px-4 sm:px-6 py-16 space-y-8">
        {/* Header */}
        <div>
          <h1 className="text-3xl font-extrabold tracking-tight gradient-text mb-1">{tr.title}</h1>
          <p className="text-slate-500 text-sm max-w-2xl">{tr.description}</p>
        </div>

        {/* Managed Ports */}
        <Card hover={false}>
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-bold text-slate-900">{tr.ports_heading}</h2>
            <button
              onClick={handleRefresh}
              disabled={refreshing}
              className="btn-secondary flex items-center gap-1.5 text-sm py-1.5 px-3"
            >
              <RefreshCw className={`h-3.5 w-3.5 ${refreshing ? 'animate-spin' : ''}`} />
              {refreshing ? tr.refreshing : tr.refresh}
            </button>
          </div>

          {/* Add port */}
          <div className="flex gap-2 mb-6">
            <input
              type="text"
              value={newPort}
              onChange={(e) => setNewPort(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleAddPort()}
              placeholder={tr.add_placeholder}
              className="flex-1 max-w-xs"
            />
            <button
              onClick={handleAddPort}
              disabled={!newPort.trim()}
              className="btn-primary flex items-center gap-1.5 text-sm py-1.5 px-4"
            >
              <Plus className="h-3.5 w-3.5" />
              {tr.add_button}
            </button>
          </div>

          {ports.length === 0 ? (
            <p className="text-slate-400 text-sm">{tr.no_ports}</p>
          ) : (
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-slate-100">
                    <th className="text-left pb-3 font-semibold text-slate-600 pr-6">{tr.col_port}</th>
                    <th className="text-left pb-3 font-semibold text-slate-600 pr-6">{tr.col_account}</th>
                    <th className="text-left pb-3 font-semibold text-slate-600 pr-6">{tr.col_name}</th>
                    <th className="text-left pb-3 font-semibold text-slate-600 pr-6">{tr.col_status}</th>
                    <th className="text-left pb-3 font-semibold text-slate-600 pr-6">{tr.col_last_checked}</th>
                    <th className="pb-3" />
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-50">
                  {ports.map((port) => {
                    const acct = accountByPort(port);
                    return (
                      <tr key={port} className="group">
                        <td className="py-3 pr-6 font-mono text-brand-700 font-semibold">{port}</td>
                        <td className="py-3 pr-6">
                          {acct?.email ? (
                            <span className="font-medium text-slate-700">{acct.email}</span>
                          ) : (
                            <span className="text-slate-400">—</span>
                          )}
                        </td>
                        <td className="py-3 pr-6 text-slate-600">
                          {acct?.display_name || <span className="text-slate-400">—</span>}
                        </td>
                        <td className="py-3 pr-6">
                          {!acct ? (
                            <span className="inline-flex items-center gap-1 text-slate-400 text-xs">
                              <StatusDot ok={false} />
                              {tr.pending}
                            </span>
                          ) : acct.online ? (
                            <span className="inline-flex items-center gap-1 text-emerald-600 text-xs font-medium">
                              <StatusDot ok={true} />
                              {tr.online}
                            </span>
                          ) : (
                            <span className="inline-flex items-center gap-1 text-slate-500 text-xs">
                              <StatusDot ok={false} />
                              {tr.offline}
                            </span>
                          )}
                        </td>
                        <td className="py-3 pr-6 text-slate-400 text-xs">
                          {acct ? relativeTime(acct.last_checked, tr.never) : tr.never}
                        </td>
                        <td className="py-3 text-right">
                          <button
                            onClick={() => handleRemovePort(port)}
                            className="opacity-0 group-hover:opacity-100 p-1.5 rounded-lg text-slate-400 hover:bg-red-50 hover:text-red-500 transition-all"
                            title={tr.remove}
                          >
                            <Trash2 className="h-3.5 w-3.5" />
                          </button>
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          )}
        </Card>

        {/* Discovered Accounts (ports not yet in managed list) */}
        {accounts.filter((a) => !ports.includes(a.cdp_port)).length > 0 && (
          <Card hover={false}>
            <h2 className="text-lg font-bold text-slate-900 mb-4">{tr.accounts_heading}</h2>
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-slate-100">
                    <th className="text-left pb-3 font-semibold text-slate-600 pr-6">{tr.col_port}</th>
                    <th className="text-left pb-3 font-semibold text-slate-600 pr-6">{tr.col_account}</th>
                    <th className="text-left pb-3 font-semibold text-slate-600 pr-6">{tr.col_name}</th>
                    <th className="text-left pb-3 font-semibold text-slate-600 pr-6">{tr.col_status}</th>
                    <th className="text-left pb-3 font-semibold text-slate-600">{tr.col_last_checked}</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-50">
                  {accounts.filter((a) => !ports.includes(a.cdp_port)).map((acct) => (
                    <tr key={acct.cdp_port}>
                      <td className="py-3 pr-6 font-mono text-brand-700 font-semibold">{acct.cdp_port}</td>
                      <td className="py-3 pr-6">
                        <span className="font-medium text-slate-700">{acct.email}</span>
                      </td>
                      <td className="py-3 pr-6 text-slate-600">{acct.display_name}</td>
                      <td className="py-3 pr-6">
                        <span className={`inline-flex items-center gap-1 text-xs font-medium ${acct.online ? 'text-emerald-600' : 'text-slate-500'}`}>
                          <StatusDot ok={acct.online} />
                          {acct.online ? tr.online : tr.offline}
                        </span>
                      </td>
                      <td className="py-3 text-slate-400 text-xs">{relativeTime(acct.last_checked, tr.never)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </Card>
        )}
      </main>
    </>
  );
}
