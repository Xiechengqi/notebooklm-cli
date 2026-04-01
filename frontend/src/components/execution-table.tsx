'use client';

import { StatusDot } from './status-dot';
import { useLang } from '@/lib/use-lang';
import { t } from '@/lib/i18n';
import type { ExecutionRecord } from '@/lib/types';

function formatTimestamp(epoch: number, lang: 'en' | 'zh'): string {
  const tr = t(lang).components;
  const now = Date.now() / 1000;
  const diff = Math.max(0, now - epoch);
  if (diff < 60) return tr.just_now;
  if (diff < 3600) return `${Math.floor(diff / 60)}${tr.minutes_ago}`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}${tr.hours_ago}`;
  return `${Math.floor(diff / 86400)}${tr.days_ago}`;
}

export function ExecutionTable({ records, showAll = false }: { records: ExecutionRecord[]; showAll?: boolean }) {
  const { lang } = useLang();
  const tr = t(lang).components;

  if (records.length === 0) {
    return <p className="text-sm text-slate-500">{tr.no_executions}</p>;
  }

  const displayed = showAll ? records : records.slice(-6).reverse();

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-sm">
        <thead>
          <tr className="text-left text-xs font-semibold uppercase tracking-wider text-slate-500 border-b border-slate-200">
            <th className="pb-2 pr-4">{tr.when}</th>
            <th className="pb-2 pr-4">{tr.source}</th>
            <th className="pb-2 pr-4">{tr.command}</th>
            <th className="pb-2 pr-4">{tr.status}</th>
            <th className="pb-2">{tr.summary_heading}</th>
          </tr>
        </thead>
        <tbody>
          {displayed.map((r, i) => (
            <tr
              key={i}
              className="border-b border-slate-100 last:border-0"
            >
              <td className="py-2 pr-4 text-slate-500 whitespace-nowrap">
                {formatTimestamp(r.timestamp, lang)}
              </td>
              <td className="py-2 pr-4 text-slate-600">{r.source}</td>
              <td className="py-2 pr-4 font-medium text-slate-700">
                {r.command}
              </td>
              <td className="py-2 pr-4">
                <span className="flex items-center gap-1.5">
                  <StatusDot ok={r.ok} />
                  <span className={r.ok ? 'text-emerald-600' : 'text-red-600'}>
                    {r.ok ? tr.status_ok : tr.status_err}
                  </span>
                </span>
              </td>
              <td className="py-2 text-slate-600 max-w-[400px] break-words whitespace-pre-wrap">
                {r.summary}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
