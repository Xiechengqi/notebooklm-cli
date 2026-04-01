use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio::sync::RwLock;

use crate::agent_browser::client::AgentBrowserClient;
use crate::agent_browser::types::AgentBrowserOptions;
use crate::db::{AccountEntry, Db};
use crate::notebooklm::rpc;

/// Probe a single CDP port: navigate to NotebookLM home, detect logged-in user.
async fn probe_port(binary: &str, cdp_port: &str, timeout_secs: u64) -> Option<AccountEntry> {
    let client = AgentBrowserClient::new(AgentBrowserOptions {
        binary: binary.to_string(),
        cdp_port: cdp_port.to_string(),
        session_name: format!("discovery-{cdp_port}"),
        timeout_secs,
    });

    client.open(rpc::NOTEBOOKLM_HOME_URL).await.ok()?;
    client.wait_ms(3000).await.ok()?;

    let state = rpc::get_page_state(&client).await.ok()?;

    if state.login_required || state.hostname != rpc::NOTEBOOKLM_DOMAIN {
        return None;
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    Some(AccountEntry {
        cdp_port: cdp_port.to_string(),
        email: state.title.clone(),
        display_name: String::new(),
        online: true,
        last_checked: now,
    })
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Run discovery for a list of ports.
pub async fn discover(
    db: &Db,
    binary: &str,
    cdp_ports: &[String],
    timeout_secs: u64,
    skip_cached: bool,
) {
    for port in cdp_ports {
        if skip_cached {
            if let Ok(Some(entry)) = db.get_account(port) {
                if entry.online && !entry.email.is_empty() {
                    continue;
                }
            }
        }

        match probe_port(binary, port, timeout_secs).await {
            Some(entry) => {
                if let Err(e) = db.upsert_account(&entry) {
                    eprintln!("[discovery] upsert_account({port}): {e}");
                }
            }
            None => {
                if let Err(e) = db.ensure_port(port) {
                    eprintln!("[discovery] ensure_port({port}): {e}");
                }
                if let Err(e) = db.set_offline(port, now_secs()) {
                    eprintln!("[discovery] set_offline({port}): {e}");
                }
            }
        }
    }
}

/// Spawn a background task that runs full discovery every hour.
pub fn spawn_periodic(
    db: Db,
    binary: String,
    cdp_ports: Arc<RwLock<Vec<String>>>,
    timeout_secs: u64,
) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3600));
        loop {
            interval.tick().await;
            let ports = cdp_ports.read().await.clone();
            discover(&db, &binary, &ports, timeout_secs, false).await;
        }
    });
}
