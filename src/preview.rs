use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use serde::Serialize;

use crate::agent_browser::client::AgentBrowserClient;
use crate::agent_browser::types::AgentBrowserOptions;
use crate::db::NewPreviewNoteEntry;
use crate::errors::{AppError, AppResult};
use crate::notebooklm::rpc;
use crate::server::AppState;

const PREVIEW_SYNC_INTERVAL_SECS: u64 = 300;

#[derive(Debug, Clone, Serialize)]
pub struct PreviewSyncResult {
    pub added: u64,
    pub skipped: u64,
    pub failed_ports: u64,
}

#[derive(Debug, Deserialize)]
struct NotebookListResult {
    notebooks: Vec<NotebookEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct NotebookEntry {
    id: String,
    title: String,
}

#[derive(Debug, Deserialize)]
struct NoteListResult {
    notes: Vec<NoteEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct NoteEntry {
    title: String,
}

#[derive(Debug, Deserialize)]
struct NoteDetailResult {
    title: String,
    content: String,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn preview_text(content: &str, max_chars: usize) -> String {
    let normalized = content.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut preview = String::new();
    for ch in normalized.chars().take(max_chars) {
        preview.push(ch);
    }
    preview
}

fn note_key(title: &str) -> String {
    title.trim().to_lowercase()
}

async fn list_notebooks(client: &AgentBrowserClient) -> AppResult<Vec<NotebookEntry>> {
    rpc::ensure_home_page(client).await?;
    let result = rpc::call_rpc(client, rpc::RPC_LIST, "[null, 1, null, [2]]").await?;
    let script = format!(
        r#"(() => {{
            const result = {result};
            if (!Array.isArray(result) || result.length === 0) return JSON.stringify({{ notebooks: [] }});
            const rawNotebooks = Array.isArray(result[0]) ? result[0] : result;
            const notebooks = (Array.isArray(rawNotebooks) ? rawNotebooks : [])
                .filter(item => Array.isArray(item))
                .map(item => {{
                    const id = typeof item[2] === 'string' ? item[2] : '';
                    let title = typeof item[0] === 'string' ? item[0].replace(/^thought\s*\n/, '') : '';
                    title = title.replace(/\s+/g, ' ').trim() || 'Untitled Notebook';
                    return {{ id, title }};
                }})
                .filter(item => item.id);
            return JSON.stringify({{ notebooks }});
        }})()"#,
        result = serde_json::to_string(&result).unwrap_or_else(|_| "null".to_string()),
    );
    let parsed: NotebookListResult = client.eval_json(&script).await?;
    Ok(parsed.notebooks)
}

async fn list_notes(client: &AgentBrowserClient, notebook_id: &str) -> AppResult<Vec<NoteEntry>> {
    rpc::ensure_notebook_page(client, notebook_id).await?;
    let parsed: NoteListResult = client
        .eval_json(
            r#"(() => {
                const notes = Array.from(document.querySelectorAll('artifact-library-note'))
                    .map(node => {
                        const titleNode = node.querySelector('.artifact-title');
                        const title = (titleNode?.textContent || '').replace(/\s+/g, ' ').trim();
                        return { title };
                    })
                    .filter(note => note.title);
                return JSON.stringify({ notes });
            })()"#,
        )
        .await?;
    Ok(parsed.notes)
}

async fn open_note(client: &AgentBrowserClient, title: &str) -> AppResult<()> {
    let title_json =
        serde_json::to_string(title).map_err(|err| AppError::Internal(err.to_string()))?;
    let script = format!(
        r#"(async (needle) => {{
            const normalize = (value) => (value || '').replace(/\s+/g, ' ').trim().toLowerCase();
            const target = normalize(needle);
            const rows = Array.from(document.querySelectorAll('artifact-library-note'));
            const row = rows.find(node => {{
                const titleNode = node.querySelector('.artifact-title');
                return normalize(titleNode?.textContent || '') === target;
            }});
            if (!row) return JSON.stringify({{ ok: false, error: 'note not found' }});
            row.scrollIntoView({{ block: 'center' }});
            row.click();
            await new Promise(resolve => setTimeout(resolve, 1200));
            return JSON.stringify({{ ok: true }});
        }})({title})"#,
        title = title_json,
    );
    let result: serde_json::Value = client.eval_json(&script).await?;
    if result.get("ok").and_then(serde_json::Value::as_bool) == Some(true) {
        return Ok(());
    }
    Err(AppError::BrowserExecutionFailed(
        result
            .get("error")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("failed to open note")
            .to_string(),
    ))
}

async fn read_visible_note(client: &AgentBrowserClient) -> AppResult<NoteDetailResult> {
    client
        .eval_json(
            r#"(() => {
                const normalize = (value) => (value || '').replace(/\u00a0/g, ' ').replace(/\r\n/g, '\n').trim();
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
                    title: normalize(title),
                    content: normalize(content),
                });
            })()"#,
        )
        .await
}

async fn sync_port(state: &Arc<AppState>, cdp_port: &str) -> AppResult<PreviewSyncResult> {
    let runtime = state.runtime.read().await;
    let client = AgentBrowserClient::new(AgentBrowserOptions {
        binary: runtime.config.agent_browser.binary.clone(),
        cdp_port: cdp_port.to_string(),
        session_name: format!("preview-sync-{cdp_port}"),
        timeout_secs: runtime.config.agent_browser.timeout_secs,
    });
    drop(runtime);

    client.open(rpc::NOTEBOOKLM_HOME_URL).await?;
    client.wait_ms(2500).await?;
    let page_state = rpc::get_page_state(&client).await?;
    if page_state.login_required {
        return Err(AppError::NotebooklmLoginRequired);
    }

    let notebooks = list_notebooks(&client).await?;
    let mut added = 0_u64;
    let mut skipped = 0_u64;
    let google_account = page_state.google_account;

    for notebook in notebooks {
        let notes = list_notes(&client, &notebook.id).await?;
        for note in notes {
            let key = note_key(&note.title);
            if state
                .db
                .preview_note_exists(&google_account, &notebook.id, &key)?
            {
                skipped += 1;
                continue;
            }

            open_note(&client, &note.title).await?;
            client.wait_ms(1000).await?;
            let detail = read_visible_note(&client).await?;
            if detail.title.trim().is_empty() {
                continue;
            }

            let timestamp = now_secs();
            state.db.insert_preview_note(&NewPreviewNoteEntry {
                cdp_port: cdp_port.to_string(),
                google_account: google_account.clone(),
                notebook_id: notebook.id.clone(),
                notebook_title: notebook.title.clone(),
                note_key: key,
                note_title: detail.title,
                content_preview: preview_text(&detail.content, 240),
                content: detail.content,
                fetched_at: timestamp,
                created_at: timestamp,
            })?;
            added += 1;
        }
    }

    Ok(PreviewSyncResult {
        added,
        skipped,
        failed_ports: 0,
    })
}

pub async fn run_sync(state: Arc<AppState>) -> AppResult<PreviewSyncResult> {
    {
        let mut status = state.preview_status.write().await;
        if status.running {
            return Ok(PreviewSyncResult {
                added: 0,
                skipped: 0,
                failed_ports: 0,
            });
        }
        status.running = true;
        status.last_started_at = Some(now_secs());
        status.last_error = None;
        status.last_added = 0;
        status.last_skipped = 0;
        status.last_failed_ports = 0;
    }

    let ports = state.cdp_ports.read().await.clone();
    let mut total_added = 0_u64;
    let mut total_skipped = 0_u64;
    let mut failed_ports = 0_u64;
    let mut last_error = None;

    for port in ports {
        match sync_port(&state, &port).await {
            Ok(result) => {
                total_added += result.added;
                total_skipped += result.skipped;
            }
            Err(err) => {
                failed_ports += 1;
                last_error = Some(format!("{port}: {err}"));
                eprintln!("[preview] sync_port({port}): {err}");
            }
        }
    }

    {
        let mut status = state.preview_status.write().await;
        status.running = false;
        status.last_finished_at = Some(now_secs());
        status.last_added = total_added;
        status.last_skipped = total_skipped;
        status.last_failed_ports = failed_ports;
        status.last_error = last_error.clone();
    }

    Ok(PreviewSyncResult {
        added: total_added,
        skipped: total_skipped,
        failed_ports,
    })
}

pub fn spawn_periodic(state: Arc<AppState>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(PREVIEW_SYNC_INTERVAL_SECS));
        loop {
            interval.tick().await;
            let _ = run_sync(state.clone()).await;
        }
    });
}
