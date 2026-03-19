mod adapter;
mod core;
mod repo;
mod server;
mod shared;

use anyhow::Result;
use core::Registry_core;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("rules_mcp=info".parse()?),
        )
        .init();

    info!("RulesMCP MCP server starting");

    // Ensure Rules repo is cloned and up-to-date
    let repo_path = repo::ensure_repo().await?;
    info!("Rules repo at: {}", repo_path.display());

    // Load registry from register.jsonl
    let mut registry = Registry_core::new();
    registry.load(&repo_path).await?;
    info!("Loaded {} rules from registry", registry.entries.len());

    // Create MCP server with shared registry
    let registry = Arc::new(Mutex::new(registry));
    let rules_server = adapter::RulesMcpServer {
        registry: registry.clone(),
        repo_path,
    };

    // Run stdio MCP server
    rules_server.run_stdio().await?;

    Ok(())
}
