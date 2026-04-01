'use client';

import { useEffect, useState } from 'react';
import { Nav } from '@/components/nav';
import { Card } from '@/components/card';
import { Spinner } from '@/components/spinner';
import { useLang } from '@/lib/use-lang';
import { t } from '@/lib/i18n';
import * as api from '@/lib/api';
import type { SkillSpec } from '@/lib/types';

export default function SkillsPage() {
  const { lang } = useLang();
  const tr = t(lang).skills;
  const [skills, setSkills] = useState<SkillSpec[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    (async () => {
      try {
        const res = await api.getSkills();
        setSkills(res.data);
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

  const skillDocs: Record<string, { description: string; steps: { step: string; command: string; params: string }[]; example: string }> = {
    research_notebook: {
      description: lang === 'zh'
        ? '获取 notebook 概要：摘要 + source 列表 + 对话历史。适用于快速了解 notebook 内容和对话上下文。'
        : 'Get notebook overview: summary + source list + conversation history. Useful for quickly understanding notebook contents and conversation context.',
      steps: [
        { step: '1', command: 'summary', params: 'notebook_id' },
        { step: '2', command: 'source_list', params: 'notebook_id' },
        { step: '3', command: 'history', params: 'notebook_id' },
      ],
      example: lang === 'zh'
        ? 'research_notebook notebook_id\n→ 获取摘要、source 列表、对话历史'
        : 'research_notebook notebook_id\n→ Get summary, source list, conversation history',
    },
    deep_read_source: {
      description: lang === 'zh'
        ? '深度阅读 source：指南摘要 + 全文内容。适用于深入了解某个 source 的完整信息。'
        : 'Deep read a source: guide summary + full text content. Useful for thoroughly understanding a specific source.',
      steps: [
        { step: '1', command: 'source_guide', params: 'notebook_id, source_id' },
        { step: '2', command: 'source_fulltext', params: 'notebook_id, source_id' },
      ],
      example: lang === 'zh'
        ? 'deep_read_source notebook_id source_id\n→ 获取 source 指南摘要和全文'
        : 'deep_read_source notebook_id source_id\n→ Get source guide summary and full text',
    },
    notebook_overview: {
      description: lang === 'zh'
        ? '全局概览：列出所有 notebook 并检查状态。适用于了解账号下所有 notebook 的概况。'
        : 'Global overview: list all notebooks and check status. Useful for getting a bird\'s-eye view of all notebooks.',
      steps: [
        { step: '1', command: 'status', params: '-' },
        { step: '2', command: 'list', params: '-' },
      ],
      example: lang === 'zh'
        ? 'notebook_overview\n→ 检查状态并列出所有 notebook'
        : 'notebook_overview\n→ Check status and list all notebooks',
    },
  };

  return (
    <>
      <Nav authenticated />
      <main className="max-w-5xl mx-auto px-4 sm:px-6 py-16 space-y-8">
        <div>
          <h1 className="text-2xl font-bold text-slate-900 mb-2">{tr.skills_title}</h1>
          <p className="text-sm text-slate-500">{tr.skills_description}</p>
        </div>

        {skills.map((skill) => {
          const doc = skillDocs[skill.name];
          return (
            <Card key={skill.name} hover={false}>
              <div className="mb-4">
                <h2 className="text-lg font-bold text-slate-900 flex items-center gap-2">
                  <span className="text-brand-600">{skill.name}</span>
                  {skill.requires_auth && (
                    <span className="text-xs px-1.5 py-0.5 rounded-full bg-amber-50 text-amber-600">auth</span>
                  )}
                </h2>
                <p className="text-sm text-slate-500 mt-1">{doc?.description || skill.summary}</p>
              </div>

              {doc && (
                <>
                  <div className="mb-4">
                    <h3 className="text-sm font-semibold text-slate-700 mb-2">{lang === 'zh' ? '执行步骤' : 'Execution Steps'}</h3>
                    <div className="space-y-2">
                      {doc.steps.map((s, i) => (
                        <div key={i} className="flex items-start gap-3 text-sm">
                          <span className="w-6 h-6 rounded-full bg-brand-50 text-brand-600 flex items-center justify-center text-xs font-bold flex-shrink-0 mt-0.5">
                            {s.step}
                          </span>
                          <div>
                            <code className="text-brand-600">{s.command}</code>
                            {s.params !== '-' && <span className="text-slate-400 ml-1">({s.params})</span>}
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>

                  <div className="bg-slate-50 rounded-lg p-3 text-sm">
                    <span className="text-slate-400 font-mono">{doc.example}</span>
                  </div>
                </>
              )}
            </Card>
          );
        })}
      </main>
    </>
  );
}
