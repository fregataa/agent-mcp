use mcp_common::McpApiError;
use reqwest::Client;

use crate::types::*;

pub struct GitHubClient {
    client: Client,
    token: String,
}

impl GitHubClient {
    pub fn new(token: String) -> Self {
        Self {
            client: mcp_common::build_client(),
            token,
        }
    }

    fn api_url(&self, path: &str) -> String {
        format!("https://api.github.com{}", path)
    }

    fn add_headers(
        &self,
        builder: reqwest::RequestBuilder,
    ) -> reqwest::RequestBuilder {
        builder
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
    }

    pub async fn get_pr(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
    ) -> Result<GitHubPr, McpApiError> {
        let url = self.api_url(&format!("/repos/{}/{}/pulls/{}", owner, repo, pull_number));
        let resp = self
            .add_headers(self.client.get(&url))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(McpApiError::Api { status, body });
        }
        Ok(resp.json().await?)
    }
}
