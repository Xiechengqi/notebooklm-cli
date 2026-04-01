use clap::{Args, Parser, Subcommand};

use crate::commands::executor::CommandExecutor;
use crate::commands::registry::CommandRegistry;
use crate::config;
use crate::errors::{AppError, AppResult};
use crate::manifest::build_manifest;
use crate::server;

#[derive(Debug, Parser)]
#[command(
    name = "notebooklm-cli",
    version,
    about = "NotebookLM native CLI and local control plane"
)]
pub struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Debug, Subcommand)]
enum CliCommand {
    Describe(DescribeArgs),
    Execute(ExecuteArgs),
    Serve(ServeArgs),
}

#[derive(Debug, Args)]
struct DescribeArgs {
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct ExecuteArgs {
    command: String,
    #[arg(long, default_value = "{}")]
    params: String,
    #[arg(long)]
    cdp_port: Option<String>,
}

#[derive(Debug, Args)]
struct ServeArgs {
    #[arg(long)]
    host: Option<String>,
    #[arg(long)]
    port: Option<u16>,
    #[arg(long, value_delimiter = ',')]
    cdp_ports: Vec<String>,
    #[arg(long)]
    password: Option<String>,
}

impl Cli {
    pub async fn run(self) -> AppResult<()> {
        match self.command {
            CliCommand::Describe(args) => {
                let (config, path, _) = config::load_or_init().await?;
                let manifest = build_manifest(
                    path.display().to_string(),
                    config.server.host.clone(),
                    config.server.port,
                );
                if args.json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&manifest)
                            .map_err(|err| AppError::Internal(err.to_string()))?
                    );
                } else {
                    println!("notebooklm-cli");
                    println!("commands: {}", manifest.commands.len());
                    println!("mcp tools: {}", manifest.mcp_tools.len());
                    println!("skills: {}", manifest.skills.len());
                    println!("server: {}", manifest.server_defaults.base_url);
                }
                Ok(())
            }
            CliCommand::Execute(args) => {
                let (config, _, _) = config::load_or_init().await?;
                let mut params: serde_json::Value = serde_json::from_str(&args.params)
                    .map_err(|err| AppError::InvalidParams(err.to_string()))?;
                let cdp_port = args.cdp_port.unwrap_or_default();
                if let Some(obj) = params.as_object_mut() {
                    obj.insert(
                        "cdp_port".to_string(),
                        serde_json::Value::String(cdp_port.clone()),
                    );
                }
                let managed_ports = if cdp_port.is_empty() {
                    vec![]
                } else {
                    vec![cdp_port]
                };
                let executor = CommandExecutor::new(CommandRegistry::new());
                let result = executor
                    .execute(&args.command, params, &config, &managed_ports)
                    .await?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&result)
                        .map_err(|err| AppError::Internal(err.to_string()))?
                );
                Ok(())
            }
            CliCommand::Serve(args) => {
                server::serve(args.host, args.port, args.cdp_ports, args.password).await
            }
        }
    }
}
