use crate::store::AcpStore;
use crate::types::*;
use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateAcpSessionInput {
    /// Name of the ACP agent to connect to
    pub agent_name: String,
    /// Transport: stdio, http, or websocket
    pub transport: AcpTransport,
    /// Command for stdio transport (e.g. "claude-code", "codex")
    pub command: Option<String>,
    /// URL for http/websocket transport
    pub url: Option<String>,
    /// Workspace directory path
    pub workspace: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SendAcpPromptInput {
    pub session_id: String,
    /// Task to delegate (sent as ContentBlock[] with type:text)
    pub prompt: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct StreamAcpEventsInput {
    pub session_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RequestAcpPermissionInput {
    pub session_id: String,
    pub permission_type: PermissionType,
    pub path: Option<String>,
    pub command: Option<String>,
    /// approve or deny
    pub decision: String,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetAcpPatchInput {
    pub session_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ReviewAcpPatchInput {
    pub session_id: String,
    pub patch_id: String,
    /// approve, reject, or applied
    pub decision: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TerminateAcpSessionInput {
    pub session_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ExposeAdkAgentAsAcpInput {
    /// ADK-Rust agent name to expose
    pub agent_name: String,
    /// Name for the ACP server
    pub acp_server_name: String,
    /// Transport to serve on
    pub transport: AcpTransport,
    /// Bind address for http/websocket (e.g. "0.0.0.0:9000")
    pub bind_addr: Option<String>,
}

#[derive(Clone)]
pub struct AcpServer {
    pub store: std::sync::Arc<AcpStore>,
}

#[tool_router(server_handler)]
impl AcpServer {
    #[tool(description = "Show registered ACP clients and servers")]
    fn list_acp_connections(&self) -> String {
        let conns = self.store.list_connections();
        let exposed = self.store.list_exposed_agents();
        serde_json::to_string_pretty(&serde_json::json!({
            "connections": conns.iter().map(|c| serde_json::json!({
                "id": c.id, "name": c.name, "type": c.connection_type,
                "transport": c.transport, "status": c.status, "workspace": c.workspace,
            })).collect::<Vec<_>>(),
            "exposed_agents": exposed.iter().map(|e| serde_json::json!({
                "id": e.id, "agent_name": e.agent_name, "acp_server_name": e.acp_server_name,
                "transport": e.transport, "status": e.status,
            })).collect::<Vec<_>>(),
        })).unwrap()
    }

    #[tool(description = "Start ACP session in a workspace (sends initialize + session/new over ACP wire)")]
    fn create_acp_session(&self, Parameters(i): Parameters<CreateAcpSessionInput>) -> String {
        // Register connection (simulates ACP initialize with version negotiation)
        let conn = self.store.register_connection(
            i.agent_name, ConnectionType::Client, i.transport,
            i.command, i.url, Some(i.workspace.clone()),
        );

        // Create session (simulates ACP session/new)
        match self.store.create_session(&conn.id, &i.workspace) {
            Ok(session) => serde_json::to_string_pretty(&serde_json::json!({
                "session_id": session.id,
                "connection_id": conn.id,
                "workspace": session.workspace,
                "status": "active",
                "protocol_version": conn.protocol_version,
                "capabilities": conn.capabilities,
                "message": "ACP session started. Use send_acp_prompt to delegate tasks.",
            })).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Delegate task to ACP-compatible agent (sends session/prompt with ContentBlock[])")]
    fn send_acp_prompt(&self, Parameters(i): Parameters<SendAcpPromptInput>) -> String {
        match self.store.send_prompt(&i.session_id, &i.prompt) {
            Ok(event) => serde_json::to_string_pretty(&serde_json::json!({
                "event_id": event.id,
                "method": "session/prompt",
                "prompt_sent": i.prompt,
                "status": "sent",
                "message": "Prompt sent. Use stream_acp_events to get session/update responses.",
            })).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Get streaming session/update events (preserves full ACP params.update payload)")]
    fn stream_acp_events(&self, Parameters(i): Parameters<StreamAcpEventsInput>) -> String {
        match self.store.get_events(&i.session_id) {
            Ok(events) => serde_json::to_string_pretty(&serde_json::json!({
                "session_id": i.session_id,
                "events": events.iter().map(|e| serde_json::json!({
                    "id": e.id, "type": e.event_type, "payload": e.payload, "timestamp": e.timestamp,
                })).collect::<Vec<_>>(),
            })).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Handle permission request from coding agent — evaluate against governance policy")]
    fn request_acp_permission(&self, Parameters(i): Parameters<RequestAcpPermissionInput>) -> String {
        let decision = match i.decision.as_str() {
            "approve" | "approved" => PermissionDecision::Approved,
            _ => PermissionDecision::Denied,
        };

        // First register the permission request
        let perm = self.store.add_permission_request(&i.session_id, i.permission_type, i.path, i.command);
        match perm {
            Ok(p) => {
                // Then decide it
                match self.store.decide_permission(&i.session_id, &p.id, decision.clone(), i.reason) {
                    Ok(decided) => serde_json::to_string_pretty(&serde_json::json!({
                        "permission_id": decided.id,
                        "type": decided.permission_type,
                        "path": decided.path,
                        "command": decided.command,
                        "decision": decided.decision,
                        "reason": decided.reason,
                    })).unwrap(),
                    Err(e) => format!("Error: {}", e),
                }
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Retrieve proposed file changes from session (platform-derived aggregation from tool-call content, not core ACP)")]
    fn get_acp_patch(&self, Parameters(i): Parameters<GetAcpPatchInput>) -> String {
        match self.store.get_patches(&i.session_id) {
            Ok(patches) => serde_json::to_string_pretty(&serde_json::json!({
                "session_id": i.session_id,
                "patches": patches.iter().map(|p| serde_json::json!({
                    "id": p.id, "status": p.status,
                    "files": p.files.iter().map(|f| serde_json::json!({
                        "path": f.path, "operation": f.operation, "diff_lines": f.diff.lines().count(),
                    })).collect::<Vec<_>>(),
                    "review_notes": p.review_notes,
                })).collect::<Vec<_>>(),
                "note": "Patches are platform-derived aggregations from tool-call content in session/update events.",
            })).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Approve, reject, or annotate a proposed patch")]
    fn review_acp_patch(&self, Parameters(i): Parameters<ReviewAcpPatchInput>) -> String {
        let status = match i.decision.as_str() {
            "approve" | "approved" => PatchStatus::Approved,
            "applied" => PatchStatus::Applied,
            _ => PatchStatus::Rejected,
        };
        match self.store.review_patch(&i.session_id, &i.patch_id, status, i.notes) {
            Ok(patch) => serde_json::to_string_pretty(&serde_json::json!({
                "patch_id": patch.id,
                "status": patch.status,
                "review_notes": patch.review_notes,
                "files_affected": patch.files.len(),
            })).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    /// session/cancel is a notification on the ACP wire (no id, no response expected).
    /// The platform layer records the cancellation and returns its own acknowledgment.
    #[tool(description = "Stop ACP session and capture trace (sends session/cancel notification on wire)")]
    fn terminate_acp_session(&self, Parameters(i): Parameters<TerminateAcpSessionInput>) -> String {
        match self.store.terminate_session(&i.session_id) {
            Ok(session) => serde_json::to_string_pretty(&serde_json::json!({
                "session_id": session.id,
                "status": "cancelled",
                "events_captured": session.events.len(),
                "permissions_total": session.permissions.len(),
                "patches_total": session.patches.len(),
                "note": "session/cancel sent as notification (no wire response). Platform trace captured.",
            })).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Publish ADK-Rust agent as ACP-compatible server (usable from VS Code, Claude Desktop)")]
    fn expose_adk_agent_as_acp(&self, Parameters(i): Parameters<ExposeAdkAgentAsAcpInput>) -> String {
        let agent = self.store.expose_agent(&i.agent_name, &i.acp_server_name, i.transport, i.bind_addr);
        serde_json::to_string_pretty(&serde_json::json!({
            "id": agent.id,
            "agent_name": agent.agent_name,
            "acp_server_name": agent.acp_server_name,
            "transport": agent.transport,
            "bind_addr": agent.bind_addr,
            "protocol_version": agent.protocol_version,
            "status": "serving",
            "message": "ADK-Rust agent now accessible as ACP server.",
        })).unwrap()
    }
}
