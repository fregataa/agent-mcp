use rmcp::{
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::*,
    tool, tool_handler, tool_router,
    ErrorData as McpError, ServerHandler,
};

use crate::adf::adf_to_text;
use crate::client::JiraClient;
use crate::types::*;

#[derive(Clone)]
pub struct JiraServer {
    client: std::sync::Arc<JiraClient>,
    my_account_id: Option<String>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl JiraServer {
    pub fn new(client: JiraClient, my_account_id: Option<String>) -> Self {
        Self {
            client: std::sync::Arc::new(client),
            my_account_id,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Search JIRA issues using JQL query language")]
    async fn jira_search_issues(
        &self,
        Parameters(params): Parameters<SearchIssuesParams>,
    ) -> Result<CallToolResult, McpError> {
        let max_results = params.max_results.unwrap_or(20).min(50);
        match self.client.search_issues(&params.jql, max_results).await {
            Ok(result) => {
                let total_str = result
                    .total
                    .map(|t| t.to_string())
                    .unwrap_or_else(|| format!("{}+", result.issues.len()));
                let mut text = format!(
                    "Found {} issues (showing up to {}):\n\n",
                    total_str, max_results
                );
                for issue in &result.issues {
                    let status = issue
                        .fields
                        .status
                        .as_ref()
                        .map(|s| s.name.as_str())
                        .unwrap_or("Unknown");
                    let summary = issue
                        .fields
                        .summary
                        .as_deref()
                        .unwrap_or("(no summary)");
                    let assignee = issue
                        .fields
                        .assignee
                        .as_ref()
                        .and_then(|a| a.display_name.as_deref())
                        .unwrap_or("Unassigned");
                    text.push_str(&format!(
                        "- {} [{}] {}\n  Assignee: {}\n",
                        issue.key, status, summary, assignee
                    ));
                }
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => Err(McpError::internal_error(e.to_mcp_error(), None)),
        }
    }

    #[tool(description = "Get detailed information about a JIRA issue by its key")]
    async fn jira_get_issue(
        &self,
        Parameters(params): Parameters<GetIssueParams>,
    ) -> Result<CallToolResult, McpError> {
        match self.client.get_issue(&params.issue_key).await {
            Ok(issue) => {
                let f = &issue.fields;
                let status = f
                    .status
                    .as_ref()
                    .map(|s| s.name.as_str())
                    .unwrap_or("Unknown");
                let summary = f.summary.as_deref().unwrap_or("(no summary)");
                let issue_type = f
                    .issuetype
                    .as_ref()
                    .map(|t| t.name.as_str())
                    .unwrap_or("Unknown");
                let priority = f
                    .priority
                    .as_ref()
                    .map(|p| p.name.as_str())
                    .unwrap_or("None");
                let assignee = f
                    .assignee
                    .as_ref()
                    .and_then(|a| a.display_name.as_deref())
                    .unwrap_or("Unassigned");
                let reporter = f
                    .reporter
                    .as_ref()
                    .and_then(|a| a.display_name.as_deref())
                    .unwrap_or("Unknown");
                let description = f
                    .description
                    .as_ref()
                    .map(adf_to_text)
                    .unwrap_or_else(|| "(no description)".to_string());
                let created = f.created.as_deref().unwrap_or("Unknown");
                let updated = f.updated.as_deref().unwrap_or("Unknown");

                // Get available transitions
                let transitions_text =
                    match self.client.get_transitions(&params.issue_key).await {
                        Ok(resp) => {
                            let items: Vec<String> = resp
                                .transitions
                                .iter()
                                .map(|t| format!("  - {} (id: {})", t.name, t.id))
                                .collect();
                            if items.is_empty() {
                                "  (none)".to_string()
                            } else {
                                items.join("\n")
                            }
                        }
                        Err(_) => "  (failed to fetch)".to_string(),
                    };

                let text = format!(
                    "Key: {}\nType: {}\nSummary: {}\nStatus: {}\nPriority: {}\nAssignee: {}\nReporter: {}\nCreated: {}\nUpdated: {}\n\nDescription:\n{}\n\nAvailable Transitions:\n{}",
                    issue.key, issue_type, summary, status, priority, assignee, reporter, created, updated, description, transitions_text
                );
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => Err(McpError::internal_error(e.to_mcp_error(), None)),
        }
    }

    #[tool(description = "Create a new JIRA issue")]
    async fn jira_create_issue(
        &self,
        Parameters(params): Parameters<CreateIssueParams>,
    ) -> Result<CallToolResult, McpError> {
        match self
            .client
            .create_issue(
                &params.project_key,
                &params.issue_type,
                &params.summary,
                params.description.as_deref(),
                params.custom_fields.as_ref(),
            )
            .await
        {
            Ok(result) => {
                let text = format!(
                    "Issue created successfully!\nKey: {}\nID: {}\nURL: {}",
                    result.key, result.id, result.self_url
                );
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => Err(McpError::internal_error(e.to_mcp_error(), None)),
        }
    }

    #[tool(description = "Update a JIRA issue (summary, description, assignee, priority, custom fields). Use 'me' as assignee to assign to the configured user, 'none' to unassign.")]
    async fn jira_update_issue(
        &self,
        Parameters(params): Parameters<UpdateIssueParams>,
    ) -> Result<CallToolResult, McpError> {
        let assignee = params.assignee.as_deref().map(|a| {
            if a == "me" {
                self.my_account_id.as_deref().unwrap_or(a)
            } else {
                a
            }
        });
        match self
            .client
            .update_issue(
                &params.issue_key,
                params.summary.as_deref(),
                params.description.as_deref(),
                assignee,
                params.priority.as_deref(),
                params.custom_fields.as_ref(),
            )
            .await
        {
            Ok(()) => {
                let text = format!("Issue {} updated successfully", params.issue_key);
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => Err(McpError::internal_error(e.to_mcp_error(), None)),
        }
    }

    #[tool(description = "Search for JIRA users by name or email to get their account ID (needed for assignee)")]
    async fn jira_find_user(
        &self,
        Parameters(params): Parameters<FindUserParams>,
    ) -> Result<CallToolResult, McpError> {
        match self.client.find_user(&params.query).await {
            Ok(users) => {
                if users.is_empty() {
                    return Ok(CallToolResult::success(vec![Content::text(
                        "No users found",
                    )]));
                }
                let mut text = format!("Found {} user(s):\n\n", users.len());
                for user in &users {
                    let name = user.display_name.as_deref().unwrap_or("(unknown)");
                    let email = user
                        .email_address
                        .as_deref()
                        .unwrap_or("(no email)");
                    let active = if user.active.unwrap_or(true) {
                        "active"
                    } else {
                        "inactive"
                    };
                    text.push_str(&format!(
                        "- {} <{}> [{}]\n  Account ID: {}\n",
                        name, email, active, user.account_id
                    ));
                }
                if let Some(ref my_id) = self.my_account_id {
                    text.push_str(&format!("\nNote: Your configured account ID is: {}", my_id));
                }
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => Err(McpError::internal_error(e.to_mcp_error(), None)),
        }
    }

    #[tool(description = "Transition a JIRA issue to a new status (use jira_get_issue to see available transitions)")]
    async fn jira_transition_issue(
        &self,
        Parameters(params): Parameters<TransitionIssueParams>,
    ) -> Result<CallToolResult, McpError> {
        match self
            .client
            .transition_issue(&params.issue_key, &params.transition_id)
            .await
        {
            Ok(()) => {
                let text = format!(
                    "Issue {} transitioned successfully (transition id: {})",
                    params.issue_key, params.transition_id
                );
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => Err(McpError::internal_error(e.to_mcp_error(), None)),
        }
    }

    #[tool(description = "Add a comment to a JIRA issue")]
    async fn jira_add_comment(
        &self,
        Parameters(params): Parameters<AddCommentParams>,
    ) -> Result<CallToolResult, McpError> {
        match self
            .client
            .add_comment(&params.issue_key, &params.body)
            .await
        {
            Ok(comment) => {
                let text = format!(
                    "Comment added successfully to {}!\nComment ID: {}",
                    params.issue_key, comment.id
                );
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => Err(McpError::internal_error(e.to_mcp_error(), None)),
        }
    }

    #[tool(description = "List all comments on a JIRA issue")]
    async fn jira_list_comments(
        &self,
        Parameters(params): Parameters<ListCommentsParams>,
    ) -> Result<CallToolResult, McpError> {
        match self.client.list_comments(&params.issue_key).await {
            Ok(result) => {
                let mut text = format!(
                    "Comments on {} ({} total):\n\n",
                    params.issue_key, result.total
                );
                for comment in &result.comments {
                    let author = comment
                        .author
                        .as_ref()
                        .and_then(|a| a.display_name.as_deref())
                        .unwrap_or("Unknown");
                    let body = comment
                        .body
                        .as_ref()
                        .map(adf_to_text)
                        .unwrap_or_else(|| "(empty)".to_string());
                    let created = comment.created.as_deref().unwrap_or("Unknown");
                    text.push_str(&format!(
                        "--- Comment {} ---\nAuthor: {}\nCreated: {}\n{}\n\n",
                        comment.id, author, created, body
                    ));
                }
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => Err(McpError::internal_error(e.to_mcp_error(), None)),
        }
    }
}

#[tool_handler]
impl ServerHandler for JiraServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "JIRA MCP server providing tools for searching, creating, and managing JIRA issues."
                    .to_string(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
