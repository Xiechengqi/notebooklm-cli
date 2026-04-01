mod agent_browser;
mod auth;
mod cli;
mod commands;
mod config;
mod db;
mod discovery;
mod embedded;
mod errors;
mod manifest;
mod models;
mod notebooklm;
mod response;
mod server;

use crate::cli::Cli;
use clap::Parser;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(err) = cli.run().await {
        eprintln!("{err}");
        std::process::exit(err.exit_code());
    }
}
