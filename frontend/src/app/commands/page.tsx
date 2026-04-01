'use client';

import { useEffect, useState, useMemo } from 'react';
import { Nav } from '@/components/nav';
import { Card } from '@/components/card';
import { Spinner } from '@/components/spinner';
import { useLang } from '@/lib/use-lang';
import { t } from '@/lib/i18n';
import * as api from '@/lib/api';
import type { AccountEntry, CommandSpec } from '@/lib/types';

function buildCliCommand(name: string, cdpPort: string, params: Record<string, unknown>): string {
  const { cdp_port: _, ...rest } = params as Record<string, unknown> & { cdp_port?: unknown };
  const json = JSON.stringify(rest);
  const portFlag = cdpPort ? ` --cdp-port ${cdpPort}` : '';
  if (json === '{}') {
    return `notebooklm-cli execute ${name}${portFlag}`;
  }
  return `notebooklm-cli execute ${name}${portFlag} --params '${json}'`;
}

export default function CommandsPage() {
  const { lang } = useLang();
  const tr = t(lang).commands;
  const [commands, setCommands] = useState<CommandSpec[]>([]);
  const [accounts, setAccounts] = useState<AccountEntry[]>([]);
  const [selected, setSelected] = useState('');
  const [cdpPort, setCdpPort] = useState('');
  const [params, setParams] = useState<Record<string, string>>({});
  const [running, setRunning] = useState(false);
  const [result, setResult] = useState('');
  const [cliCmd, setCliCmd] = useState('');
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    (async () => {
      try {
        const [cmdRes, accRes] = await Promise.all([api.getCommands(), api.getAccounts()]);
        setCommands(cmdRes.data);
        if (cmdRes.data.length > 0) {
          const cached = localStorage.getItem('notebooklm-cli:last-command');
          const initial = cached && cmdRes.data.some((c) => c.name === cached)
            ? cached
            : cmdRes.data[0].name;
          setSelected(initial);
        }
        setAccounts(accRes.data);
        if (accRes.data.length > 0) {
          const cached = localStorage.getItem('notebooklm-cli:last-cdp-port');
          const initial = cached && accRes.data.some((a) => a.cdp_port === cached)
            ? cached
            : accRes.data[0].cdp_port;
          setCdpPort(initial);
        }
      } catch { /* 401 */ }
      finally { setLoading(false); }
    })();
  }, []);

  const cmd = useMemo(() => commands.find((c) => c.name === selected), [commands, selected]);

  const handleSelect = (name: string) => {
    setSelected(name);
    localStorage.setItem('notebooklm-cli:last-command', name);
    setParams({});
    setResult('');
    setCliCmd('');
  };

  const handleParamChange = (name: string, value: string) => {
    setParams((prev) => ({ ...prev, [name]: value }));
  };

  const parseParams = (): Record<string, unknown> | null => {
    if (!cmd) return null;
    const parsed: Record<string, unknown> = { cdp_port: cdpPort };
    for (const p of cmd.params) {
      const val = (params[p.name] || '').trim();
      if (!val) continue;
      if (p.kind === 'integer') {
        parsed[p.name] = parseInt(val, 10);
      } else if (p.kind === 'array') {
        try { parsed[p.name] = JSON.parse(val); }
        catch (e) { setResult(`Invalid JSON for ${p.name}: ${e}`); return null; }
      } else {
        parsed[p.name] = val;
      }
    }
    return parsed;
  };

  const handleExecute = async () => {
    const parsed = parseParams();
    if (!parsed) return;

    setCliCmd(buildCliCommand(selected, cdpPort, parsed));
    setRunning(true);
    setResult('');
    try {
      const res = await api.executeCommand(selected, parsed);
      setResult(JSON.stringify(res, null, 2));
    } catch (e) {
      setResult(`Network error: ${e}`);
    } finally {
      setRunning(false);
    }
  };

  if (loading) {
    return (
      <>
        <Nav authenticated />
        <main className="max-w-7xl mx-auto px-4 sm:px-6 py-16 flex justify-center"><Spinner /></main>
      </>
    );
  }

  return (
    <>
      <Nav authenticated />
      <main className="w-[80vw] mx-auto px-4 sm:px-6 py-16">
        <div className="grid gap-6 lg:grid-cols-[1fr_320px]">
          {/* Left: executor */}
          <Card hover={false}>
            <h1 className="text-2xl font-bold text-slate-900 mb-1">{cmd?.name || tr.title}</h1>
            <p className="text-sm text-slate-500 mb-6">{cmd?.summary || tr.description}</p>

            <div className="space-y-4">
              {/* Account selector */}
              <div>
                <label>{tr.account}</label>
                {accounts.length === 0 ? (
                  <p className="mt-1 text-xs text-amber-600">{tr.no_accounts}</p>
                ) : (
                  <select
                    value={cdpPort}
                    onChange={(e) => { setCdpPort(e.target.value); localStorage.setItem('notebooklm-cli:last-cdp-port', e.target.value); }}
                    className="mt-1"
                  >
                    {accounts.map((a) => (
                      <option key={a.cdp_port} value={a.cdp_port}>
                        {a.email ? `${a.email} — port ${a.cdp_port}` : `port ${a.cdp_port}`}
                      </option>
                    ))}
                  </select>
                )}
              </div>

              {cmd?.params.map((p) => (
                <div key={p.name}>
                  <label>
                    {p.name}{p.required && <span className="text-red-500 ml-1">*</span>}
                  </label>
                  {p.kind === 'array' ? (
                    <textarea
                      rows={2}
                      placeholder={`${p.description} (JSON array)`}
                      value={params[p.name] || ''}
                      onChange={(e) => handleParamChange(p.name, e.target.value)}
                      className="mt-1"
                    />
                  ) : (
                    <input
                      type={p.kind === 'integer' ? 'number' : 'text'}
                      placeholder={p.description}
                      value={params[p.name] || ''}
                      onChange={(e) => handleParamChange(p.name, e.target.value)}
                      className="mt-1"
                    />
                  )}
                </div>
              ))}

              <button
                onClick={handleExecute}
                disabled={running}
                className="btn-primary"
              >
                {running ? <><Spinner /> {tr.running}</> : tr.execute}
              </button>

              {cliCmd && (
                <pre className="mt-4 text-xs"><span className="text-slate-400">$ </span>{cliCmd}</pre>
              )}

              {result && <pre className="mt-2 max-h-96 overflow-auto whitespace-pre-wrap break-words">{result}</pre>}
            </div>
          </Card>

          {/* Right: command list */}
          <Card hover={false}>
            <h2 className="text-lg font-bold text-slate-900 mb-4">{tr.registered}</h2>
            <ul className="space-y-1">
              {commands.map((c) => (
                <li key={c.name}>
                  <button
                    onClick={() => handleSelect(c.name)}
                    className={`w-full text-left px-3 py-2 rounded-lg text-sm transition-colors ${
                      c.name === selected
                        ? 'bg-brand-50 text-brand-600 font-semibold'
                        : 'text-slate-700 hover:bg-slate-50'
                    }`}
                  >
                    {c.name}
                  </button>
                </li>
              ))}
            </ul>
          </Card>
        </div>
      </main>
    </>
  );
}
