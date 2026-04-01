'use client';

import { useEffect, useState } from 'react';
import { Nav } from '@/components/nav';
import { Card } from '@/components/card';
import { Spinner } from '@/components/spinner';
import { useLang } from '@/lib/use-lang';
import { t } from '@/lib/i18n';
import * as api from '@/lib/api';
import type { CommandSpec, ToolSpec } from '@/lib/types';

export default function DocsPage() {
  const { lang } = useLang();
  const tr = t(lang).docs;
  const [commands, setCommands] = useState<CommandSpec[]>([]);
  const [tools, setTools] = useState<ToolSpec[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    (async () => {
      try {
        const [cmdRes, toolRes] = await Promise.all([api.getCommands(), api.getMcpTools()]);
        setCommands(cmdRes.data);
        setTools(toolRes.data);
      } catch { /* 401 */ }
      finally { setLoading(false); }
    })();
  }, []);

  if (loading) {
    return (
      <>
        <Nav authenticated />
        <main className="max-w-5xl mx-auto px-4 sm:px-6 py-16 flex justify-center"><Spinner /></main>
      </>
    );
  }

  const baseUrl = typeof window !== 'undefined' ? `${window.location.protocol}//${window.location.host}` : '';

  return (
    <>
      <Nav authenticated />
      <main className="max-w-5xl mx-auto px-4 sm:px-6 py-16 space-y-8">
        <div>
          <h1 className="text-2xl font-bold text-slate-900 mb-2">{tr.title}</h1>
          <p className="text-sm text-slate-500">{tr.description}</p>
        </div>

        {/* Claude Code Integration */}
        <Card hover={false}>
          <h2 className="text-lg font-bold text-slate-900 mb-4 flex items-center gap-2">
            Claude Code
            <span className="text-xs px-1.5 py-0.5 rounded-full bg-brand-50 text-brand-600">MCP</span>
          </h2>
          <div className="space-y-4 text-sm">
            <div>
              <h3 className="font-semibold text-slate-700 mb-2">{lang === 'zh' ? '添加 MCP Server' : 'Add MCP Server'}</h3>
              <p className="text-slate-500 mb-2">
                {lang === 'zh'
                  ? '在终端运行以下命令，将 notebooklm-cli 注册为 Claude Code 的 MCP server：'
                  : 'Run the following command in your terminal to register notebooklm-cli as a Claude Code MCP server:'}
              </p>
              <pre className="bg-slate-50 border border-slate-200 rounded-lg p-4 text-xs overflow-x-auto whitespace-pre-wrap break-all">
{`claude mcp add --transport http --header "Authorization: Bearer <PASSWORD>" --scope user notebooklm-cli ${baseUrl}/mcp`}
              </pre>
              <p className="text-slate-400 text-xs mt-2">
                {lang === 'zh'
                  ? '将 <PASSWORD> 替换为控制台密码。添加后重启 Claude Code 生效。'
                  : 'Replace <PASSWORD> with your console password. Restart Claude Code after adding.'}
              </p>
            </div>

            <div className="bg-brand-50 rounded-lg p-4">
              <h3 className="font-semibold text-brand-700 mb-2">{lang === 'zh' ? '认证方式' : 'Authentication'}</h3>
              <p className="text-slate-600 text-xs">
                {lang === 'zh'
                  ? '使用控制台密码作为 Bearer token。前往设置页面可以修改密码。'
                  : 'Use the Console password as the Bearer token. Go to Settings to change it.'}
              </p>
            </div>
          </div>
        </Card>

        {/* MCP API Reference */}
        <Card hover={false}>
          <h2 className="text-lg font-bold text-slate-900 mb-4 flex items-center gap-2">
            MCP API
            <span className="text-xs px-1.5 py-0.5 rounded-full bg-violet-50 text-violet-600">REST</span>
          </h2>
          <div className="space-y-4 text-sm">
            <div>
              <h3 className="font-semibold text-slate-700 mb-2">POST /mcp</h3>
              <p className="text-slate-500 mb-2">{lang === 'zh' ? 'MCP JSON-RPC 2.0 端点，支持 tools/list 和 tools/call 方法。' : 'MCP JSON-RPC 2.0 endpoint. Supports tools/list and tools/call methods.'}</p>
              <pre className="bg-slate-50 border border-slate-200 rounded-lg p-4 text-xs overflow-x-auto">
{`# List all tools
curl -X POST ${baseUrl}/mcp \\
  -H "Content-Type: application/json" \\
  -H "Authorization: Bearer <PASSWORD>" \\
  -d '{"jsonrpc":"2.0","id":"1","method":"tools/list","params":{}}'

# Call a tool
curl -X POST ${baseUrl}/mcp \\
  -H "Content-Type: application/json" \\
  -H "Authorization: Bearer <PASSWORD>" \\
  -d '{
    "jsonrpc":"2.0",
    "id":"1",
    "method":"tools/call",
    "params": {
      "name": "notebooklm_list",
      "arguments": {"cdp_port": "9222"}
    }
  }'`}
              </pre>
            </div>
          </div>
        </Card>

        {/* MCP Tools Table */}
        <Card hover={false}>
          <h2 className="text-lg font-bold text-slate-900 mb-4 flex items-center gap-2">
            {lang === 'zh' ? 'MCP 工具' : 'MCP Tools'}
            <span className="text-xs px-1.5 py-0.5 rounded-full bg-slate-100 text-slate-600">{tools.length}</span>
          </h2>
          <div className="overflow-x-auto">
            <table className="w-full text-sm text-left">
              <thead>
                <tr className="border-b border-slate-200">
                  <th className="py-2 pr-4 font-semibold text-slate-700">{lang === 'zh' ? '工具' : 'Tool'}</th>
                  <th className="py-2 pr-4 font-semibold text-slate-700">{lang === 'zh' ? '命令' : 'Command'}</th>
                  <th className="py-2 pr-4 font-semibold text-slate-700">{lang === 'zh' ? '类型' : 'Type'}</th>
                </tr>
              </thead>
              <tbody>
                {tools.map((tool) => (
                  <tr key={tool.name} className="border-b border-slate-100 last:border-0">
                    <td className="py-2 pr-4 font-semibold text-slate-900">{tool.name}</td>
                    <td className="py-2 pr-4 text-slate-600">{tool.command}</td>
                    <td className="py-2">
                      <span className={`text-xs px-1.5 py-0.5 rounded-full ${
                        tool.read_only
                          ? 'bg-emerald-50 text-emerald-600'
                          : 'bg-amber-50 text-amber-600'
                      }`}>
                        {tool.read_only ? (lang === 'zh' ? '只读' : 'read') : (lang === 'zh' ? '写入' : 'write')}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>

        {/* CLI Commands Table */}
        <Card hover={false}>
          <h2 className="text-lg font-bold text-slate-900 mb-4 flex items-center gap-2">
            CLI Commands
            <span className="text-xs px-1.5 py-0.5 rounded-full bg-slate-100 text-slate-600">{commands.length}</span>
          </h2>
          <div className="overflow-x-auto">
            <table className="w-full text-sm text-left">
              <thead>
                <tr className="border-b border-slate-200">
                  <th className="py-2 pr-4 font-semibold text-slate-700">{tr.command}</th>
                  <th className="py-2 pr-4 font-semibold text-slate-700">{tr.category}</th>
                  <th className="py-2 pr-4 font-semibold text-slate-700">{tr.mode}</th>
                  <th className="py-2 font-semibold text-slate-700">{tr.summary}</th>
                </tr>
              </thead>
              <tbody>
                {commands.map((c) => (
                  <tr key={c.name} className="border-b border-slate-100 last:border-0">
                    <td className="py-2 pr-4 font-semibold text-slate-900">{c.name}</td>
                    <td className="py-2 pr-4">
                      <span className={`text-xs px-1.5 py-0.5 rounded-full ${
                        c.category === 'read'
                          ? 'bg-emerald-50 text-emerald-600'
                          : 'bg-amber-50 text-amber-600'
                      }`}>{c.category}</span>
                    </td>
                    <td className="py-2 pr-4 text-slate-600">{c.execution_mode}</td>
                    <td className="py-2 text-slate-600">{c.summary}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>
      </main>
    </>
  );
}
