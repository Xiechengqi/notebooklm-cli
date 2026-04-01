use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::{AppError, AppResult};
use crate::notebooklm::commands::source_list;
use crate::notebooklm::rpc;

fn parse_source_count(value: &Value) -> usize {
    value.as_array().map(|rows| rows.len()).unwrap_or(0)
}

async fn click_create_notebook(client: &AgentBrowserClient) -> AppResult<()> {
    let _ = client.run(&["wait", "--text", "新建笔记本"]).await;
    let _ = client.run(&["wait", "--text", "Create notebook"]).await;
    if client.run(&["wait", "button.create-notebook-button"]).await.is_ok()
        && client.run(&["click", "button.create-notebook-button"]).await.is_ok()
    {
        return Ok(());
    }
    if client
        .run(&["find", "role", "button", "click", "--name", "新建笔记本"])
        .await
        .is_ok()
    {
        return Ok(());
    }
    if client
        .run(&["find", "role", "button", "click", "--name", "Create notebook"])
        .await
        .is_ok()
    {
        return Ok(());
    }
    if client
        .run(&["find", "first", "button.create-notebook-button", "click"])
        .await
        .is_ok()
    {
        return Ok(());
    }
    click_text_variants(client, &["新建笔记本", "Create notebook"]).await
}

async fn resolve_or_create_notebook_id(
    client: &AgentBrowserClient,
    params: &Value,
) -> AppResult<String> {
    if let Some(id) = params.get("notebook_id").and_then(Value::as_str) {
        if !id.is_empty() {
            return Ok(id.to_string());
        }
    }

    let state = rpc::get_page_state(client).await?;
    if state.login_required {
        return Err(AppError::NotebooklmLoginRequired);
    }
    if state.kind == "notebook"
        && !state.notebook_id.is_empty()
        && state.notebook_id != "creating"
    {
        return Ok(state.notebook_id);
    }
    if state.kind != "home" {
        rpc::ensure_home_page(client).await?;
    }

    click_create_notebook(client).await?;
    for _ in 0..20 {
        client.wait_ms(500).await?;
        let next = rpc::get_page_state(client).await?;
        if next.login_required {
            return Err(AppError::NotebooklmLoginRequired);
        }
        if next.kind == "notebook"
            && !next.notebook_id.is_empty()
            && next.notebook_id != "creating"
        {
            return Ok(next.notebook_id);
        }
    }

    Err(AppError::BrowserExecutionFailed(
        "timed out waiting for a new notebook to open".to_string(),
    ))
}

async fn click_text_variants(client: &AgentBrowserClient, values: &[&str]) -> AppResult<()> {
    for value in values {
        if client.run(&["find", "text", value, "click"]).await.is_ok() {
            return Ok(());
        }
    }
    Err(AppError::BrowserExecutionFailed(format!(
        "none of these text targets were clickable: {}",
        values.join(", ")
    )))
}

async fn fill_url_variants(client: &AgentBrowserClient, url: &str) -> AppResult<()> {
    let attempts: [Vec<&str>; 4] = [
        vec!["find", "label", "输入网址", "fill", url],
        vec!["find", "label", "Enter URL", "fill", url],
        vec!["find", "placeholder", "粘贴任何链接", "fill", url],
        vec!["find", "placeholder", "Paste any link", "fill", url],
    ];

    for args in attempts {
        if client.run(&args).await.is_ok() {
            return Ok(());
        }
    }

    Err(AppError::BrowserExecutionFailed(
        "url input not found".to_string(),
    ))
}

fn ref_by_name(data: &Value, names: &[&str], role: &str) -> Option<String> {
    let refs = data.get("refs")?.as_object()?;
    for (key, value) in refs {
        let value_role = value.get("role").and_then(Value::as_str).unwrap_or("");
        let value_name = value.get("name").and_then(Value::as_str).unwrap_or("");
        if value_role != role {
            continue;
        }
        if names.iter().any(|name| value_name.contains(name)) {
            return Some(format!("@{key}"));
        }
    }
    None
}

pub async fn execute(client: &AgentBrowserClient, params: &Value) -> AppResult<Value> {
    let youtube_url = params
        .get("url")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();

    if youtube_url.is_empty() {
        return Err(AppError::InvalidParams("url is required".to_string()));
    }

    let normalized = youtube_url.to_lowercase();
    if !(normalized.contains("youtube.com/") || normalized.contains("youtu.be/")) {
        return Err(AppError::InvalidParams(
            "url must be a YouTube link".to_string(),
        ));
    }

    let notebook_id = resolve_or_create_notebook_id(client, params).await?;
    rpc::ensure_notebook_page(client, &notebook_id).await?;

    let before = source_list::execute(client, params).await?;
    let before_count = parse_source_count(&before);

    let _ = client.eval_json::<Value>(
        r#"(() => {
            const close = Array.from(document.querySelectorAll('button')).find(el => {
                const label = (el.getAttribute('aria-label') || '').toLowerCase();
                return label.includes('关闭笔记视图') || label.includes('close note view');
            });
            if (close instanceof HTMLElement) close.click();
            return JSON.stringify({ ok: true });
        })()"#,
    ).await;
    client.wait_ms(250).await?;

    let initial_snapshot = client.run(&["snapshot", "-i"]).await?;
    let initial_data = initial_snapshot.data.unwrap_or(Value::Null);

    if let Some(website_ref) = ref_by_name(&initial_data, &["网站", "Website"], "button") {
        client.run(&["click", &website_ref]).await?;
    } else {
        client
            .run(&["find", "first", "button.add-source-button", "click"])
            .await?;
        client.wait_ms(1000).await?;
        if let Some(website_ref) = ref_by_name(
            &client.run(&["snapshot", "-i"]).await?.data.unwrap_or(Value::Null),
            &["网站", "Website"],
            "button",
        ) {
            client.run(&["click", &website_ref]).await?;
        } else {
            click_text_variants(client, &["网站", "Website"]).await?;
        }
    }

    client.wait_ms(1000).await?;
    let website_snapshot = client.run(&["snapshot", "-i"]).await?;
    let website_data = website_snapshot.data.unwrap_or(Value::Null);
    if let Some(url_ref) = ref_by_name(&website_data, &["输入网址", "Enter URL"], "textbox") {
        client.run(&["fill", &url_ref, &youtube_url]).await?;
    } else {
        fill_url_variants(client, &youtube_url).await?;
    }
    client.wait_ms(500).await?;
    let insert_snapshot = client.run(&["snapshot", "-i"]).await?;
    let insert_data = insert_snapshot.data.unwrap_or(Value::Null);
    if let Some(insert_ref) = ref_by_name(&insert_data, &["插入", "Insert"], "button") {
        client.run(&["click", &insert_ref]).await?;
    } else {
        click_text_variants(client, &["插入", "Insert"]).await?;
    }
    client.wait_ms(4500).await?;

    let after = source_list::execute(client, params).await?;
    let after_rows = after.as_array().cloned().unwrap_or_default();
    let after_count = after_rows.len();
    let newest = after_rows.last().cloned().unwrap_or(Value::Null);

    Ok(json!({
        "notebook_id": notebook_id,
        "youtube_url": youtube_url,
        "url": format!("https://{}/notebook/{}", rpc::NOTEBOOKLM_DOMAIN, notebook_id),
        "accepted": true,
        "source_count_before": before_count,
        "source_count_after": after_count,
        "latest_source": newest,
    }))
}
