use rmcp::{
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::*,
    tool, tool_handler, tool_router,
    ErrorData as McpError, ServerHandler,
};

use crate::client::GitHubClient;
use crate::types::*;

#[derive(Clone)]
pub struct GitHubServer {
    client: std::sync::Arc<GitHubClient>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl GitHubServer {
    pub fn new(client: GitHubClient) -> Self {
        Self {
            client: std::sync::Arc::new(client),
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Create a new GitHub pull request")]
    async fn github_create_pr(
        &self,
        Parameters(params): Parameters<CreatePrParams>,
    ) -> Result<CallToolResult, McpError> {
        match self
            .client
            .create_pr(
                &params.owner,
                &params.repo,
                &params.title,
                &params.head,
                &params.base,
                params.body.as_deref(),
                params.draft,
            )
            .await
        {
            Ok(pr) => {
                let text = format!(
                    "PR created successfully!\nNumber: #{}\nTitle: {}\nURL: {}\nState: {}\nDraft: {}",
                    pr.number,
                    pr.title,
                    pr.html_url,
                    pr.state,
                    pr.draft.unwrap_or(false)
                );
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => Err(McpError::internal_error(e.to_mcp_error(), None)),
        }
    }

    #[tool(description = "Update an existing GitHub pull request (title, body, or state)")]
    async fn github_update_pr(
        &self,
        Parameters(params): Parameters<UpdatePrParams>,
    ) -> Result<CallToolResult, McpError> {
        match self
            .client
            .update_pr(
                &params.owner,
                &params.repo,
                params.pull_number,
                params.title.as_deref(),
                params.body.as_deref(),
                params.state.as_deref(),
            )
            .await
        {
            Ok(pr) => {
                let text = format!(
                    "PR updated successfully!\nNumber: #{}\nTitle: {}\nURL: {}\nState: {}",
                    pr.number, pr.title, pr.html_url, pr.state
                );
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => Err(McpError::internal_error(e.to_mcp_error(), None)),
        }
    }

    #[tool(description = "Get detailed information about a GitHub pull request")]
    async fn github_get_pr(
        &self,
        Parameters(params): Parameters<GetPrParams>,
    ) -> Result<CallToolResult, McpError> {
        match self
            .client
            .get_pr(&params.owner, &params.repo, params.pull_number)
            .await
        {
            Ok(pr) => {
                let author = pr
                    .user
                    .as_ref()
                    .map(|u| u.login.as_str())
                    .unwrap_or("Unknown");
                let head_ref = pr
                    .head
                    .as_ref()
                    .and_then(|h| h.ref_name.as_deref())
                    .unwrap_or("Unknown");
                let base_ref = pr
                    .base
                    .as_ref()
                    .and_then(|b| b.ref_name.as_deref())
                    .unwrap_or("Unknown");
                let body = pr.body.as_deref().unwrap_or("(no description)");
                let merged = pr.merged.unwrap_or(false);
                let additions = pr
                    .additions
                    .map(|a| a.to_string())
                    .unwrap_or_else(|| "N/A".to_string());
                let deletions = pr
                    .deletions
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "N/A".to_string());
                let changed = pr
                    .changed_files
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "N/A".to_string());

                let text = format!(
                    "PR #{}: {}\nURL: {}\nState: {}\nDraft: {}\nMerged: {}\nAuthor: {}\nBranch: {} → {}\nAdditions: {}\nDeletions: {}\nChanged files: {}\nCreated: {}\nUpdated: {}\n\nBody:\n{}",
                    pr.number,
                    pr.title,
                    pr.html_url,
                    pr.state,
                    pr.draft.unwrap_or(false),
                    merged,
                    author,
                    head_ref,
                    base_ref,
                    additions,
                    deletions,
                    changed,
                    pr.created_at.as_deref().unwrap_or("Unknown"),
                    pr.updated_at.as_deref().unwrap_or("Unknown"),
                    body
                );
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => Err(McpError::internal_error(e.to_mcp_error(), None)),
        }
    }
}

#[tool_handler]
impl ServerHandler for GitHubServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "GitHub MCP server providing tools for creating, updating, and viewing pull requests."
                    .to_string(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
