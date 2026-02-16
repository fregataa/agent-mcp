use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// --- Tool parameter types ---

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreatePrParams {
    #[schemars(description = "Repository owner (user or organization)")]
    pub owner: String,
    #[schemars(description = "Repository name")]
    pub repo: String,
    #[schemars(description = "PR title")]
    pub title: String,
    #[schemars(description = "Branch containing changes")]
    pub head: String,
    #[schemars(description = "Branch to merge into (e.g. main)")]
    pub base: String,
    #[schemars(description = "PR description/body (markdown), appended after the resolves line")]
    pub body: Option<String>,
    #[schemars(description = "Create as draft PR")]
    pub draft: Option<bool>,
    #[schemars(description = "GitHub issue number to resolve (prepends 'resolves #N' to body)")]
    pub issue_number: Option<u64>,
    #[schemars(description = "JIRA issue key (e.g. BA-1234), shown alongside the resolves line")]
    pub jira_key: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdatePrParams {
    #[schemars(description = "Repository owner")]
    pub owner: String,
    #[schemars(description = "Repository name")]
    pub repo: String,
    #[schemars(description = "PR number")]
    pub pull_number: u64,
    #[schemars(description = "New PR title")]
    pub title: Option<String>,
    #[schemars(description = "New PR body (markdown)")]
    pub body: Option<String>,
    #[schemars(description = "New state: open or closed")]
    pub state: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetPrParams {
    #[schemars(description = "Repository owner")]
    pub owner: String,
    #[schemars(description = "Repository name")]
    pub repo: String,
    #[schemars(description = "PR number")]
    pub pull_number: u64,
}

// --- GitHub API response types ---

#[derive(Debug, Deserialize, Serialize)]
pub struct GitHubPr {
    pub number: u64,
    pub title: String,
    pub state: String,
    pub html_url: String,
    pub body: Option<String>,
    pub draft: Option<bool>,
    pub user: Option<GitHubUser>,
    pub head: Option<GitHubRef>,
    pub base: Option<GitHubRef>,
    pub merged: Option<bool>,
    pub mergeable: Option<bool>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub additions: Option<u64>,
    pub deletions: Option<u64>,
    pub changed_files: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitHubUser {
    pub login: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitHubRef {
    #[serde(rename = "ref")]
    pub ref_name: Option<String>,
    pub sha: Option<String>,
}
