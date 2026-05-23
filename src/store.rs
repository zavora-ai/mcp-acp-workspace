use crate::types::*;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

pub struct AcpStore {
    connections: Mutex<HashMap<String, AcpConnection>>,
    sessions: Mutex<HashMap<String, AcpSession>>,
    exposed_agents: Mutex<Vec<ExposedAgent>>,
}

impl AcpStore {
    pub fn new() -> Self {
        Self {
            connections: Mutex::new(HashMap::new()),
            sessions: Mutex::new(HashMap::new()),
            exposed_agents: Mutex::new(Vec::new()),
        }
    }

    pub fn list_connections(&self) -> Vec<AcpConnection> {
        self.connections.lock().unwrap().values().cloned().collect()
    }

    pub fn register_connection(
        &self, name: String, connection_type: ConnectionType, transport: AcpTransport,
        command: Option<String>, url: Option<String>, workspace: Option<String>,
    ) -> AcpConnection {
        let conn = AcpConnection {
            id: format!("acp_{}", Uuid::new_v4().simple()),
            name, connection_type, transport, command, url,
            protocol_version: "0.1".to_string(),
            capabilities: AcpCapabilities::default(),
            status: ConnectionStatus::Connected,
            workspace,
            created_at: Utc::now(),
        };
        self.connections.lock().unwrap().insert(conn.id.clone(), conn.clone());
        conn
    }

    pub fn create_session(&self, connection_id: &str, workspace: &str) -> Result<AcpSession, String> {
        let conns = self.connections.lock().unwrap();
        if !conns.contains_key(connection_id) {
            return Err(format!("Connection not found: {}", connection_id));
        }
        drop(conns);

        let session = AcpSession {
            id: format!("sess_{}", Uuid::new_v4().simple()),
            connection_id: connection_id.to_string(),
            workspace: workspace.to_string(),
            status: SessionStatus::Active,
            permissions: Vec::new(),
            patches: Vec::new(),
            events: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.sessions.lock().unwrap().insert(session.id.clone(), session.clone());
        Ok(session)
    }

    pub fn send_prompt(&self, session_id: &str, prompt: &str) -> Result<SessionEvent, String> {
        let mut sessions = self.sessions.lock().unwrap();
        let session = sessions.get_mut(session_id).ok_or_else(|| format!("Session not found: {}", session_id))?;
        if session.status != SessionStatus::Active {
            return Err(format!("Session not active: {:?}", session.status));
        }

        // Record the prompt as an event (in production, this sends session/prompt over ACP wire)
        let event = SessionEvent {
            id: format!("evt_{}", Uuid::new_v4().simple()),
            event_type: "session/prompt".to_string(),
            payload: serde_json::json!({
                "method": "session/prompt",
                "params": {
                    "prompt": [{"type": "text", "text": prompt}]
                }
            }),
            timestamp: Utc::now(),
        };
        session.events.push(event.clone());
        session.updated_at = Utc::now();
        Ok(event)
    }

    pub fn get_events(&self, session_id: &str) -> Result<Vec<SessionEvent>, String> {
        let sessions = self.sessions.lock().unwrap();
        let session = sessions.get(session_id).ok_or_else(|| format!("Session not found: {}", session_id))?;
        Ok(session.events.clone())
    }

    pub fn add_permission_request(
        &self, session_id: &str, permission_type: PermissionType,
        path: Option<String>, command: Option<String>,
    ) -> Result<Permission, String> {
        let mut sessions = self.sessions.lock().unwrap();
        let session = sessions.get_mut(session_id).ok_or_else(|| format!("Session not found: {}", session_id))?;

        let perm = Permission {
            id: format!("perm_{}", Uuid::new_v4().simple()),
            permission_type, path, command,
            decision: PermissionDecision::Pending,
            reason: None,
            decided_at: None,
        };
        session.permissions.push(perm.clone());
        session.updated_at = Utc::now();
        Ok(perm)
    }

    pub fn decide_permission(&self, session_id: &str, permission_id: &str, decision: PermissionDecision, reason: Option<String>) -> Result<Permission, String> {
        let mut sessions = self.sessions.lock().unwrap();
        let session = sessions.get_mut(session_id).ok_or_else(|| format!("Session not found: {}", session_id))?;
        let perm = session.permissions.iter_mut().find(|p| p.id == permission_id)
            .ok_or_else(|| format!("Permission not found: {}", permission_id))?;
        perm.decision = decision;
        perm.reason = reason;
        perm.decided_at = Some(Utc::now());
        session.updated_at = Utc::now();
        Ok(perm.clone())
    }

    pub fn add_patch(&self, session_id: &str, files: Vec<FileChange>) -> Result<Patch, String> {
        let mut sessions = self.sessions.lock().unwrap();
        let session = sessions.get_mut(session_id).ok_or_else(|| format!("Session not found: {}", session_id))?;
        let patch = Patch {
            id: format!("patch_{}", Uuid::new_v4().simple()),
            session_id: session_id.to_string(),
            files, status: PatchStatus::Proposed,
            review_notes: None, created_at: Utc::now(),
        };
        session.patches.push(patch.clone());
        session.updated_at = Utc::now();
        Ok(patch)
    }

    pub fn get_patches(&self, session_id: &str) -> Result<Vec<Patch>, String> {
        let sessions = self.sessions.lock().unwrap();
        let session = sessions.get(session_id).ok_or_else(|| format!("Session not found: {}", session_id))?;
        Ok(session.patches.clone())
    }

    pub fn review_patch(&self, session_id: &str, patch_id: &str, status: PatchStatus, notes: Option<String>) -> Result<Patch, String> {
        let mut sessions = self.sessions.lock().unwrap();
        let session = sessions.get_mut(session_id).ok_or_else(|| format!("Session not found: {}", session_id))?;
        let patch = session.patches.iter_mut().find(|p| p.id == patch_id)
            .ok_or_else(|| format!("Patch not found: {}", patch_id))?;
        patch.status = status;
        patch.review_notes = notes;
        session.updated_at = Utc::now();
        Ok(patch.clone())
    }

    /// session/cancel is a notification on the ACP wire (no response expected).
    /// The platform layer records the cancellation and returns its own ack.
    pub fn terminate_session(&self, session_id: &str) -> Result<AcpSession, String> {
        let mut sessions = self.sessions.lock().unwrap();
        let session = sessions.get_mut(session_id).ok_or_else(|| format!("Session not found: {}", session_id))?;
        session.status = SessionStatus::Cancelled;
        session.events.push(SessionEvent {
            id: format!("evt_{}", Uuid::new_v4().simple()),
            event_type: "session/cancel".to_string(),
            payload: serde_json::json!({"method": "notifications/session/cancel", "params": {"sessionId": session_id}}),
            timestamp: Utc::now(),
        });
        session.updated_at = Utc::now();
        Ok(session.clone())
    }

    pub fn expose_agent(&self, agent_name: &str, acp_server_name: &str, transport: AcpTransport, bind_addr: Option<String>) -> ExposedAgent {
        let agent = ExposedAgent {
            id: format!("exposed_{}", Uuid::new_v4().simple()),
            agent_name: agent_name.to_string(),
            acp_server_name: acp_server_name.to_string(),
            transport, bind_addr,
            protocol_version: "0.1".to_string(),
            status: ConnectionStatus::Connected,
            created_at: Utc::now(),
        };
        self.exposed_agents.lock().unwrap().push(agent.clone());
        agent
    }

    pub fn list_exposed_agents(&self) -> Vec<ExposedAgent> {
        self.exposed_agents.lock().unwrap().clone()
    }
}
