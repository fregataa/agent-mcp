mod client;
mod server;
mod types;

use anyhow::{bail, Context, Result};
use rmcp::{transport::stdio, ServiceExt};
use tracing_subscriber::EnvFilter;

use client::GitHubClient;
use server::GitHubServer;

fn env_var(name: &str) -> Result<String> {
    std::env::var(name).with_context(|| format!("Missing environment variable: {}", name))
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let token = env_var("GITHUB_TOKEN")?;

    if token.is_empty() {
        bail!("GITHUB_TOKEN is empty");
    }

    let client = GitHubClient::new(token);
    let server = GitHubServer::new(client);

    tracing::info!("Starting GitHub MCP server on stdio");

    let service = server.serve(stdio()).await?;
    service.waiting().await?;

    Ok(())
}
