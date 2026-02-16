use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// --- Tool parameter types ---

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchIssuesParams {
    #[schemars(description = "JQL query string")]
    pub jql: String,
    #[schemars(description = "Maximum number of results (default: 20, max: 50)")]
    pub max_results: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetIssueParams {
    #[schemars(description = "Issue key (e.g. PROJ-123)")]
    pub issue_key: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateIssueParams {
    #[schemars(description = "Project key (e.g. PROJ)")]
    pub project_key: String,
    #[schemars(description = "Issue type name (e.g. Task, Bug, Story)")]
    pub issue_type: String,
    #[schemars(description = "Issue summary/title")]
    pub summary: String,
    #[schemars(description = "Issue description in plain text (will be converted to ADF)")]
    pub description: Option<String>,
    #[schemars(description = "Custom fields as JSON object (e.g. {\"customfield_10001\": \"value\", \"customfield_10002\": {\"id\": \"10100\"}})")]
    pub custom_fields: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateIssueParams {
    #[schemars(description = "Issue key (e.g. PROJ-123)")]
    pub issue_key: String,
    #[schemars(description = "New summary/title")]
    pub summary: Option<String>,
    #[schemars(description = "New description in plain text (will be converted to ADF)")]
    pub description: Option<String>,
    #[schemars(description = "Assignee account ID (use 'none' to unassign)")]
    pub assignee: Option<String>,
    #[schemars(description = "Priority name (e.g. High, Medium, Low)")]
    pub priority: Option<String>,
    #[schemars(description = "Custom fields as JSON object (e.g. {\"customfield_10001\": \"value\"})")]
    pub custom_fields: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FindUserParams {
    #[schemars(description = "Search query (name or email address)")]
    pub query: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TransitionIssueParams {
    #[schemars(description = "Issue key (e.g. PROJ-123)")]
    pub issue_key: String,
    #[schemars(description = "Transition ID (get available transitions from issue details)")]
    pub transition_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddCommentParams {
    #[schemars(description = "Issue key (e.g. PROJ-123)")]
    pub issue_key: String,
    #[schemars(description = "Comment body in plain text (will be converted to ADF)")]
    pub body: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListCommentsParams {
    #[schemars(description = "Issue key (e.g. PROJ-123)")]
    pub issue_key: String,
}

// --- JIRA API response types ---

#[derive(Debug, Deserialize)]
pub struct JiraSearchResponse {
    pub issues: Vec<JiraIssue>,
    pub total: u32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct JiraIssue {
    pub key: String,
    #[serde(rename = "self")]
    pub self_url: Option<String>,
    pub fields: JiraIssueFields,
}

#[derive(Debug, Deserialize)]
pub struct JiraIssueFields {
    pub summary: Option<String>,
    pub status: Option<JiraStatus>,
    pub assignee: Option<JiraUser>,
    pub reporter: Option<JiraUser>,
    pub priority: Option<JiraPriority>,
    pub issuetype: Option<JiraIssueType>,
    pub description: Option<serde_json::Value>,
    pub created: Option<String>,
    pub updated: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct JiraStatus {
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct JiraUser {
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "emailAddress")]
    pub email_address: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct JiraPriority {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct JiraIssueType {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct JiraCreateResponse {
    pub id: String,
    pub key: String,
    #[serde(rename = "self")]
    pub self_url: String,
}

#[derive(Debug, Deserialize)]
pub struct JiraTransitionsResponse {
    pub transitions: Vec<JiraTransition>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JiraTransition {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct JiraCommentsResponse {
    pub comments: Vec<JiraComment>,
    pub total: u32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct JiraComment {
    pub id: String,
    pub author: Option<JiraUser>,
    pub body: Option<serde_json::Value>,
    pub created: Option<String>,
    pub updated: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct JiraUserSearchResult {
    #[serde(rename = "accountId")]
    pub account_id: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "emailAddress")]
    pub email_address: Option<String>,
    pub active: Option<bool>,
}
