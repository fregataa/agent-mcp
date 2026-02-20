use thiserror::Error;

#[derive(Debug, Error)]
pub enum McpApiError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("API error ({status}): {body}")]
    Api { status: u16, body: String },

    #[error("Failed to deserialize response: {message}\nBody: {body}")]
    Deserialize { message: String, body: String },

    #[error("Missing environment variable: {0}")]
    MissingEnv(String),
}

impl McpApiError {
    pub fn to_mcp_error(&self) -> String {
        self.to_string()
    }
}
