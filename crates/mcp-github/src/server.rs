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
    #[tool(description = "List reviews on a GitHub pull request (approve, request changes, comment, etc.)")]
    async fn github_list_pr_reviews(
        &self,
        Parameters(params): Parameters<ListPrReviewsParams>,
    ) -> Result<CallToolResult, McpError> {
        match self
            .client
            .list_pr_reviews(&params.owner, &params.repo, params.pull_number)
            .await
        {
            Ok(reviews) => {
                if reviews.is_empty() {
                    return Ok(CallToolResult::success(vec![Content::text(
                        "No reviews found",
                    )]));
                }
                let mut text = format!("Reviews ({}):\n\n", reviews.len());
                for r in &reviews {
                    let author = r
                        .user
                        .as_ref()
                        .map(|u| u.login.as_str())
                        .unwrap_or("Unknown");
                    let body = r.body.as_deref().unwrap_or("(no body)");
                    let submitted = r.submitted_at.as_deref().unwrap_or("Unknown");
                    text.push_str(&format!(
                        "--- Review {} ---\nAuthor: {}\nState: {}\nSubmitted: {}\n{}\n\n",
                        r.id, author, r.state, submitted, body
                    ));
                }
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => Err(McpError::internal_error(e.to_mcp_error(), None)),
        }
    }

    #[tool(description = "List inline review comments on a GitHub pull request (comments on specific lines of code)")]
    async fn github_list_pr_comments(
        &self,
        Parameters(params): Parameters<ListPrCommentsParams>,
    ) -> Result<CallToolResult, McpError> {
        match self
            .client
            .list_pr_comments(&params.owner, &params.repo, params.pull_number)
            .await
        {
            Ok(comments) => {
                if comments.is_empty() {
                    return Ok(CallToolResult::success(vec![Content::text(
                        "No review comments found",
                    )]));
                }
                let mut text = format!("Review comments ({}):\n\n", comments.len());
                for c in &comments {
                    let author = c
                        .user
                        .as_ref()
                        .map(|u| u.login.as_str())
                        .unwrap_or("Unknown");
                    let body = c.body.as_deref().unwrap_or("(no body)");
                    let path = c.path.as_deref().unwrap_or("(unknown file)");
                    let line = c
                        .line
                        .or(c.original_line)
                        .map(|l| l.to_string())
                        .unwrap_or_else(|| "N/A".to_string());
                    let created = c.created_at.as_deref().unwrap_or("Unknown");
                    let diff_hunk = c.diff_hunk.as_deref().unwrap_or("");
                    text.push_str(&format!(
                        "--- Comment {} ---\nAuthor: {}\nFile: {} (line {})\nCreated: {}\n\n{}\n\n```diff\n{}\n```\n\n",
                        c.id, author, path, line, created, body, diff_hunk
                    ));
                }
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
                "GitHub MCP server providing tools for viewing pull requests."
                    .to_string(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
