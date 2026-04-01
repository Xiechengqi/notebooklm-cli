use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::AppResult;
use crate::notebooklm::rpc;

#[derive(Debug, Deserialize)]
struct SummaryResult {
    #[serde(default)]
    summary: Option<SummaryEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SummaryEntry {
    notebook_id: String,
    title: String,
    summary: String,
    url: String,
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
            if (!Array.isArray(detail)) return JSON.stringify({{ summary: null }});

            const title = (typeof detail[0] === 'string' ? detail[0] : '').replace(/\s+/g, ' ').trim() || 'Untitled Notebook';
            const notebookId = typeof detail[2] === 'string' ? detail[2] : '{notebook_id}';

            // Find the summary: a long string that is not the title, id, or emoji
            const summaryText = detail
                .filter((v, i) => i !== 0 && i !== 2 && i !== 3)
                .find(v => typeof v === 'string' && v.trim().length >= 80);

            if (typeof summaryText !== 'string') {{
                return JSON.stringify({{ summary: null }});
            }}

            return JSON.stringify({{
                summary: {{
                    notebook_id: notebookId,
                    title,
                    summary: summaryText.trim(),
                    url: 'https://notebooklm.google.com/notebook/' + notebookId,
                }}
            }});
        }})()"#,
        result = serde_json::to_string(&result).unwrap_or("null".to_string()),
        notebook_id = notebook_id,
    );

    let parsed: SummaryResult = client.eval_json(&script).await?;
    match parsed.summary {
        Some(s) => Ok(json!(s)),
        None => Ok(Value::Null),
    }
}
