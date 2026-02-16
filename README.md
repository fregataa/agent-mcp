# agent-mcp

JIRA and GitHub MCP servers for Claude Code agent-teams.

## Architecture

Cargo workspace with 3 crates:

- **mcp-common** — Shared HTTP client and error types
- **mcp-jira** — JIRA MCP server (stdio transport)
- **mcp-github** — GitHub MCP server (stdio transport)

## Tools

### JIRA (6 tools)

| Tool | Description |
|---|---|
| `jira_search_issues` | Search issues using JQL |
| `jira_get_issue` | Get issue details by key |
| `jira_create_issue` | Create a new issue |
| `jira_transition_issue` | Change issue status |
| `jira_add_comment` | Add comment to an issue |
| `jira_list_comments` | List comments on an issue |

### GitHub (3 tools)

| Tool | Description |
|---|---|
| `github_create_pr` | Create a pull request |
| `github_update_pr` | Update PR title/body/state |
| `github_get_pr` | Get PR details |

## Setup

### Build

```bash
cargo build --release
```

### Environment Variables

Copy `.env.example` and fill in your credentials:

```
JIRA_BASE_URL=https://lablup.atlassian.net
JIRA_EMAIL=sanghun@lablup.com
JIRA_API_TOKEN=<token>
GITHUB_TOKEN=<token>
```

### Claude Code Registration

Add to `~/.claude.json`:

```json
{
  "mcpServers": {
    "jira": {
      "command": "/path/to/agent-mcp/target/release/mcp-jira",
      "env": {
        "JIRA_BASE_URL": "https://lablup.atlassian.net",
        "JIRA_EMAIL": "sanghun@lablup.com",
        "JIRA_API_TOKEN": "<token>"
      }
    },
    "github": {
      "command": "/path/to/agent-mcp/target/release/mcp-github",
      "env": {
        "GITHUB_TOKEN": "<token>"
      }
    }
  }
}
```

## Tech Stack

- [rmcp](https://github.com/modelcontextprotocol/rust-sdk) 0.15 — Rust MCP SDK
- tokio — Async runtime
- reqwest — HTTP client
- serde / schemars — Serialization and JSON Schema
