'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';
import { ArrowRight, Terminal, Wrench, Settings, Copy, Check, Sparkles, MonitorSmartphone } from 'lucide-react';
import { Nav } from '@/components/nav';
import { Card } from '@/components/card';
import { StatusDot } from '@/components/status-dot';
import { Spinner } from '@/components/spinner';
import { useLang } from '@/lib/use-lang';
import { t } from '@/lib/i18n';
import * as api from '@/lib/api';
import type { AppConfig, BootstrapInfo } from '@/lib/types';

function CopyButton({ text, label, copiedLabel }: { text: string; label: string; copiedLabel: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <button
      onClick={handleCopy}
      className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold transition-all duration-200 bg-white/80 hover:bg-white text-brand-600 border border-brand-200 hover:border-brand-300 hover:shadow-sm"
    >
      {copied ? <Check className="h-3.5 w-3.5" /> : <Copy className="h-3.5 w-3.5" />}
      {copied ? copiedLabel : label}
    </button>
  );
}

export default function HomePage() {
  const { lang } = useLang();
  const tr = t(lang).home;
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [bootstrap, setBootstrap] = useState<BootstrapInfo | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    (async () => {
      try {
        const bs = await api.bootstrap();
        if (bs.password_required) {
          window.location.href = '/setup/password';
          return;
        }
        setBootstrap(bs);
        const cfgRes = await api.getConfig();
        setConfig(cfgRes.data);
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
        <main className="max-w-7xl mx-auto px-4 sm:px-6 py-16 flex justify-center">
          <Spinner />
        </main>
      </>
    );
  }

  const baseUrl = config ? `http://${config.server.host}:${config.server.port}` : '';
  const mcpCommand = `claude mcp add --transport http --header "Authorization: Bearer <PASSWORD>" --scope user notebooklm-cli ${baseUrl}/mcp`;

  return (
    <>
      <Nav authenticated />
      <main className="max-w-7xl mx-auto px-4 sm:px-6 py-16">
        {/* Hero */}
        <div className="mb-12 text-center">
          <h1 className="text-4xl sm:text-5xl lg:text-6xl font-extrabold tracking-tight gradient-text mb-4">
            notebooklm-cli
          </h1>
          <p className="text-lg text-slate-500 max-w-2xl mx-auto">
            {tr.tagline}<code className="font-semibold text-brand-600">agent-browser</code>{tr.tagline_suffix}
          </p>
        </div>

        {/* Claude Code Integration — Hero CTA */}
        <div className="relative mb-12 rounded-2xl overflow-hidden bg-gradient-to-br from-indigo-600 to-violet-600 p-[1px]">
          <div className="relative rounded-2xl bg-gradient-to-br from-indigo-50 via-white to-violet-50 p-8 sm:p-10">
            {/* Badge */}
            <div className="flex items-center gap-2 mb-4">
              <span className="flex items-center gap-1.5 px-3 py-1 rounded-full bg-gradient-to-r from-indigo-600 to-violet-600 text-white text-xs font-bold shadow-[0_4px_14px_0_rgba(79,70,229,0.3)]">
                <Sparkles className="h-3.5 w-3.5" />
                {tr.claude_code_badge}
              </span>
            </div>

            {/* Heading */}
            <h2 className="text-2xl sm:text-3xl font-extrabold text-slate-900 mb-2 tracking-tight">
              {tr.claude_code_title}
            </h2>
            <p className="text-slate-500 mb-6 max-w-2xl">
              {tr.claude_code_subtitle}
            </p>

            {/* Command block */}
            <div className="mb-4">
              <p className="text-sm font-medium text-slate-600 mb-2">{tr.claude_code_step1}</p>
              <div className="relative group">
                <pre className="bg-slate-900 text-slate-100 rounded-xl p-4 pr-24 text-sm overflow-x-auto font-mono leading-relaxed">
                  {mcpCommand}
                </pre>
                <div className="absolute top-3 right-3">
                  <CopyButton text={mcpCommand} label={tr.copy} copiedLabel={tr.copied} />
                </div>
              </div>
              <p className="text-slate-400 text-xs mt-2">
                {tr.claude_code_replace}
              </p>
            </div>

            {/* Auth info */}
            <div className="bg-brand-50/60 rounded-xl p-4 border border-brand-100">
              <h3 className="font-semibold text-brand-700 text-sm mb-1">{tr.claude_code_auth_title}</h3>
              <p className="text-slate-600 text-xs">{tr.claude_code_auth_desc}</p>
            </div>
          </div>
        </div>

        {/* Info Grid */}
        <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
          {/* Service Status */}
          <Card>
            <h2 className="text-lg font-bold text-slate-900 mb-4">{tr.service_status}</h2>
            <dl className="space-y-3">
              <div><dt>{tr.dt_api}</dt><dd><code>{baseUrl}</code></dd></div>
              <div><dt>{tr.dt_config}</dt><dd><code>~/.config/notebooklm-cli/config.toml</code></dd></div>
            </dl>
          </Card>

          {/* Agent Browser */}
          <Card>
            <h2 className="text-lg font-bold text-slate-900 mb-4">{tr.agent_browser}</h2>
            <dl className="space-y-3">
              <div>
                <dt>{tr.dt_binary}</dt>
                <dd>
                  {config?.agent_browser.binary
                    ? <code>{config.agent_browser.binary}</code>
                    : <span className="text-amber-600 text-xs">{tr.binary_not_set} <a href="https://github.com/vercel-labs/agent-browser" target="_blank" rel="noopener noreferrer" className="underline hover:text-amber-700">github.com/vercel-labs/agent-browser</a></span>
                  }
                </dd>
              </div>
              <div>
                <dt>{tr.dt_cdp}</dt>
                <dd className="flex items-center gap-2">
                  {bootstrap?.cdp.ports.length === 0 ? (
                    <span className="text-amber-600 text-xs">{tr.cdp_not_set}</span>
                  ) : (
                    <>
                      <span className="text-xs">
                        <StatusDot ok={(bootstrap?.cdp.online ?? 0) > 0} />
                        <span className="text-emerald-600 font-medium">{bootstrap?.cdp.online}{tr.cdp_online}</span>
                        {' · '}
                        <span className="text-slate-500">{bootstrap?.cdp.offline}{tr.cdp_offline}</span>
                      </span>
                      <Link href="/cdp" className="text-xs text-brand-600 hover:underline font-medium ml-auto flex items-center gap-1">
                        <MonitorSmartphone className="h-3 w-3" />
                        {tr.cdp_manage}
                      </Link>
                    </>
                  )}
                </dd>
              </div>
            </dl>
          </Card>

          {/* Quick Actions */}
          <Card>
            <h2 className="text-lg font-bold text-slate-900 mb-4">{tr.quick_actions}</h2>
            <div className="space-y-3">
              {[
                { href: '/commands', icon: Terminal, label: tr.action_commands },
                { href: '/mcp', icon: Wrench, label: tr.action_mcp },
                { href: '/settings', icon: Settings, label: tr.action_settings },
              ].map((item) => (
                <Link
                  key={item.href}
                  href={item.href}
                  className="group flex items-center gap-3 p-3 -mx-3 rounded-lg hover:bg-slate-50 transition-colors"
                >
                  <div className="flex items-center justify-center w-10 h-10 rounded-xl bg-brand-50 text-brand-600">
                    <item.icon className="h-5 w-5" />
                  </div>
                  <span className="text-sm font-medium text-slate-700 flex-1">{item.label}</span>
                  <ArrowRight className="h-4 w-4 text-slate-400 transition-transform group-hover:translate-x-1" />
                </Link>
              ))}
            </div>
          </Card>
        </div>
      </main>
    </>
  );
}
