use serde::Deserialize;
use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::AppResult;
use crate::notebooklm::rpc;

#[derive(Debug, Deserialize)]
struct NoteListResult {
    notes: Vec<NoteEntry>,
}

#[derive(Debug, Deserialize)]
struct NoteEntry {
    title: String,
    #[serde(default)]
    created_at: Option<String>,
}

pub async fn execute(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let notebook_id = rpc::resolve_notebook_id(client, params).await?;
    rpc::ensure_notebook_page(client, &notebook_id).await?;

    // Extract notes from Studio panel DOM
    let script = format!(
        r#"(() => {{
            const notebookId = '{notebook_id}';
            const rawNotes = Array.from(document.querySelectorAll('artifact-library-note')).map(node => {{
                const titleNode = node.querySelector('.artifact-title');
                const title = (titleNode?.textContent || '').trim();
                const text = (node.innerText || node.textContent || '').replace(/\s+/g, ' ').trim();
                return {{ title, text }};
            }});

            const notes = rawNotes
                .filter(row => row.title)
                .map(row => {{
                    let text = row.text
                        .replace(/\bsticky_note_2\b/g, ' ')
                        .replace(/\bmore_vert\b/g, ' ')
                        .replace(/\s+/g, ' ')
                        .trim();
                    const suffix = text.startsWith(row.title)
                        ? text.slice(row.title.length).trim()
                        : text.replace(row.title, '').trim();
                    return {{
                        title: row.title,
                        created_at: suffix || null,
                    }};
                }});

            return JSON.stringify({{ notes }});
        }})()"#,
        notebook_id = notebook_id,
    );

    let parsed: NoteListResult = client.eval_json(&script).await?;
    let url = format!(
        "https://{}/notebook/{}",
        rpc::NOTEBOOKLM_DOMAIN,
        notebook_id
    );

    let result: Vec<Value> = parsed
        .notes
        .into_iter()
        .map(|note| {
            json!({
                "notebook_id": notebook_id,
                "title": note.title,
                "created_at": note.created_at,
                "url": url,
            })
        })
        .collect();

    Ok(json!(result))
}
