use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::{AppError, AppResult};
use crate::notebooklm::rpc;

#[derive(Debug, Deserialize)]
struct GuideResult {
    #[serde(default)]
    guide: Option<GuideEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GuideEntry {
    source_id: String,
    notebook_id: String,
    title: String,
    summary: String,
    keywords: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ResolvedSource {
    #[serde(default)]
    source_id: Option<String>,
    #[serde(default)]
    source_title: Option<String>,
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

    // Resolve source ID and title
    let detail_params = format!(r#"["{}", null, [2], null, 0]"#, notebook_id);
    let detail_result = rpc::call_rpc(client, rpc::RPC_NOTEBOOK_DETAIL, &detail_params).await?;

    let resolve_script = format!(
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

            let found = sources.find(s => s.id.toLowerCase() === query);
            if (!found) found = sources.find(s => s.title.toLowerCase() === query);
            if (!found) {{
                const partial = sources.filter(s => s.title.toLowerCase().includes(query));
                if (partial.length === 1) found = partial[0];
            }}

            return JSON.stringify({{ source_id: found ? found.id : null, source_title: found ? found.title : null }});
        }})()"#,
        result = serde_json::to_string(&detail_result).unwrap_or("null".to_string()),
        query = serde_json::to_string(&query).unwrap_or("\"\"".to_string()),
    );

    let resolved: ResolvedSource = client.eval_json(&resolve_script).await?;
    let source_id = resolved
        .source_id
        .ok_or_else(|| AppError::InvalidParams(format!("source '{}' not found", query)))?;
    let source_title = resolved.source_title.unwrap_or_default();

    // Call guide RPC
    let rpc_params = format!(r#"[[[["{}"]]]"#, source_id) + "]";
    let result = rpc::call_rpc(client, rpc::RPC_SOURCE_GUIDE, &rpc_params).await?;

    let script = format!(
        r#"(() => {{
            const result = {result};
            if (!Array.isArray(result) || result.length === 0 || !Array.isArray(result[0])) {{
                return JSON.stringify({{ guide: null }});
            }}

            const outer = result[0];
            const guide = Array.isArray(outer) && outer.length > 0 && Array.isArray(outer[0]) ? outer[0] : outer;
            if (!Array.isArray(guide)) return JSON.stringify({{ guide: null }});

            const summary = Array.isArray(guide[1]) && typeof guide[1][0] === 'string'
                ? guide[1][0].trim()
                : '';
            const keywords = Array.isArray(guide[2]) && Array.isArray(guide[2][0])
                ? guide[2][0].filter(item => typeof item === 'string' && item.trim().length > 0)
                : [];

            if (!summary) return JSON.stringify({{ guide: null }});

            return JSON.stringify({{
                guide: {{
                    source_id: '{source_id}',
                    notebook_id: '{notebook_id}',
                    title: {source_title},
                    summary,
                    keywords,
                }}
            }});
        }})()"#,
        result = serde_json::to_string(&result).unwrap_or("null".to_string()),
        source_id = source_id,
        notebook_id = notebook_id,
        source_title = serde_json::to_string(&source_title).unwrap_or("\"\"".to_string()),
    );

    let parsed: GuideResult = client.eval_json(&script).await?;
    match parsed.guide {
        Some(g) => Ok(json!(g)),
        None => Ok(Value::Null),
    }
}
