use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::{AppError, AppResult};
use crate::notebooklm::rpc;

pub async fn execute(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let notebook_id = rpc::resolve_notebook_id(client, params).await?;
    rpc::ensure_notebook_page(client, &notebook_id).await?;

    let title = params
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    let content = params
        .get("content")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    let title_json = Value::String(title.clone()).to_string();
    let content_json = Value::String(content.clone()).to_string();

    let script = format!(
        r#"(async (title, content) => {{
            const sleep = (ms) => new Promise(resolve => setTimeout(resolve, ms));
            const buttons = () => Array.from(document.querySelectorAll('button'));
            const match = (el, phrases) => {{
                const hay = [
                    el.textContent || '',
                    el.getAttribute('aria-label') || '',
                    el.getAttribute('title') || '',
                    String(el.className || ''),
                ].join(' ').toLowerCase();
                return phrases.some(phrase => hay.includes(phrase));
            }};
            const setValue = (el, value) => {{
                const proto = Object.getPrototypeOf(el);
                const desc = Object.getOwnPropertyDescriptor(proto, 'value');
                if (desc && typeof desc.set === 'function') desc.set.call(el, value);
                else el.value = value;
                el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                el.dispatchEvent(new Event('change', {{ bubbles: true }}));
            }};

            const sourceClose = document.querySelector('button.close-button');
            if (sourceClose instanceof HTMLElement) {{
                sourceClose.click();
                await sleep(300);
            }}

            const noteClose = buttons().find(el => match(el, ['关闭笔记视图', 'close note view']));
            if (noteClose instanceof HTMLElement) {{
                noteClose.click();
                await sleep(300);
            }}

            const add = document.querySelector('button.add-note-button')
                || buttons().find(el => match(el, ['添加笔记', 'add note']));
            if (!(add instanceof HTMLElement)) {{
                return JSON.stringify({{ ok: false, error: 'add-note button not found' }});
            }}

            add.click();
            await sleep(1200);

            const titleInput = document.querySelector('input.note-header__editable-title');
            const editor = document.querySelector('.ql-editor, .note-editor [contenteditable=true], .note-editor textarea');
            if (!(titleInput instanceof HTMLInputElement) || !editor) {{
                return JSON.stringify({{ ok: false, error: 'note editor not found' }});
            }}

            if (title) {{
                setValue(titleInput, title);
                titleInput.blur();
                await sleep(200);
            }}

            if (content) {{
                if (editor instanceof HTMLTextAreaElement || editor instanceof HTMLInputElement) {{
                    setValue(editor, content);
                    editor.blur();
                }} else {{
                    editor.focus();
                    editor.textContent = content;
                    editor.dispatchEvent(new InputEvent('input', {{ bubbles: true, inputType: 'insertText', data: content }}));
                    editor.blur();
                }}
            }}

            await sleep(1500);

            return JSON.stringify({{
                ok: true,
                title: (titleInput.value || titleInput.textContent || '').trim(),
                content: (editor.innerText || editor.textContent || editor.value || '').trim(),
            }});
        }})({title}, {content})"#,
        title = title_json,
        content = content_json,
    );

    let result: Value = client.eval_json(&script).await?;
    if result.get("ok").and_then(Value::as_bool) != Some(true) {
        let message = result
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or("failed to create note")
            .to_string();
        return Err(AppError::BrowserExecutionFailed(message));
    }

    let final_title = result
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let final_content = result
        .get("content")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    if final_title.is_empty() {
        return Err(AppError::BrowserExecutionFailed(
            "note editor did not produce a title".to_string(),
        ));
    }

    Ok(json!({
        "notebook_id": notebook_id,
        "title": final_title,
        "content": final_content,
        "url": format!("https://{}/notebook/{}", rpc::NOTEBOOKLM_DOMAIN, notebook_id),
        "created": true,
    }))
}
