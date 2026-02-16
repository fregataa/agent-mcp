use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use mcp_common::McpApiError;
use reqwest::Client;

use crate::adf::text_to_adf;
use crate::types::*;

pub struct JiraClient {
    client: Client,
    base_url: String,
    auth_header: String,
}

impl JiraClient {
    pub fn new(base_url: String, email: String, api_token: String) -> Self {
        let auth = BASE64.encode(format!("{}:{}", email, api_token));
        Self {
            client: mcp_common::build_client(),
            base_url: base_url.trim_end_matches('/').to_string(),
            auth_header: format!("Basic {}", auth),
        }
    }

    pub async fn search_issues(
        &self,
        jql: &str,
        max_results: u32,
    ) -> Result<JiraSearchResponse, McpApiError> {
        let url = format!("{}/rest/api/3/search", self.base_url);
        let resp = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .header("Accept", "application/json")
            .query(&[
                ("jql", jql),
                ("maxResults", &max_results.to_string()),
                (
                    "fields",
                    "summary,status,assignee,reporter,priority,issuetype,created,updated",
                ),
            ])
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(McpApiError::Api { status, body });
        }
        Ok(resp.json().await?)
    }

    pub async fn get_issue(&self, issue_key: &str) -> Result<JiraIssue, McpApiError> {
        let url = format!("{}/rest/api/3/issue/{}", self.base_url, issue_key);
        let resp = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .header("Accept", "application/json")
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(McpApiError::Api { status, body });
        }
        Ok(resp.json().await?)
    }

    pub async fn create_issue(
        &self,
        project_key: &str,
        issue_type: &str,
        summary: &str,
        description: Option<&str>,
        custom_fields: Option<&serde_json::Value>,
    ) -> Result<JiraCreateResponse, McpApiError> {
        let url = format!("{}/rest/api/3/issue", self.base_url);

        let mut fields = serde_json::json!({
            "project": { "key": project_key },
            "issuetype": { "name": issue_type },
            "summary": summary,
        });

        if let Some(desc) = description {
            fields["description"] = text_to_adf(desc);
        }

        if let Some(custom) = custom_fields {
            if let Some(obj) = custom.as_object() {
                for (k, v) in obj {
                    fields[k] = v.clone();
                }
            }
        }

        let body = serde_json::json!({ "fields": fields });

        let resp = self
            .client
            .post(&url)
            .header("Authorization", &self.auth_header)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(McpApiError::Api { status, body });
        }
        Ok(resp.json().await?)
    }

    pub async fn get_transitions(
        &self,
        issue_key: &str,
    ) -> Result<JiraTransitionsResponse, McpApiError> {
        let url = format!(
            "{}/rest/api/3/issue/{}/transitions",
            self.base_url, issue_key
        );
        let resp = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .header("Accept", "application/json")
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(McpApiError::Api { status, body });
        }
        Ok(resp.json().await?)
    }

    pub async fn transition_issue(
        &self,
        issue_key: &str,
        transition_id: &str,
    ) -> Result<(), McpApiError> {
        let url = format!(
            "{}/rest/api/3/issue/{}/transitions",
            self.base_url, issue_key
        );
        let body = serde_json::json!({
            "transition": { "id": transition_id }
        });

        let resp = self
            .client
            .post(&url)
            .header("Authorization", &self.auth_header)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(McpApiError::Api { status, body });
        }
        Ok(())
    }

    pub async fn add_comment(
        &self,
        issue_key: &str,
        body_text: &str,
    ) -> Result<JiraComment, McpApiError> {
        let url = format!(
            "{}/rest/api/3/issue/{}/comment",
            self.base_url, issue_key
        );
        let body = serde_json::json!({
            "body": text_to_adf(body_text)
        });

        let resp = self
            .client
            .post(&url)
            .header("Authorization", &self.auth_header)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(McpApiError::Api { status, body });
        }
        Ok(resp.json().await?)
    }

    pub async fn list_comments(
        &self,
        issue_key: &str,
    ) -> Result<JiraCommentsResponse, McpApiError> {
        let url = format!(
            "{}/rest/api/3/issue/{}/comment",
            self.base_url, issue_key
        );
        let resp = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .header("Accept", "application/json")
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
