# API Reference

## list_acp_connections

Show registered ACP clients and exposed agents.

**Parameters:** None.

**Returns:**
```json
{
  "connections": [{ "id": "acp_...", "name": "claude-code", "type": "client", "transport": "stdio", "status": "connected" }],
  "exposed_agents": [{ "id": "exposed_...", "agent_name": "code-review-agent", "acp_server_name": "adk-reviewer", "status": "connected" }]
}
```

---

## create_acp_session

Start an ACP session. Sends `initialize` (version negotiation) then `session/new` (workspace binding) on the ACP wire.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `agent_name` | string | Yes | ACP agent to connect to |
| `transport` | enum | Yes | `stdio`, `http`, or `websocket` |
| `command` | string | No | Command for stdio (e.g. "claude") |
| `url` | string | No | URL for http/websocket |
| `workspace` | string | Yes | Workspace directory path |

**Returns:**
```json
{ "session_id": "sess_...", "connection_id": "acp_...", "workspace": "/my-project", "status": "active", "protocol_version": "0.1" }
```

---

## send_acp_prompt

Delegate a task to the coding agent. Sends `session/prompt` with `ContentBlock[]` on the ACP wire.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `session_id` | string | Yes | Active session |
| `prompt` | string | Yes | Task description |

**Returns:**
```json
{ "event_id": "evt_...", "method": "session/prompt", "status": "sent" }
```

---

## stream_acp_events

Get session/update events. Preserves full ACP `params.update` payload shape.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `session_id` | string | Yes | Session to query |

**Returns:**
```json
{ "session_id": "sess_...", "events": [{ "id": "evt_...", "type": "session/prompt", "payload": {...}, "timestamp": "..." }] }
```

---

## request_acp_permission

Handle a permission request from the coding agent. Evaluates against governance policy.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `session_id` | string | Yes | Session |
| `permission_type` | enum | Yes | `file_write`, `file_read`, `command`, `network` |
| `path` | string | No | File path (for file permissions) |
| `command` | string | No | Command (for command permissions) |
| `decision` | string | Yes | `approve` or `deny` |
| `reason` | string | No | Justification |

**Returns:**
```json
{ "permission_id": "perm_...", "type": "file_write", "path": "src/auth.rs", "decision": "approved", "reason": "Approved for auth refactor" }
```

---

## get_acp_patch

Retrieve proposed file changes. Platform-derived aggregation from tool-call content in `session/update` events — not a core ACP concept.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `session_id` | string | Yes | Session to query |

**Returns:**
```json
{ "session_id": "sess_...", "patches": [{ "id": "patch_...", "status": "proposed", "files": [{ "path": "src/auth.rs", "operation": "modify" }] }] }
```

---

## review_acp_patch

Approve, reject, or annotate a proposed patch.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `session_id` | string | Yes | Session |
| `patch_id` | string | Yes | Patch to review |
| `decision` | string | Yes | `approve`, `reject`, or `applied` |
| `notes` | string | No | Review notes |

**Returns:**
```json
{ "patch_id": "patch_...", "status": "approved", "review_notes": "LGTM", "files_affected": 1 }
```

---

## terminate_acp_session

Stop session. Sends `session/cancel` as a notification on the ACP wire (no response expected). Platform captures trace.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `session_id` | string | Yes | Session to terminate |

**Returns:**
```json
{ "session_id": "sess_...", "status": "cancelled", "events_captured": 5, "permissions_total": 2, "patches_total": 1, "note": "session/cancel sent as notification. Platform trace captured." }
```

---

## expose_adk_agent_as_acp

Publish an ADK-Rust agent as an ACP-compatible server, making it usable from VS Code, Claude Desktop, or any ACP client.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `agent_name` | string | Yes | ADK-Rust agent to expose |
| `acp_server_name` | string | Yes | Name for the ACP server |
| `transport` | enum | Yes | `stdio`, `http`, or `websocket` |
| `bind_addr` | string | No | Bind address for http/websocket |

**Returns:**
```json
{ "id": "exposed_...", "agent_name": "code-review-agent", "acp_server_name": "adk-reviewer", "transport": "http", "bind_addr": "0.0.0.0:9000", "status": "serving" }
```
