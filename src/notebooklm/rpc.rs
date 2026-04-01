use serde::Deserialize;

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::{AppError, AppResult};

pub const NOTEBOOKLM_DOMAIN: &str = "notebooklm.google.com";
pub const NOTEBOOKLM_HOME_URL: &str = "https://notebooklm.google.com/";

pub const RPC_LIST: &str = "wXbhsf";
pub const RPC_NOTEBOOK_DETAIL: &str = "rLM1Ne";
pub const RPC_HISTORY_THREADS: &str = "hPTbtc";
pub const RPC_HISTORY_DETAIL: &str = "khqZz";
pub const RPC_SOURCE_FULLTEXT: &str = "hizoJc";
pub const RPC_SOURCE_GUIDE: &str = "tr032e";

#[derive(Debug, Deserialize)]
pub struct PageState {
    pub url: String,
    pub title: String,
    pub hostname: String,
    pub kind: String,
    #[serde(rename = "notebookId")]
    pub notebook_id: String,
    #[serde(rename = "loginRequired")]
    pub login_required: bool,
    #[serde(rename = "notebookCount")]
    pub notebook_count: u32,
    #[serde(rename = "googleAccount")]
    pub google_account: String,
}

/// Navigate to a notebook page, ensuring we're on the right URL.
pub async fn ensure_notebook_page(
    client: &AgentBrowserClient,
    notebook_id: &str,
) -> AppResult<()> {
    let url = format!("https://{NOTEBOOKLM_DOMAIN}/notebook/{notebook_id}");
    client.open(&url).await?;
    client.wait_ms(3000).await?;
    Ok(())
}

/// Navigate to the NotebookLM home page.
pub async fn ensure_home_page(client: &AgentBrowserClient) -> AppResult<()> {
    client.open(NOTEBOOKLM_HOME_URL).await?;
    client.wait_ms(3000).await?;
    Ok(())
}

/// Detect page state via eval.
pub async fn get_page_state(client: &AgentBrowserClient) -> AppResult<PageState> {
    let script = r#"(async () => {
        const url = window.location.href;
        const title = document.title || '';
        const hostname = window.location.hostname || '';
        const notebookMatch = url.match(/\/notebook\/([^/?#]+)/);
        const notebookId = notebookMatch ? notebookMatch[1] : '';
        const kind = notebookId
            ? 'notebook'
            : (hostname === 'notebooklm.google.com' ? 'home' : 'unknown');

        const textNodes = Array.from(document.querySelectorAll('a, button, [role="button"], h1, h2'))
            .map(node => (node.textContent || '').trim().toLowerCase())
            .filter(Boolean);
        let loginRequired = textNodes.some(text =>
            text.includes('sign in') ||
            text.includes('log in')
        );

        // Check if page auth tokens exist to override heuristic
        if (loginRequired && hostname === 'notebooklm.google.com') {
            const wiz = window.WIZ_global_data || {};
            const html = document.documentElement.innerHTML;
            const csrf = (typeof wiz.SNlM0e === 'string' && wiz.SNlM0e) || (html.match(/"SNlM0e":"([^"]+)"/) || [])[1] || '';
            const sid = (typeof wiz.FdrFJe === 'string' && wiz.FdrFJe) || (html.match(/"FdrFJe":"([^"]+)"/) || [])[1] || '';
            if (csrf && sid) loginRequired = false;
        }

        const notebookCount = Array.from(document.querySelectorAll('a[href*="/notebook/"]'))
            .map(node => node instanceof HTMLAnchorElement ? node.href : '')
            .filter(Boolean)
            .reduce((count, href, index, list) => list.indexOf(href) === index ? count + 1 : count, 0);

        const emailRegex = /[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}/ig;
        const candidates = [
            title,
            document.body?.innerText || '',
            ...Array.from(document.querySelectorAll('[aria-label], [title]')).flatMap(node => [
                node.getAttribute('aria-label') || '',
                node.getAttribute('title') || '',
            ]),
        ];
        const googleAccount = candidates
            .flatMap(text => (text.match(emailRegex) || []).map(value => value.trim()))
            .find(Boolean) || '';

        return JSON.stringify({ url, title, hostname, kind, notebookId, loginRequired, notebookCount, googleAccount });
    })()"#;

    client.eval_json::<PageState>(script).await
}

/// Get the notebook ID from params or detect from current page.
pub async fn resolve_notebook_id(
    client: &AgentBrowserClient,
    params: &serde_json::Value,
) -> AppResult<String> {
    if let Some(id) = params.get("notebook_id").and_then(|v| v.as_str()) {
        if !id.is_empty() {
            return Ok(id.to_string());
        }
    }

    let state = get_page_state(client).await?;
    if state.login_required {
        return Err(AppError::NotebooklmLoginRequired);
    }
    if state.kind == "notebook" && !state.notebook_id.is_empty() {
        return Ok(state.notebook_id);
    }

    Err(AppError::InvalidParams(
        "notebook_id is required (not currently on a notebook page)".to_string(),
    ))
}

/// Build and execute an RPC call via in-page fetch, returning the raw JSON result.
/// This is the core RPC function that mirrors the TypeScript `callNotebooklmRpc`.
pub fn build_rpc_eval_script(rpc_id: &str, params_json: &str) -> String {
    format!(
        r#"(async () => {{
            const wiz = window.WIZ_global_data || {{}};
            const html = document.documentElement.innerHTML;
            const csrfToken = (typeof wiz.SNlM0e === 'string' && wiz.SNlM0e) || (html.match(/"SNlM0e":"([^"]+)"/) || [])[1] || '';
            const sessionId = (typeof wiz.FdrFJe === 'string' && wiz.FdrFJe) || (html.match(/"FdrFJe":"([^"]+)"/) || [])[1] || '';

            if (!csrfToken || !sessionId) {{
                return JSON.stringify({{ error: 'NOTEBOOKLM_LOGIN_REQUIRED', message: 'No auth tokens found' }});
            }}

            const rpcId = {rpc_id};
            const params = {params_json};
            const rpcRequest = [[[rpcId, JSON.stringify(params), null, 'generic']]];
            const body = 'f.req=' + encodeURIComponent(JSON.stringify(rpcRequest)) + '&at=' + encodeURIComponent(csrfToken) + '&';
            const url = 'https://notebooklm.google.com/_/LabsTailwindUi/data/batchexecute'
                + '?rpcids=' + rpcId
                + '&source-path=' + encodeURIComponent(location.pathname || '/')
                + '&hl=en'
                + '&f.sid=' + encodeURIComponent(sessionId)
                + '&rt=c';

            const resp = await fetch(url, {{
                method: 'POST',
                headers: {{ 'Content-Type': 'application/x-www-form-urlencoded;charset=UTF-8' }},
                body: body,
                credentials: 'include',
            }});

            if (resp.status === 401 || resp.status === 403) {{
                return JSON.stringify({{ error: 'NOTEBOOKLM_LOGIN_REQUIRED', message: 'Auth error ' + resp.status }});
            }}
            if (!resp.ok) {{
                return JSON.stringify({{ error: 'NOTEBOOKLM_RPC_FAILED', message: 'HTTP ' + resp.status }});
            }}

            const rawBody = await resp.text();
            // Strip anti-XSSI prefix
            const cleaned = rawBody.replace(/^\)\]\}}'\r?\n/, '').trim();
            if (!cleaned) return JSON.stringify({{ error: 'NOTEBOOKLM_RPC_FAILED', message: 'Empty response' }});

            // Parse chunked response
            const lines = cleaned.split('\n');
            const chunks = [];
            for (let i = 0; i < lines.length; i++) {{
                const line = lines[i].trim();
                if (!line) continue;
                if (/^\d+$/.test(line)) {{
                    const nextLine = lines[i + 1];
                    if (nextLine) {{
                        try {{ chunks.push(JSON.parse(nextLine)); }} catch {{}}
                        i++;
                    }}
                    continue;
                }}
                if (line.startsWith('[')) {{
                    try {{ chunks.push(JSON.parse(line)); }} catch {{}}
                }}
            }}

            // Extract RPC result
            for (const chunk of chunks) {{
                if (!Array.isArray(chunk)) continue;
                const items = Array.isArray(chunk[0]) ? chunk : [chunk];
                for (const item of items) {{
                    if (!Array.isArray(item) || item.length < 1) continue;
                    if (item[0] === 'er') {{
                        const errorCode = typeof item[2] === 'number' ? item[2] : (typeof item[5] === 'number' ? item[5] : null);
                        if (errorCode === 401 || errorCode === 403) {{
                            return JSON.stringify({{ error: 'NOTEBOOKLM_LOGIN_REQUIRED', message: 'RPC auth error ' + errorCode }});
                        }}
                        return JSON.stringify({{ error: 'NOTEBOOKLM_RPC_FAILED', message: 'RPC error' + (errorCode ? ' code=' + errorCode : '') }});
                    }}
                    if (item[0] === 'wrb.fr' && item[1] === rpcId) {{
                        const payload = item[2];
                        if (typeof payload === 'string') {{
                            try {{ return JSON.stringify({{ result: JSON.parse(payload) }}); }} catch {{}}
                            return JSON.stringify({{ result: payload }});
                        }}
                        return JSON.stringify({{ result: payload }});
                    }}
                }}
            }}

            return JSON.stringify({{ result: null }});
        }})()"#,
        rpc_id = serde_json::to_string(rpc_id).unwrap_or_default(),
        params_json = params_json,
    )
}

#[derive(Debug, Deserialize)]
pub struct RpcResponse {
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub message: Option<String>,
}

/// Execute an RPC call and return the result value.
pub async fn call_rpc(
    client: &AgentBrowserClient,
    rpc_id: &str,
    params_json: &str,
) -> AppResult<serde_json::Value> {
    let script = build_rpc_eval_script(rpc_id, params_json);
    let resp: RpcResponse = client.eval_json(&script).await?;

    if let Some(error) = resp.error {
        if error == "NOTEBOOKLM_LOGIN_REQUIRED" {
            return Err(AppError::NotebooklmLoginRequired);
        }
        let msg = resp.message.unwrap_or(error);
        return Err(AppError::NotebooklmRpcFailed(msg));
    }

    Ok(resp.result.unwrap_or(serde_json::Value::Null))
}
