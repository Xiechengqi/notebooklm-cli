use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::AppResult;
use crate::notebooklm::rpc;

#[derive(Debug, Deserialize)]
struct SourceListResult {
    sources: Vec<SourceEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct SourceEntry {
    id: String,
    notebook_id: String,
    title: String,
    url: String,
    #[serde(default, rename = "type")]
    kind: Option<String>,
    #[serde(default)]
    size: Option<u64>,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    updated_at: Option<String>,
}

pub async fn execute(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let notebook_id = rpc::resolve_notebook_id(client, params).await?;
    rpc::ensure_notebook_page(client, &notebook_id).await?;

    let rpc_params = format!(r#"["{}", null, [2], null, 0]"#, notebook_id);
    let result = rpc::call_rpc(client, rpc::RPC_NOTEBOOK_DETAIL, &rpc_params).await?;

    let script = format!(
        r#"(() => {{
            let detail = {result};
            while (Array.isArray(detail) && detail.length === 1 && Array.isArray(detail[0])) detail = detail[0];
            if (!Array.isArray(detail) || detail.length < 3) return JSON.stringify({{ sources: [] }});

            const notebookId = typeof detail[2] === 'string' ? detail[2] : '{notebook_id}';
            const rawSources = Array.isArray(detail[1]) ? detail[1] : [];

            function findFirstString(v) {{
                if (typeof v === 'string' && v.trim()) return v.trim();
                if (!Array.isArray(v)) return null;
                for (const item of v) {{ const f = findFirstString(item); if (f) return f; }}
                return null;
            }}

            function parseSourceType(v) {{
                const code = typeof v === 'number' ? v : (Array.isArray(v) && typeof v[1] === 'number' ? v[1] : null);
                if (code === 8) return 'pasted-text';
                if (code === 9) return 'youtube';
                if (code === 2) return 'generated-text';
                if (code === 3) return 'pdf';
                if (code === 4) return 'audio';
                if (code === 5) return 'web';
                if (code === 6) return 'video';
                return code == null ? null : 'type-' + code;
            }}

            function toIso(v) {{
                const s = Array.isArray(v) ? v[0] : v;
                if (typeof s !== 'number' || !isFinite(s)) return null;
                return new Date(s * 1000).toISOString();
            }}

            const sources = rawSources
                .filter(e => Array.isArray(e))
                .map(entry => {{
                    const id = findFirstString(entry[0]) || '';
                    const title = (typeof entry[1] === 'string' ? entry[1] : '').replace(/\s+/g, ' ').trim() || 'Untitled source';
                    const meta = Array.isArray(entry[2]) ? entry[2] : [];
                    const typeInfo = typeof meta[4] === 'number' ? meta[4] : entry[3];

                    return {{
                        id,
                        notebook_id: notebookId,
                        title,
                        url: 'https://notebooklm.google.com/notebook/' + notebookId,
                        type: parseSourceType(typeInfo),
                        size: typeof meta[1] === 'number' && isFinite(meta[1]) ? meta[1] : null,
                        created_at: toIso(meta[2]),
                        updated_at: toIso(meta[14]),
                    }};
                }})
                .filter(row => row.id);

            return JSON.stringify({{ sources }});
        }})()"#,
        result = serde_json::to_string(&result).unwrap_or("null".to_string()),
        notebook_id = notebook_id,
    );

    let parsed: SourceListResult = client.eval_json(&script).await?;
    Ok(json!(parsed.sources))
}
