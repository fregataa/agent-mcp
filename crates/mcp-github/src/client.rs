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

    #[allow(clippy::too_many_arguments)]
    pub async fn create_pr(
        &self,
        owner: &str,
        repo: &str,
        title: &str,
        head: &str,
        base: &str,
        body: Option<&str>,
        draft: Option<bool>,
    ) -> Result<GitHubPr, McpApiError> {
        let url = self.api_url(&format!("/repos/{}/{}/pulls", owner, repo));
        let mut payload = serde_json::json!({
            "title": title,
            "head": head,
            "base": base,
        });
        if let Some(body) = body {
            payload["body"] = serde_json::Value::String(body.to_string());
        }
        if let Some(draft) = draft {
            payload["draft"] = serde_json::Value::Bool(draft);
        }

        let resp = self
            .add_headers(self.client.post(&url))
            .json(&payload)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(McpApiError::Api { status, body });
        }
        Ok(resp.json().await?)
    }

    pub async fn update_pr(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        title: Option<&str>,
        body: Option<&str>,
        state: Option<&str>,
    ) -> Result<GitHubPr, McpApiError> {
        let url = self.api_url(&format!("/repos/{}/{}/pulls/{}", owner, repo, pull_number));
        let mut payload = serde_json::json!({});
        if let Some(title) = title {
            payload["title"] = serde_json::Value::String(title.to_string());
        }
        if let Some(body) = body {
            payload["body"] = serde_json::Value::String(body.to_string());
        }
        if let Some(state) = state {
            payload["state"] = serde_json::Value::String(state.to_string());
        }

        let resp = self
            .add_headers(self.client.patch(&url))
            .json(&payload)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(McpApiError::Api { status, body });
        }
        Ok(resp.json().await?)
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
