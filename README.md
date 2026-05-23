# ACP Workspace MCP Server

[![Crates.io](https://img.shields.io/crates/v/mcp-acp-workspace.svg)](https://crates.io/crates/mcp-acp-workspace)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)

Enterprise control plane for [ACP (Agent Client Protocol)](https://agentclientprotocol.com/) sessions in [ADK-Rust Enterprise](https://enterprise.adk-rust.com). Provides 9 MCP tools for managing coding delegates — permission gates, patch review, session lifecycle, and agent exposure.

## Architecture

<p align="center">
  <img src="https://raw.githubusercontent.com/zavora-ai/mcp-acp-workspace/main/docs/architecture.svg" alt="ACP Workspace Architecture" width="600"/>
</p>

## Key Principles

- **Enterprise control plane** — wraps ACP wire protocol with governance, not redefines it.
- **Permission-gated** — file writes, shell commands, and network access require explicit approval.
- **Patch review** — proposed changes are reviewable before application (platform-derived, not core ACP).
- **Full audit trail** — every session, prompt, permission decision, and patch is recorded.
- **Bidirectional** — manage coding delegates AND expose ADK-Rust agents as ACP servers.
- **Protocol-aligned** — `session/cancel` as notification, `session/update` payload preserved in full.

## Tools

| Tool | Purpose | Risk Class |
|------|---------|------------|
| `list_acp_connections` | Show registered ACP clients and servers | Read-only |
| `create_acp_session` | Start ACP session (initialize + session/new) | Internal write |
| `send_acp_prompt` | Delegate task to coding agent (session/prompt) | External write |
| `stream_acp_events` | Get session/update events (full payload preserved) | Read-only |
| `request_acp_permission` | Handle permission request with governance policy | High write |
| `get_acp_patch` | Retrieve proposed file changes (platform aggregation) | Read-only |
| `review_acp_patch` | Approve, reject, or annotate a patch | Internal write |
| `terminate_acp_session` | Stop session via session/cancel notification | Internal write |
| `expose_adk_agent_as_acp` | Publish ADK-Rust agent as ACP server | Internal write |

## ACP Protocol Alignment

| ACP Wire Method | MCP Tool | Notes |
|-----------------|----------|-------|
| `initialize` | `create_acp_session` | Version negotiation, capability exchange |
| `session/new` | `create_acp_session` | Workspace binding |
| `session/prompt` | `send_acp_prompt` | ContentBlock[] with type:text |
| `session/update` | `stream_acp_events` | Full params.update payload preserved |
| Permission request | `request_acp_permission` | Mapped to governance policy |
| `session/cancel` | `terminate_acp_session` | Notification (no wire response) |

## Installation

### Build from source

```bash
git clone https://github.com/zavora-ai/mcp-acp-workspace
cd mcp-acp-workspace
cargo build --release
```

The binary is at `target/release/mcp-acp-workspace`.

### Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "acp-workspace": {
      "command": "/path/to/mcp-acp-workspace"
    }
  }
}
```

### Kiro

Add to `.kiro/settings/mcp.json`:

```json
{
  "mcpServers": {
    "acp-workspace": {
      "command": "/path/to/mcp-acp-workspace"
    }
  }
}
```

### Cursor

Add to `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "acp-workspace": {
      "command": "/path/to/mcp-acp-workspace"
    }
  }
}
```

### Windsurf

Add to `~/.codeium/windsurf/mcp_config.json`:

```json
{
  "mcpServers": {
    "acp-workspace": {
      "command": "/path/to/mcp-acp-workspace"
    }
  }
}
```

## Sequence Diagram

<p align="center">
  <img src="https://raw.githubusercontent.com/zavora-ai/mcp-acp-workspace/main/docs/sequence-diagram.svg" alt="ACP Session Sequence Diagram" width="750"/>
</p>

## Verified Output — ADK-ACP with Gemini 2.5 Flash

Tested end-to-end with `adk-acp` server, real Gemini LLM calls, and tool execution:

```
=== ACP Session Lifecycle ===

1. initialize → capabilities received
   { "streaming": true, "tool_use": true, "tool_names": ["read_file", "list_directory"] }

2. session/create → session_id: 6f519835-3217-4021-90a2-f63a1683e774

3. session/prompt ("List the files in the current directory")
   Tool call: list_directory({})
   Agent: I found 6 entries in the current directory:
     • Cargo.toml (file)
     • target (directory)
     • Cargo.lock (file)
     • README.md (file)
     • .env.example (file)
     • src (directory)

4. session/close → ✓
```

## Quick Start

```
> create_acp_session(agent_name: "claude-code", transport: "stdio", command: "claude", workspace: "/my-project")

{ "session_id": "sess_abc...", "status": "active", "protocol_version": "0.1" }

> send_acp_prompt(session_id, "Refactor auth module to use JWT tokens")

{ "status": "sent", "message": "Use stream_acp_events to get responses." }

> request_acp_permission(session_id, permission_type: "file_write", path: "src/auth.rs", decision: "approve")

{ "decision": "approved", "path": "src/auth.rs" }

> get_acp_patch(session_id)

{ "patches": [{ "id": "patch_...", "files": [{ "path": "src/auth.rs", "operation": "modify" }] }] }

> review_acp_patch(session_id, patch_id, decision: "approve")

{ "status": "approved", "files_affected": 1 }

> expose_adk_agent_as_acp(agent_name: "code-review-agent", acp_server_name: "adk-reviewer", transport: "http")

{ "status": "serving", "acp_server_name": "adk-reviewer" }

> terminate_acp_session(session_id)

{ "status": "cancelled", "events_captured": 5, "note": "session/cancel sent as notification" }
```

## Security Model

```
Coding agent requests permission → Governance policy evaluation → Approve/Deny → Audit logged
```

- File writes require explicit permission mapped to governance policy
- Shell commands require permission with command allowlist
- Network access gated separately
- Workspace trust boundaries enforced (can't write outside workspace)
- All sessions produce audit traces
- Patches reviewable before application

## Documentation

| Document | Description |
|----------|-------------|
| [API Reference](docs/api-reference.md) | All 9 tools with parameters and returns |
| [Architecture](docs/architecture.svg) | System diagram |
| [Sequence Diagram](docs/sequence-diagram.svg) | Full ACP session lifecycle |
| [ACP Protocol](https://agentclientprotocol.com/) | Official ACP specification |

## Design Decisions

- **`session/cancel` is a notification** — no response expected on the ACP wire. Platform returns its own ack.
- **`session/update` payload preserved** — full `params.update` shape stored, not flattened.
- **Patches are platform-derived** — aggregated from tool-call content in session/update events. Not a core ACP concept.
- **Capabilities negotiated** — omitted capabilities treated as unsupported per ACP spec.
- **Session lifecycle** — supports `new`, `load` (replays history), `resume` (does not), and `close`.

## MCP Server Manifest

```toml
server_id = "mcp_acp_workspace"
display_name = "ACP Workspace MCP"
version = "1.0.0"
domain = "protocol"
risk_level = "high"
writes_allowed = "gated"
transports = ["stdio"]
governance_gates = ["permission_gated", "patch_review_required", "audit_all_sessions"]
```

## Contributors

<!-- ALL-CONTRIBUTORS-LIST:START -->
| [<img src="https://github.com/jkmaina.png" width="80px;" alt=""/><br /><sub><b>James Karanja Maina</b></sub>](https://github.com/jkmaina) |
|:---:|
<!-- ALL-CONTRIBUTORS-LIST:END -->

## License

Apache-2.0 — see [LICENSE](LICENSE) for details.

---

Part of the [ADK-Rust Enterprise](https://enterprise.adk-rust.com) MCP server ecosystem.
