'use client';

import { useEffect, useState } from 'react';
import { Nav } from '@/components/nav';
import { Card } from '@/components/card';
import { Spinner } from '@/components/spinner';
import { useLang } from '@/lib/use-lang';
import { t } from '@/lib/i18n';
import * as api from '@/lib/api';
import type { PreviewNoteEntry, PreviewSyncStatus } from '@/lib/types';

function formatTime(value: number | null, fallback: string) {
  if (!value) {
    return fallback;
  }
  return new Date(value * 1000).toLocaleString();
}

type NotebookGroup = {
  notebookId: string;
  notebookTitle: string;
  notes: PreviewNoteEntry[];
};

type AccountGroup = {
  account: string;
  notebooks: NotebookGroup[];
};

function groupNotes(notes: PreviewNoteEntry[], unknownAccount: string): AccountGroup[] {
  const accountMap = new Map<string, Map<string, NotebookGroup>>();

  for (const note of notes) {
    const account = note.google_account || unknownAccount;
    let notebookMap = accountMap.get(account);
    if (!notebookMap) {
      notebookMap = new Map();
      accountMap.set(account, notebookMap);
    }

    const notebookKey = `${note.notebook_id}:${note.notebook_title}`;
    let notebook = notebookMap.get(notebookKey);
    if (!notebook) {
      notebook = {
        notebookId: note.notebook_id,
        notebookTitle: note.notebook_title || note.notebook_id,
        notes: [],
      };
      notebookMap.set(notebookKey, notebook);
    }
    notebook.notes.push(note);
  }

  return Array.from(accountMap.entries())
    .map(([account, notebooks]) => ({
      account,
      notebooks: Array.from(notebooks.values())
        .map((notebook) => ({
          ...notebook,
          notes: [...notebook.notes].sort((a, b) => b.fetched_at - a.fetched_at),
        }))
        .sort((a, b) => a.notebookTitle.localeCompare(b.notebookTitle)),
    }))
    .sort((a, b) => a.account.localeCompare(b.account));
}

export default function PreviewPage() {
  const { lang } = useLang();
  const tr = t(lang).preview;
  const [notes, setNotes] = useState<PreviewNoteEntry[]>([]);
  const [status, setStatus] = useState<PreviewSyncStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [syncing, setSyncing] = useState(false);
  const groups = groupNotes(notes, tr.unknown_account);

  const load = async () => {
    const [notesRes, statusRes] = await Promise.all([
      api.getPreviewNotes(),
      api.getPreviewStatus(),
    ]);
    setNotes(notesRes.data);
    setStatus(statusRes.data);
  };

  useEffect(() => {
    (async () => {
      try {
        const bs = await api.bootstrap();
        if (bs.password_required) {
          window.location.href = '/setup/password';
          return;
        }
        setStatus(bs.preview);
        await load();
      } catch {
        // 401 handled by api wrapper
      } finally {
        setLoading(false);
      }
    })();
  }, []);

  const handleSync = async () => {
    setSyncing(true);
    try {
      await api.triggerPreviewSync();
      await load();
    } finally {
      setSyncing(false);
    }
  };

  if (loading) {
    return (
      <>
        <Nav authenticated />
        <main className="max-w-6xl mx-auto px-4 sm:px-6 py-16 flex justify-center">
          <Spinner />
        </main>
      </>
    );
  }

  return (
    <>
      <Nav authenticated />
      <main className="max-w-6xl mx-auto px-4 sm:px-6 py-16 space-y-8">
        <div className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <h1 className="text-2xl font-bold text-slate-900 mb-2">{tr.title}</h1>
            <p className="text-sm text-slate-500">{tr.description}</p>
          </div>
          <button
            onClick={handleSync}
            disabled={syncing || status?.running}
            className="px-4 py-2 rounded-lg bg-brand-600 text-white text-sm font-medium disabled:opacity-60"
          >
            {syncing || status?.running ? tr.syncing : tr.sync_now}
          </button>
        </div>

        <Card hover={false}>
          <div className="grid gap-4 sm:grid-cols-4 text-sm">
            <div>
              <div className="text-slate-500">{tr.last_sync}</div>
              <div className="mt-1 font-medium text-slate-900">
                {formatTime(status?.last_finished_at ?? null, tr.never)}
              </div>
            </div>
            <div>
              <div className="text-slate-500">{tr.added}</div>
              <div className="mt-1 font-medium text-slate-900">{status?.last_added ?? 0}</div>
            </div>
            <div>
              <div className="text-slate-500">{tr.skipped}</div>
              <div className="mt-1 font-medium text-slate-900">{status?.last_skipped ?? 0}</div>
            </div>
            <div>
              <div className="text-slate-500">{tr.failed_ports}</div>
              <div className="mt-1 font-medium text-slate-900">{status?.last_failed_ports ?? 0}</div>
            </div>
          </div>
          {status?.last_error ? (
            <div className="mt-4 rounded-lg bg-red-50 text-red-700 px-3 py-2 text-sm">
              {status.last_error}
            </div>
          ) : null}
        </Card>

        {notes.length === 0 ? (
          <Card hover={false}>
            <p className="text-sm text-slate-500">{tr.empty}</p>
          </Card>
        ) : (
          <div className="space-y-4">
            {groups.map((group) => (
              <Card key={group.account} hover={false}>
                <details open>
                  <summary className="cursor-pointer list-none flex items-center justify-between gap-4">
                    <div>
                      <div className="text-base font-semibold text-slate-900">
                        {tr.account}: {group.account}
                      </div>
                      <div className="text-sm text-slate-500">
                        {group.notebooks.length} {tr.notebooks}
                      </div>
                    </div>
                  </summary>
                  <div className="mt-4 space-y-4">
                    {group.notebooks.map((notebook) => (
                      <details key={`${group.account}:${notebook.notebookId}`} className="rounded-xl border border-slate-200 bg-slate-50 px-4 py-3">
                        <summary className="cursor-pointer list-none flex items-center justify-between gap-4">
                          <div>
                            <div className="text-sm font-semibold text-slate-900">
                              {tr.notebook}: {notebook.notebookTitle}
                            </div>
                            <div className="text-xs text-slate-500">
                              {notebook.notes.length} {tr.notes}
                            </div>
                          </div>
                        </summary>
                        <div className="mt-3 space-y-3">
                          {notebook.notes.map((note) => (
                            <div key={note.id} className="rounded-xl border border-slate-200 bg-white p-4">
                              <div className="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
                                <div>
                                  <h2 className="text-lg font-semibold text-slate-900">{note.note_title}</h2>
                                  <div className="text-sm text-slate-500">
                                    {tr.cdp_port}: {note.cdp_port}
                                  </div>
                                </div>
                                <div className="text-xs text-slate-500">
                                  {formatTime(note.fetched_at, tr.never)}
                                </div>
                              </div>
                              <details className="mt-3">
                                <summary className="cursor-pointer list-none rounded-xl bg-slate-50 border border-slate-200 p-4 text-sm text-slate-700">
                                  {note.content_preview || note.content}
                                </summary>
                                <div className="mt-3 rounded-xl bg-slate-50 border border-slate-200 p-4 whitespace-pre-wrap text-sm text-slate-700">
                                  {note.content}
                                </div>
                              </details>
                            </div>
                          ))}
                        </div>
                      </details>
                    ))}
                  </div>
                </details>
              </Card>
            ))}
          </div>
        )}
      </main>
    </>
  );
}
