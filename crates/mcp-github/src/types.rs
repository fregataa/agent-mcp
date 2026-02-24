use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// --- Tool parameter types ---

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
