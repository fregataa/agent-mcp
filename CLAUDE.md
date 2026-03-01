# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Rust Cargo workspace providing JIRA and GitHub MCP (Model Context Protocol) servers for Claude Code agent-teams. Each server runs as a standalone binary communicating via stdio transport.

## Build & Development Commands

```bash
cargo build --release                  # Build all crates
cargo build --release -p mcp-jira      # Build only JIRA server
cargo build --release -p mcp-github    # Build only GitHub server
cargo test                             # Run all tests
cargo test -p mcp-jira                 # Run tests for a single crate
cargo fmt                              # Format code
cargo clippy                           # Lint
```

## Architecture

Three crates in `crates/`:

- **mcp-common** — Shared `McpApiError` enum (in `error.rs`) and `build_client()` HTTP client factory (in `http.rs`). All API errors flow through `McpApiError` which has variants for request failures, API errors (with status/body), deserialization errors, and missing env vars.

- **mcp-jira** — JIRA MCP server. `JiraClient` wraps JIRA REST API v3 with Basic Auth (base64 email:token). `JiraServer` exposes tools via rmcp macros. `adf.rs` handles Atlassian Document Format conversion (plain text <-> ADF).

- **mcp-github** — GitHub MCP server. `GitHubClient` wraps GitHub REST API with Bearer token auth. `GitHubServer` exposes PR management tools.

### Adding a New Tool

Three files must be modified in each server crate — always in this order:

1. **`types.rs`** — Add parameter struct (derives `Deserialize`, `JsonSchema`) and any new API response types. Each field needs `#[schemars(description = "...")]`.

2. **`client.rs`** — Add async method on the client struct that calls the external API, handles HTTP errors, and returns `Result<T, McpApiError>`.

3. **`server.rs`** — Add `#[tool(description = "...")]` method on the server struct that calls the client method, formats output as text, and returns `Result<CallToolResult, McpError>`.

### rmcp Macro Stack

Each server struct uses three macro layers:

```rust
#[tool_router]           // On impl block: generates tool routing logic
impl MyServer {
    #[tool(description = "...")] // On each method: registers it as an MCP tool
    async fn my_tool(&self, Parameters(params): Parameters<MyParams>) -> Result<CallToolResult, McpError> { ... }
}

#[tool_handler]          // On ServerHandler impl: wires routing into MCP protocol
impl ServerHandler for MyServer { ... }
```

Tool names exposed to MCP clients are the snake_case method names (e.g., `jira_search_issues`).

### Error Handling Flow

Client methods return `Result<T, McpApiError>`. Server tool methods convert errors for MCP:

```rust
Err(e) => Err(McpError::internal_error(e.to_mcp_error(), None))
```

where `to_mcp_error()` returns `self.to_string()` (via thiserror Display).

### Response Construction

All tools return formatted text:

```rust
Ok(CallToolResult::success(vec![Content::text(formatted_string)]))
```

### Key Implementation Details

- **Arc-wrapped clients**: Server structs hold `Arc<Client>` for thread-safe sharing across async tasks.
- **JIRA response parsing**: `JiraClient` uses `bytes()` → `String::from_utf8_lossy()` → `serde_json::from_str()` instead of direct `.json()` to avoid reqwest's charset auto-detection corrupting JIRA responses.
- **ADF conversion** (`adf.rs`): `text_to_adf()` converts plain text to Atlassian Document Format JSON for creating/updating issues and comments. `adf_to_text()` recursively walks ADF JSON to extract readable text.
- **"me" assignee**: JIRA server resolves the magic string `"me"` to `JIRA_MY_ACCOUNT_ID` env var for assignee fields.
- **Eager transition fetching**: `jira_get_issue()` automatically includes available transitions in output.
- **Logs to stderr**: All tracing output goes to stderr; stdout is reserved for MCP stdio protocol.

## Environment Variables

Required in `.env` or system environment (see `.env.example`):

| Variable | Used By | Notes |
|---|---|---|
| `JIRA_BASE_URL` | mcp-jira | e.g. `https://lablup.atlassian.net` |
| `JIRA_EMAIL` | mcp-jira | For Basic Auth |
| `JIRA_API_TOKEN` | mcp-jira | Must be non-empty |
| `JIRA_MY_ACCOUNT_ID` | mcp-jira | Optional; enables "me" as assignee value |
| `GITHUB_TOKEN` | mcp-github | Bearer token, must be non-empty |

## Registration

Binaries are registered in `~/.claude.json` under `mcpServers` with command paths pointing to `target/release/mcp-jira` and `target/release/mcp-github`, with env vars passed in the config.

## Dependencies

Core: `rmcp` 0.15 (MCP SDK with server/transport-io/macros features), `tokio`, `reqwest`, `serde`/`schemars`, `thiserror`, `anyhow`.
