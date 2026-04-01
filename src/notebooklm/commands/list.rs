use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::AppResult;
use crate::notebooklm::rpc;

#[derive(Debug, Deserialize)]
struct ListResult {
    notebooks: Vec<NotebookEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
struct NotebookEntry {
    id: String,
    title: String,
    url: String,
    #[serde(default)]
    is_owner: Option<bool>,
    #[serde(default)]
    created_at: Option<String>,
}

pub async fn execute(client: &AgentBrowserClient, _params: &Value) -> AppResult<Value> {
    rpc::ensure_home_page(client).await?;

    let result = rpc::call_rpc(
        client,
        rpc::RPC_LIST,
        "[null, 1, null, [2]]",
    )
    .await?;

    // Parse the RPC result into notebook entries via JS
    let script = format!(
        r#"(() => {{
            const result = {result};
            if (!Array.isArray(result) || result.length === 0) return JSON.stringify({{ notebooks: [] }});
            const rawNotebooks = Array.isArray(result[0]) ? result[0] : result;
            if (!Array.isArray(rawNotebooks)) return JSON.stringify({{ notebooks: [] }});

            const notebooks = rawNotebooks
                .filter(item => Array.isArray(item))
                .map(item => {{
                    const meta = Array.isArray(item[5]) ? item[5] : [];
                    const timestamps = Array.isArray(meta[5]) ? meta[5] : [];
                    const id = typeof item[2] === 'string' ? item[2] : '';
                    let title = typeof item[0] === 'string' ? item[0].replace(/^thought\s*\n/, '') : '';
                    title = title.replace(/\s+/g, ' ').trim() || 'Untitled Notebook';

                    let created_at = null;
                    if (timestamps.length > 0) {{
                        const ts = Array.isArray(timestamps[0]) ? timestamps[0][0] : timestamps[0];
                        if (typeof ts === 'number' && isFinite(ts)) {{
                            created_at = new Date(ts * 1000).toISOString();
                        }}
                    }}

                    return {{
                        id,
                        title,
                        url: 'https://notebooklm.google.com/notebook/' + id,
                        is_owner: meta.length > 1 ? meta[1] === false ? false : true : true,
                        created_at,
                    }};
                }})
                .filter(row => row.id);

            return JSON.stringify({{ notebooks }});
        }})()"#,
        result = serde_json::to_string(&result).unwrap_or("null".to_string()),
    );

    let parsed: ListResult = client.eval_json(&script).await?;
    Ok(json!(parsed.notebooks))
}
