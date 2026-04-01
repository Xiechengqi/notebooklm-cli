'use client';

import { useEffect, useState } from 'react';
import { Nav } from '@/components/nav';
import { Card } from '@/components/card';
import { PasswordInput } from '@/components/password-input';
import { Spinner } from '@/components/spinner';
import { useLang } from '@/lib/use-lang';
import { t } from '@/lib/i18n';
import * as api from '@/lib/api';
import type { AppConfig } from '@/lib/types';

export default function SettingsPage() {
  const { lang } = useLang();
  const tr = t(lang).settings;
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [saveResult, setSaveResult] = useState('');
  const [pwResult, setPwResult] = useState('');

  const [host, setHost] = useState('');
  const [port, setPort] = useState('');
  const [timeout, setTimeout_] = useState('');
  const [vncUrl, setVncUrl] = useState('');
  const [newPw, setNewPw] = useState('');
  const [confirmPw, setConfirmPw] = useState('');

  useEffect(() => {
    (async () => {
      try {
        const res = await api.getConfig();
        const c = res.data;
        setConfig(c);
        setHost(c.server.host);
        setPort(String(c.server.port));
        setTimeout_(String(c.agent_browser.timeout_secs));
        setVncUrl(c.vnc.url);
      } catch {
      } finally {
        setLoading(false);
      }
    })();
  }, []);

  const handleSave = async () => {
    setSaveResult('');
    if (!config) return;
    const payload: AppConfig = {
      server: { host, port: parseInt(port, 10) || 12234 },
      agent_browser: {
        binary: config.agent_browser.binary,
        session_name: config.agent_browser.session_name,
        timeout_secs: parseInt(timeout, 10) || 60,
      },
      vnc: { url: vncUrl, embed: config.vnc.embed },
    };
    try {
      const res = await api.updateConfig(payload);
      setConfig(payload);
      setSaveResult(JSON.stringify(res, null, 2));
    } catch (e) {
      setSaveResult(`Error: ${e}`);
    }
  };

  const handleReset = () => window.location.reload();

  const passwordsMatch = newPw === confirmPw;
  const canSubmitPassword = newPw.length > 0 && confirmPw.length > 0 && passwordsMatch;

  const handleChangePassword = async () => {
    setPwResult('');
    if (newPw.length === 0) {
      setPwResult(tr.new_password_required);
      return;
    }
    if (!passwordsMatch) {
      setPwResult(tr.password_mismatch);
      return;
    }
    try {
      const res = await api.changePassword(newPw);
      setPwResult(JSON.stringify(res, null, 2));
      setNewPw('');
      setConfirmPw('');
    } catch (e) {
      setPwResult(`Error: ${e}`);
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
      <main className="max-w-7xl mx-auto px-4 sm:px-6 py-16 space-y-6">
        <Card hover={false}>
          <h2 className="text-lg font-bold text-slate-900 mb-4">{tr.server}</h2>
          <div className="grid gap-4 sm:grid-cols-2">
            <div><label>{tr.host}</label><input type="text" value={host} onChange={(e) => setHost(e.target.value)} /></div>
            <div><label>{tr.port}</label><input type="number" value={port} onChange={(e) => setPort(e.target.value)} /></div>
          </div>
        </Card>

        <Card hover={false}>
          <h2 className="text-lg font-bold text-slate-900 mb-4">{tr.agent_browser}</h2>
          <div className="grid gap-4 sm:grid-cols-2">
            <div>
              <label>{tr.binary}</label>
              <p className="mt-1 text-sm text-slate-600 bg-slate-50 border border-slate-200 rounded-lg px-3 py-2 select-all">
                {config?.agent_browser.binary || <span className="text-slate-400">{tr.not_detected}</span>}
              </p>
            </div>
            <div><label>{tr.timeout}</label><input type="number" value={timeout} onChange={(e) => setTimeout_(e.target.value)} /></div>
          </div>
        </Card>

        <Card hover={false}>
          <h2 className="text-lg font-bold text-slate-900 mb-4">{tr.vnc}</h2>
          <div className="grid gap-4 sm:grid-cols-2">
            <div><label>{tr.url}</label><input type="text" value={vncUrl} onChange={(e) => setVncUrl(e.target.value)} /></div>
          </div>
        </Card>

        <div className="flex gap-3">
          <button onClick={handleSave} className="btn-primary">{tr.save}</button>
          <button onClick={handleReset} className="btn-secondary">{tr.reset}</button>
        </div>
        {saveResult && <pre>{saveResult}</pre>}

        <Card hover={false}>
          <h2 className="text-lg font-bold text-slate-900 mb-4">{tr.change_password}</h2>
          <div className="grid gap-4 sm:grid-cols-2">
            <PasswordInput
              label={tr.new_password}
              value={newPw}
              onChange={setNewPw}
              autoComplete="new-password"
            />
            <PasswordInput
              label={tr.confirm_password}
              value={confirmPw}
              onChange={setConfirmPw}
              autoComplete="new-password"
            />
          </div>
          {newPw.length > 0 && confirmPw.length > 0 && !passwordsMatch && (
            <p className="mt-4 text-sm text-red-600">{tr.password_mismatch}</p>
          )}
          <button onClick={handleChangePassword} className="btn-primary mt-4" disabled={!canSubmitPassword}>
            {tr.change_password}
          </button>
          {pwResult && <pre className="mt-4">{pwResult}</pre>}
        </Card>
      </main>
    </>
  );
}
