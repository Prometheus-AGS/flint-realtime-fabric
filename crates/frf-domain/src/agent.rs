use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::{AgentId, SessionId, TenantId};

/// Wire-protocol variant for agent events.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentProtocol {
    AgUi,
    A2a,
    A2ui,
    Custom(String),
}

/// Lifecycle state of an agent event stream.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentEventKind {
    RunStart,
    RunEnd,
    TextDelta,
    ToolCall,
    ToolResult,
    StateSnapshot,
    Error,
    Custom(String),
}

/// Envelope for AG-UI / A2A / A2UI agent protocol events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentEvent {
    pub agent_id: AgentId,
    pub tenant_id: TenantId,
    pub session_id: SessionId,
    pub protocol: AgentProtocol,
    pub kind: AgentEventKind,
    /// Run identifier — groups events in one agent turn.
    pub run_id: String,
    /// Serialized content block (text delta, tool call, etc.).
    pub content: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}
