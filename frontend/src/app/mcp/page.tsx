'use client';

import { useEffect, useState, useCallback, useMemo } from 'react';
import { Nav } from '@/components/nav';
import { Card } from '@/components/card';
import { Spinner } from '@/components/spinner';
import { useLang } from '@/lib/use-lang';
import { t } from '@/lib/i18n';
import * as api from '@/lib/api';
import type { AccountEntry, ToolSpec, CommandSpec } from '@/lib/types';

function buildCurlCommand(toolName: string, args: Record<string, unknown>): string {
  const body = JSON.stringify({
    jsonrpc: '2.0',
    id: 'console',
    method: 'tools/call',
    params: { name: toolName, arguments: args },
  });
  return `curl -X POST http://localhost:12234/mcp \\\n  -H 'Content-Type: application/json' \\\n  -H 'Authorization: Bearer <password>' \\\n  -d '${body}'`;
}

export default function McpPage() {
  const { lang } = useLang();
  const tr = t(lang).mcp;
  const [tools, setTools] = useState<ToolSpec[]>([]);
  const [commands, setCommands] = useState<CommandSpec[]>([]);
  const [accounts, setAccounts] = useState<AccountEntry[]>([]);
  const [toolName, setToolName] = useState('');
  const [cdpPort, setCdpPort] = useState('');
  const [params, setParams] = useState<Record<string, string>>({});
  const [result, setResult] = useState('');
  const [curlCmd, setCurlCmd] = useState('');
  const [running, setRunning] = useState(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    (async () => {
      try {
        const [toolRes, cmdRes, accRes] = await Promise.all([
          api.getMcpTools(), api.getCommands(), api.getAccounts(),
        ]);
        setTools(toolRes.data);
        setCommands(cmdRes.data);
        if (toolRes.data.length > 0) {
          const cached = localStorage.getItem('notebooklm-cli:last-mcp-tool');
          const initial = cached && toolRes.data.some((t) => t.name === cached)
            ? cached
            : toolRes.data[0].name;
          setToolName(initial);
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

  const selectedTool = useMemo(() => tools.find((t) => t.name === toolName), [tools, toolName]);
  const cmdSpec = useMemo(() => {
    if (!selectedTool) return undefined;
    return commands.find((c) => c.name === selectedTool.command);
  }, [commands, selectedTool]);

  const handleToolChange = useCallback((name: string) => {
    setToolName(name);
    localStorage.setItem('notebooklm-cli:last-mcp-tool', name);
    setParams({});
    setResult('');
    setCurlCmd('');
  }, []);

  const handleParamChange = (name: string, value: string) => {
    setParams((prev) => ({ ...prev, [name]: value }));
  };

  const parseParams = (): Record<string, unknown> | null => {
    const parsed: Record<string, unknown> = { cdp_port: cdpPort };
    if (!cmdSpec) return parsed;
    for (const p of cmdSpec.params) {
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

  const handleCall = async () => {
    const parsed = parseParams();
    if (!parsed) return;

    setCurlCmd(buildCurlCommand(toolName, parsed));
    setRunning(true);
    setResult('');
    try {
      const res = await api.callMcpTool(toolName, parsed);
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
          {/* Left: caller */}
          <Card hover={false}>
            <h1 className="text-2xl font-bold text-slate-900 mb-1">{selectedTool?.name || tr.title}</h1>
            <p className="text-sm text-slate-500 mb-4">
              {selectedTool ? <>{tr.endpoint}<code>/mcp</code> &middot; command: <code>{selectedTool.command}</code></> : tr.description}
            </p>
            <pre className="mb-6 text-xs">Authorization: Bearer &lt;console-password&gt;</pre>

            <div className="space-y-4">
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

              {cmdSpec?.params.map((p) => (
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

              <button onClick={handleCall} disabled={running} className="btn-primary">
                {running ? <><Spinner /> {tr.call_tool}</> : tr.call_tool}
              </button>

              {curlCmd && (
                <pre className="mt-4 text-xs whitespace-pre-wrap"><span className="text-slate-400">$ </span>{curlCmd}</pre>
              )}

              {result && <pre className="mt-2 max-h-96 overflow-auto whitespace-pre-wrap break-words">{result}</pre>}
            </div>
          </Card>

          {/* Right: tool list */}
          <Card hover={false}>
            <h2 className="text-lg font-bold text-slate-900 mb-4">{tr.tools_heading}</h2>
            <ul className="space-y-1">
              {tools.map((tool) => (
                <li key={tool.name}>
                  <button
                    onClick={() => handleToolChange(tool.name)}
                    className={`w-full text-left px-3 py-2 rounded-lg text-sm transition-colors ${
                      tool.name === toolName
                        ? 'bg-brand-50 text-brand-600 font-semibold'
                        : 'text-slate-700 hover:bg-slate-50'
                    }`}
                  >
                    {tool.name}
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
