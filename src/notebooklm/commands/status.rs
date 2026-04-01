use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::errors::AppResult;
use crate::notebooklm::rpc;

pub async fn execute(client: &AgentBrowserClient, _params: &Value) -> AppResult<Value> {
    rpc::ensure_home_page(client).await?;
    let state = rpc::get_page_state(client).await?;

    Ok(json!({
        "url": state.url,
        "title": state.title,
        "hostname": state.hostname,
        "kind": state.kind,
        "notebook_id": state.notebook_id,
        "login_required": state.login_required,
        "notebook_count": state.notebook_count,
    }))
}
