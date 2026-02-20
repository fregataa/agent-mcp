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

- **mcp-jira** — JIRA MCP server. `JiraClient` wraps JIRA REST API v3 with Basic Auth (base64 email:token). `JiraServer` exposes tools via rmcp's `#[tool_router]`/`#[tool]` macros. `adf.rs` handles Atlassian Document Format conversion (plain text <-> ADF).

- **mcp-github** — GitHub MCP server. `GitHubClient` wraps GitHub REST API with Bearer token auth. `GitHubServer` exposes PR management tools.

### Key Pattern: Tool Definition

Tools are defined as async methods on the server struct using rmcp macros:

```rust
#[tool(description = "...")]
async fn tool_name(&self, Parameters(params): Parameters<ParamType>) -> Result<CallToolResult, McpError> { ... }
```

Parameter types derive `JsonSchema` with `#[schemars(description = "...")]` for field descriptions. The schema is auto-generated at compile time.

### Server Initialization

Each binary follows the same pattern: init tracing -> load/validate env vars -> create client -> create server -> serve on stdio. Logs go to stderr to avoid interfering with the MCP protocol on stdout.

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
