use serde_json::{Value, json};

use crate::agent_browser::client::AgentBrowserClient;
use crate::agent_browser::types::AgentBrowserOptions;
use crate::commands::registry::CommandRegistry;
use crate::config::AppConfig;
use crate::errors::{AppError, AppResult};
use crate::notebooklm::commands::{
    get, history, list, note_create, note_get, note_list, source_add_youtube, source_fulltext,
    source_get, source_guide, source_list, status, summary,
};

#[derive(Clone)]
pub struct CommandExecutor {
    registry: CommandRegistry,
}

impl CommandExecutor {
    pub fn new(registry: CommandRegistry) -> Self {
        Self { registry }
    }

    pub async fn execute(
        &self,
        command_name: &str,
        params: Value,
        config: &AppConfig,
        managed_ports: &[String],
    ) -> AppResult<Value> {
        let command = self
            .registry
            .get(command_name)
            .ok_or_else(|| AppError::CommandNotFound(command_name.to_string()))?;

        let cdp_port = params
            .get("cdp_port")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(ToString::to_string)
            .ok_or_else(|| {
                AppError::InvalidParams(
                    "cdp_port is required. Call notebooklm_accounts first to discover available accounts."
                        .to_string(),
                )
            })?;

        if !managed_ports.is_empty() && !managed_ports.contains(&cdp_port) {
            let available = managed_ports.join(", ");
            return Err(AppError::InvalidParams(format!(
                "cdp_port {cdp_port} is not managed. Available: {available}"
            )));
        }

        let client = AgentBrowserClient::new(AgentBrowserOptions {
            binary: config.agent_browser.binary.clone(),
            cdp_port,
            session_name: config.agent_browser.session_name.clone(),
            timeout_secs: config.agent_browser.timeout_secs,
        });

        match command.name {
            "status" => status::execute(&client, &params).await,
            "list" => list::execute(&client, &params).await,
            "get" => get::execute(&client, &params).await,
            "summary" => summary::execute(&client, &params).await,
            "source_list" => source_list::execute(&client, &params).await,
            "source_get" => source_get::execute(&client, &params).await,
            "source_fulltext" => source_fulltext::execute(&client, &params).await,
            "source_guide" => source_guide::execute(&client, &params).await,
            "history" => history::execute(&client, &params).await,
            "note_list" => note_list::execute(&client, &params).await,
            "note_get" => note_get::execute(&client, &params).await,
            "note_create" => note_create::execute(&client, &params).await,
            "source_add_youtube" => source_add_youtube::execute(&client, &params).await,
            _ => Ok(json!({
                "status": "planned",
                "message": format!("Command `{}` is registered but not implemented yet", command.name),
                "params": params,
            })),
        }
    }
}
