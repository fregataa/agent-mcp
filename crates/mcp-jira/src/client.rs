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
    /// Parse response body using bytes() to avoid reqwest charset decoding issues,
    /// then deserialize with serde_json for better error messages.
    async fn parse_response<T: serde::de::DeserializeOwned>(
        resp: reqwest::Response,
    ) -> Result<T, McpApiError> {
        let bytes = resp.bytes().await?;
        let body = String::from_utf8_lossy(&bytes);
        serde_json::from_str(&body).map_err(|e| {
            tracing::error!("Failed to deserialize response: {}", e);
            tracing::error!("Raw response body: {}", body);
            McpApiError::Deserialize {
                message: e.to_string(),
                body: body.into_owned(),
            }
        })
    }

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
        let url = format!("{}/rest/api/3/search/jql", self.base_url);
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
        Self::parse_response(resp).await
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
        Self::parse_response(resp).await
    }

    pub async fn create_issue(
        &self,
        project_key: &str,
        issue_type: &str,
        summary: &str,
        description: Option<&str>,
        parent: Option<&str>,
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

        if let Some(parent_key) = parent {
            fields["parent"] = serde_json::json!({"key": parent_key});
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
        Self::parse_response(resp).await
    }

    pub async fn update_issue(
        &self,
        issue_key: &str,
        summary: Option<&str>,
        description: Option<&str>,
        assignee: Option<&str>,
        priority: Option<&str>,
        custom_fields: Option<&serde_json::Value>,
    ) -> Result<(), McpApiError> {
        let url = format!("{}/rest/api/3/issue/{}", self.base_url, issue_key);

        let mut fields = serde_json::json!({});

        if let Some(s) = summary {
            fields["summary"] = serde_json::Value::String(s.to_string());
        }
        if let Some(desc) = description {
            fields["description"] = text_to_adf(desc);
        }
        if let Some(a) = assignee {
            if a == "none" {
                fields["assignee"] = serde_json::Value::Null;
            } else {
                fields["assignee"] = serde_json::json!({"accountId": a});
            }
        }
        if let Some(p) = priority {
            fields["priority"] = serde_json::json!({"name": p});
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
            .put(&url)
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
        Self::parse_response(resp).await
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
        Self::parse_response(resp).await
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
        Self::parse_response(resp).await
    }

    pub async fn find_user(
        &self,
        query: &str,
    ) -> Result<Vec<JiraUserSearchResult>, McpApiError> {
        let url = format!("{}/rest/api/3/user/search", self.base_url);
        let resp = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .header("Accept", "application/json")
            .query(&[("query", query)])
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(McpApiError::Api { status, body });
        }
        Self::parse_response(resp).await
    }
}
