use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::{AppError, AppResult};
use crate::notebooklm::rpc;

#[derive(Debug, Deserialize)]
struct FulltextResult {
    #[serde(default)]
    fulltext: Option<FulltextEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
struct FulltextEntry {
    source_id: String,
    notebook_id: String,
    title: String,
    content: String,
    char_count: u64,
    #[serde(default)]
    kind: Option<String>,
}

pub async fn execute(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let query = params
        .get("source")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();

    if query.is_empty() {
        return Err(AppError::InvalidParams("source is required".to_string()));
    }

    let notebook_id = rpc::resolve_notebook_id(client, params).await?;
    rpc::ensure_notebook_page(client, &notebook_id).await?;

    // First, resolve source ID from query
    let source_id = resolve_source_id(client, &notebook_id, &query).await?;

    // Call the fulltext RPC
    let rpc_params = format!(r#"[["{}"], [2], [2]]"#, source_id);
    let result = rpc::call_rpc(client, rpc::RPC_SOURCE_FULLTEXT, &rpc_params).await?;

    let script = format!(
        r#"(() => {{
            const result = {result};
            if (!Array.isArray(result) || result.length === 0 || !Array.isArray(result[0])) {{
                return JSON.stringify({{ fulltext: null }});
            }}

            const source = result[0];
            function findFirstString(v) {{
                if (typeof v === 'string' && v.trim()) return v.trim();
                if (!Array.isArray(v)) return null;
                for (const item of v) {{ const f = findFirstString(item); if (f) return f; }}
                return null;
            }}

            const sourceId = findFirstString(source[0]) || '';
            const title = (typeof source[1] === 'string' ? source[1] : '').replace(/\s+/g, ' ').trim() || 'Untitled source';
            const meta = Array.isArray(source[2]) ? source[2] : [];

            function parseSourceType(code) {{
                if (code === 8) return 'pasted-text';
                if (code === 9) return 'youtube';
                if (code === 2) return 'generated-text';
                if (code === 3) return 'pdf';
                if (code === 4) return 'audio';
                if (code === 5) return 'web';
                if (code === 6) return 'video';
                return code == null ? null : 'type-' + code;
            }}

            function collectLeafStrings(v, results) {{
                if (typeof v === 'string') {{ const t = v.trim(); if (t) results.push(t); return results; }}
                if (!Array.isArray(v)) return results;
                for (const item of v) collectLeafStrings(item, results);
                return results;
            }}

            const contentRoot = Array.isArray(result[3]) && result[3].length > 0 ? result[3][0] : [];
            const content = collectLeafStrings(contentRoot, []).join('\n').trim();

            if (!sourceId || !content) return JSON.stringify({{ fulltext: null }});

            return JSON.stringify({{
                fulltext: {{
                    source_id: sourceId,
                    notebook_id: '{notebook_id}',
                    title,
                    content,
                    char_count: content.length,
                    kind: parseSourceType(meta[4]),
                }}
            }});
        }})()"#,
        result = serde_json::to_string(&result).unwrap_or("null".to_string()),
        notebook_id = notebook_id,
    );

    let parsed: FulltextResult = client.eval_json(&script).await?;
    match parsed.fulltext {
        Some(ft) => Ok(json!(ft)),
        None => Ok(Value::Null),
    }
}

/// Resolve a source query (ID or title) to a source ID.
async fn resolve_source_id(
    client: &AgentBrowserClient,
    notebook_id: &str,
    query: &str,
) -> AppResult<String> {
    // Get source list first
    let rpc_params = format!(r#"["{}", null, [2], null, 0]"#, notebook_id);
    let result = rpc::call_rpc(client, rpc::RPC_NOTEBOOK_DETAIL, &rpc_params).await?;

    #[derive(Deserialize)]
    struct IdResult {
        #[serde(default)]
        source_id: Option<String>,
    }

    let script = format!(
        r#"(() => {{
            let detail = {result};
            while (Array.isArray(detail) && detail.length === 1 && Array.isArray(detail[0])) detail = detail[0];
            if (!Array.isArray(detail) || detail.length < 2) return JSON.stringify({{}});

            const rawSources = Array.isArray(detail[1]) ? detail[1] : [];
            const query = {query}.toLowerCase();

            function findFirstString(v) {{
                if (typeof v === 'string' && v.trim()) return v.trim();
                if (!Array.isArray(v)) return null;
                for (const item of v) {{ const f = findFirstString(item); if (f) return f; }}
                return null;
            }}

            const sources = rawSources.filter(e => Array.isArray(e)).map(entry => ({{
                id: findFirstString(entry[0]) || '',
                title: (typeof entry[1] === 'string' ? entry[1] : '').replace(/\s+/g, ' ').trim(),
            }})).filter(s => s.id);

            // Match by exact id, exact title, partial title
            let found = sources.find(s => s.id.toLowerCase() === query);
            if (!found) found = sources.find(s => s.title.toLowerCase() === query);
            if (!found) {{
                const partial = sources.filter(s => s.title.toLowerCase().includes(query));
                if (partial.length === 1) found = partial[0];
            }}

            return JSON.stringify({{ source_id: found ? found.id : null }});
        }})()"#,
        result = serde_json::to_string(&result).unwrap_or("null".to_string()),
        query = serde_json::to_string(query).unwrap_or("\"\"".to_string()),
    );

    let parsed: IdResult = client.eval_json(&script).await?;
    parsed.source_id.ok_or_else(|| {
        AppError::InvalidParams(format!("source '{}' not found", query))
    })
}
