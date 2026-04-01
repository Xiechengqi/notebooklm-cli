use serde::Deserialize;
use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::{AppError, AppResult};
use crate::notebooklm::rpc;

#[derive(Debug, Deserialize)]
struct NoteDetailResult {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    content: Option<String>,
}

pub async fn execute(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let query = params
        .get("note")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();

    if query.is_empty() {
        return Err(AppError::InvalidParams("note is required".to_string()));
    }

    let notebook_id = rpc::resolve_notebook_id(client, params).await?;
    rpc::ensure_notebook_page(client, &notebook_id).await?;

    // Try to read the currently visible note from the editor
    let script = r#"(() => {
        const normalizeText = (value) => (value || '').replace(/\u00a0/g, ' ').replace(/\r\n/g, '\n').trim();
        const titleNode = document.querySelector('.note-header__editable-title');
        const title = titleNode instanceof HTMLInputElement || titleNode instanceof HTMLTextAreaElement
            ? titleNode.value
            : (titleNode?.textContent || '');
        const editor = document.querySelector('.note-editor .ql-editor, .note-editor [contenteditable="true"], .note-editor textarea');
        let content = '';
        if (editor instanceof HTMLTextAreaElement || editor instanceof HTMLInputElement) {
            content = editor.value || '';
        } else if (editor) {
            content = editor.innerText || editor.textContent || '';
        }
        return JSON.stringify({
            title: normalizeText(title),
            content: normalizeText(content),
        });
    })()"#;

    let detail: NoteDetailResult = client.eval_json(script).await?;
    let title = detail.title.unwrap_or_default();
    let content = detail.content.unwrap_or_default();

    if title.is_empty() {
        return Err(AppError::InvalidParams(format!(
            "note '{}' not found — open the note in the Studio panel first",
            query
        )));
    }

    // Check if the visible note matches the query
    let needle = query.to_lowercase();
    if !title.to_lowercase().contains(&needle) {
        return Err(AppError::InvalidParams(format!(
            "visible note '{}' does not match query '{}' — click the note in the Studio panel first",
            title, query
        )));
    }

    let url = format!(
        "https://{}/notebook/{}",
        rpc::NOTEBOOKLM_DOMAIN,
        notebook_id
    );

    Ok(json!({
        "notebook_id": notebook_id,
        "title": title,
        "content": content,
        "url": url,
    }))
}
