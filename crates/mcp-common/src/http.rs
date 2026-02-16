use reqwest::Client;

pub fn build_client() -> Client {
    Client::builder()
        .user_agent("agent-mcp/0.1.0")
        .build()
        .expect("failed to build HTTP client")
}
