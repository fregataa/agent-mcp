mod adf;
mod client;
mod server;
mod types;

use anyhow::{bail, Context, Result};
use rmcp::{transport::stdio, ServiceExt};
use tracing_subscriber::EnvFilter;

use client::JiraClient;
use server::JiraServer;

fn env_var(name: &str) -> Result<String> {
    std::env::var(name).with_context(|| format!("Missing environment variable: {}", name))
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let base_url = env_var("JIRA_BASE_URL")?;
    let email = env_var("JIRA_EMAIL")?;
    let api_token = env_var("JIRA_API_TOKEN")?;

    if api_token.is_empty() {
        bail!("JIRA_API_TOKEN is empty");
    }

    let my_account_id = std::env::var("JIRA_MY_ACCOUNT_ID").ok().filter(|s| !s.is_empty());

    let client = JiraClient::new(base_url, email, api_token);
    let server = JiraServer::new(client, my_account_id);

    tracing::info!("Starting JIRA MCP server on stdio");

    let service = server.serve(stdio()).await?;
    service.waiting().await?;

    Ok(())
}
