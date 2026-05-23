use chrono::{DateTime, Utc};
use rmcp::schemars;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionType {
    Client,
    Server,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AcpTransport {
    Stdio,
    Http,
    WebSocket,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Active,
    Completed,
    Cancelled,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PermissionType {
    FileWrite,
    FileRead,
    Command,
    Network,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PermissionDecision {
    Pending,
    Approved,
    Denied,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PatchStatus {
    Proposed,
    Approved,
    Rejected,
    Applied,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FileOperation {
    Create,
    Modify,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpConnection {
    pub id: String,
    pub name: String,
    pub connection_type: ConnectionType,
    pub transport: AcpTransport,
    pub command: Option<String>,
    pub url: Option<String>,
    pub protocol_version: String,
    pub capabilities: AcpCapabilities,
    pub status: ConnectionStatus,
    pub workspace: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpCapabilities {
    pub prompts: PromptCapabilities,
    pub tools: bool,
    pub filesystem: bool,
    pub commands: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptCapabilities {
    pub text: bool,
    pub image: bool,
    pub file: bool,
}

impl Default for AcpCapabilities {
    fn default() -> Self {
        Self {
            prompts: PromptCapabilities { text: true, image: false, file: true },
            tools: true,
            filesystem: true,
            commands: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpSession {
    pub id: String,
    pub connection_id: String,
    pub workspace: String,
    pub status: SessionStatus,
    pub permissions: Vec<Permission>,
    pub patches: Vec<Patch>,
    pub events: Vec<SessionEvent>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: String,
    pub permission_type: PermissionType,
    pub path: Option<String>,
    pub command: Option<String>,
    pub decision: PermissionDecision,
    pub reason: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
}

/// Platform-derived patch aggregation from tool-call content in session/update events.
/// This is NOT a core ACP concept — it's an enterprise convenience for reviewing
/// file changes proposed by coding agents during a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patch {
    pub id: String,
    pub session_id: String,
    pub files: Vec<FileChange>,
    pub status: PatchStatus,
    pub review_notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub operation: FileOperation,
    pub diff: String,
}

/// Stores the full ACP session/update notification payload (preserving params.update shape)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvent {
    pub id: String,
    pub event_type: String,
    /// Full ACP notification payload as received on the wire
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposedAgent {
    pub id: String,
    pub agent_name: String,
    pub acp_server_name: String,
    pub transport: AcpTransport,
    pub bind_addr: Option<String>,
    pub protocol_version: String,
    pub status: ConnectionStatus,
    pub created_at: DateTime<Utc>,
}
