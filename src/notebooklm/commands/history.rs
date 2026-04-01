use serde::Deserialize;
use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::AppResult;
use crate::notebooklm::rpc;

#[derive(Debug, Deserialize)]
struct ThreadIdsResult {
    thread_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ThreadDetailResult {
    item_count: u32,
    #[serde(default)]
    preview: Option<String>,
}

pub async fn execute(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let notebook_id = rpc::resolve_notebook_id(client, params).await?;
    rpc::ensure_notebook_page(client, &notebook_id).await?;

    // Get thread IDs
    let threads_params = format!(r#"[[], null, "{}", 20]"#, notebook_id);
    let threads_result =
        rpc::call_rpc(client, rpc::RPC_HISTORY_THREADS, &threads_params).await?;

    // Extract thread IDs (UUIDs)
    let ids_script = format!(
        r#"(() => {{
            const result = {result};
            const uuidRe = /^[0-9a-f]{{8}}-[0-9a-f]{{4}}-[0-9a-f]{{4}}-[0-9a-f]{{4}}-[0-9a-f]{{12}}$/i;
            const ids = [];
            const seen = new Set();

            function collect(v) {{
                if (typeof v === 'string') {{
                    const t = v.trim();
                    if (uuidRe.test(t) && !seen.has(t)) {{ seen.add(t); ids.push(t); }}
                    return;
                }}
                if (Array.isArray(v)) for (const item of v) collect(item);
            }}

            collect(result);
            return JSON.stringify({{ thread_ids: ids }});
        }})()"#,
        result = serde_json::to_string(&threads_result).unwrap_or("null".to_string()),
    );

    let ids_parsed: ThreadIdsResult = client.eval_json(&ids_script).await?;
    if ids_parsed.thread_ids.is_empty() {
        return Ok(json!([]));
    }

    // Fetch details for each thread
    let mut rows = Vec::new();
    for thread_id in &ids_parsed.thread_ids {
        let detail_params = format!(r#"[[], null, null, "{}", 20]"#, thread_id);
        let detail_result =
            rpc::call_rpc(client, rpc::RPC_HISTORY_DETAIL, &detail_params).await?;

        let detail_script = format!(
            r#"(() => {{
                const result = {result};
                const item_count = Array.isArray(result) ? result.length : 0;

                // Extract preview: first non-UUID, non-trivial string
                const uuidRe = /^[0-9a-f]{{8}}-[0-9a-f]{{4}}-[0-9a-f]{{4}}-[0-9a-f]{{4}}-[0-9a-f]{{12}}$/i;
                function collectStrings(v, results) {{
                    if (typeof v === 'string') {{
                        const t = v.replace(/\s+/g, ' ').trim();
                        if (t && !uuidRe.test(t) && !/^\d+$/.test(t) && !/^(null|undefined)$/i.test(t)) {{
                            results.push(t);
                        }}
                        return results;
                    }}
                    if (Array.isArray(v)) for (const item of v) collectStrings(item, results);
                    return results;
                }}

                const strings = collectStrings(result, []);
                return JSON.stringify({{ item_count, preview: strings.length > 0 ? strings[0] : null }});
            }})()"#,
            result = serde_json::to_string(&detail_result).unwrap_or("null".to_string()),
        );

        let detail: ThreadDetailResult = client.eval_json(&detail_script).await?;
        rows.push(json!({
            "thread_id": thread_id,
            "notebook_id": notebook_id,
            "item_count": detail.item_count,
            "preview": detail.preview,
            "url": format!("https://{}/notebook/{}", rpc::NOTEBOOKLM_DOMAIN, notebook_id),
        }));
    }

    Ok(json!(rows))
}
