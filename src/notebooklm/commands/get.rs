use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::AppResult;
use crate::notebooklm::rpc;

#[derive(Debug, Deserialize)]
struct DetailResult {
    #[serde(default)]
    notebook: Option<NotebookDetail>,
}

#[derive(Debug, Deserialize, Serialize)]
struct NotebookDetail {
    id: String,
    title: String,
    url: String,
    #[serde(default)]
    emoji: Option<String>,
    #[serde(default)]
    source_count: Option<u32>,
    #[serde(default)]
    is_owner: Option<bool>,
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
            // Unwrap singleton arrays
            while (Array.isArray(detail) && detail.length === 1 && Array.isArray(detail[0])) detail = detail[0];
            if (!Array.isArray(detail) || detail.length < 3) return JSON.stringify({{ notebook: null }});

            const id = typeof detail[2] === 'string' ? detail[2] : '';
            if (!id) return JSON.stringify({{ notebook: null }});

            const title = (typeof detail[0] === 'string' ? detail[0] : '').replace(/\s+/g, ' ').trim() || 'Untitled Notebook';
            const emoji = typeof detail[3] === 'string' ? detail[3] : null;
            const meta = Array.isArray(detail[5]) ? detail[5] : [];
            const sources = Array.isArray(detail[1]) ? detail[1] : [];

            function toIso(v) {{
                const s = Array.isArray(v) ? v[0] : v;
                if (typeof s !== 'number' || !isFinite(s)) return null;
                return new Date(s * 1000).toISOString();
            }}

            return JSON.stringify({{
                notebook: {{
                    id,
                    title,
                    url: 'https://notebooklm.google.com/notebook/' + id,
                    emoji,
                    source_count: sources.length,
                    is_owner: meta.length > 1 ? meta[1] !== false : true,
                    created_at: toIso(meta[8]),
                    updated_at: toIso(meta[5]),
                }}
            }});
        }})()"#,
        result = serde_json::to_string(&result).unwrap_or("null".to_string()),
    );

    let parsed: DetailResult = client.eval_json(&script).await?;
    match parsed.notebook {
        Some(nb) => Ok(json!(nb)),
        None => Ok(Value::Null),
    }
}
