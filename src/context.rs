//! Context management traits for all sisters.
//!
//! v0.2.0 splits the old monolithic `ContextManagement` into two traits
//! that match how sisters actually work:
//!
//! - **SessionManagement**: Append-only sequential sessions (Memory, Vision, Identity)
//! - **WorkspaceManagement**: Switchable, named workspaces (Codebase)
//!
//! Sisters implement whichever fits. Time implements neither (stateless).
//! Hydra can query both via the unified `ContextInfo` type.

use crate::errors::SisterResult;
use crate::types::{Metadata, SisterType, UniqueId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Unique identifier for a context (session or workspace).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContextId(pub UniqueId);

impl ContextId {
    /// Create a new random context ID
    pub fn new() -> Self {
        Self(UniqueId::new())
    }

    /// The default context (always exists)
    pub fn default_context() -> Self {
        Self(UniqueId::nil())
    }

    /// Check if this is the default context
    pub fn is_default(&self) -> bool {
        self.0 == UniqueId::nil()
    }
}

impl Default for ContextId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ContextId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ctx_{}", self.0)
    }
}

impl From<&str> for ContextId {
    fn from(s: &str) -> Self {
        let s = s.strip_prefix("ctx_").unwrap_or(s);
        if let Ok(uuid) = uuid::Uuid::parse_str(s) {
            Self(UniqueId::from_uuid(uuid))
        } else {
            Self::new()
        }
    }
}

/// Summary information about a context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSummary {
    pub id: ContextId,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub item_count: usize,
    pub size_bytes: usize,
}

/// Full context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextInfo {
    pub id: ContextId,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub item_count: usize,
    pub size_bytes: usize,
    #[serde(default)]
    pub metadata: Metadata,
}

impl From<ContextInfo> for ContextSummary {
    fn from(info: ContextInfo) -> Self {
        Self {
            id: info.id,
            name: info.name,
            created_at: info.created_at,
            updated_at: info.updated_at,
            item_count: info.item_count,
            size_bytes: info.size_bytes,
        }
    }
}

/// Exportable context snapshot (for backup/transfer)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnapshot {
    /// Which sister type this came from
    pub sister_type: SisterType,

    /// Version of the sister that created this
    pub version: crate::types::Version,

    /// Context information
    pub context_info: ContextInfo,

    /// Serialized context data (sister-specific format)
    #[serde(with = "base64_serde")]
    pub data: Vec<u8>,

    /// Checksum of the data (BLAKE3)
    #[serde(with = "hex_serde")]
    pub checksum: [u8; 32],

    /// When this snapshot was created
    pub snapshot_at: DateTime<Utc>,
}

impl ContextSnapshot {
    /// Verify the checksum
    pub fn verify(&self) -> bool {
        let computed = blake3::hash(&self.data);
        computed.as_bytes() == &self.checksum
    }
}

// ═══════════════════════════════════════════════════════════════════
// SESSION MANAGEMENT — Append-only sequential sessions
// ═══════════════════════════════════════════════════════════════════

/// Session management for sisters with append-only sequential sessions.
///
/// Used by: Memory (sessions), Vision (sessions), Identity (chains)
///
/// Key difference from WorkspaceManagement:
/// - Sessions are sequential — you don't "switch back" to an old session
/// - Sessions are append-only — you can't delete past sessions
/// - The current session is always the latest one
///
/// NOT used by: Time (stateless), Codebase (uses WorkspaceManagement)
pub trait SessionManagement {
    /// Start a new session. Returns the session ID.
    /// The previous session (if any) is automatically ended
    fn start_session(&mut self, name: &str) -> SisterResult<ContextId>;

    /// Start a new session with metadata
    fn start_session_with_metadata(
        &mut self,
        name: &str,
        metadata: Metadata,
    ) -> SisterResult<ContextId> {
        let _ = metadata;
        self.start_session(name)
    }

    /// End the current session.
    /// After this, a new session must be started before operations
    fn end_session(&mut self) -> SisterResult<()>;

    /// Get the current session ID.
    /// Returns None if no session is active
    fn current_session(&self) -> Option<ContextId>;

    /// Get info about the current session
    fn current_session_info(&self) -> SisterResult<ContextInfo>;

    /// List all past sessions (most recent first)
    fn list_sessions(&self) -> SisterResult<Vec<ContextSummary>>;

    /// Get info about a specific past session
    fn get_session_info(&self, id: ContextId) -> SisterResult<ContextInfo> {
        self.list_sessions()?
            .into_iter()
            .find(|s| s.id == id)
            .map(|summary| ContextInfo {
                id: summary.id,
                name: summary.name,
                created_at: summary.created_at,
                updated_at: summary.updated_at,
                item_count: summary.item_count,
                size_bytes: summary.size_bytes,
                metadata: Metadata::new(),
            })
            .ok_or_else(|| crate::errors::SisterError::context_not_found(id.to_string()))
    }

    /// Export a session as a snapshot (for backup/transfer)
    fn export_session(&self, id: ContextId) -> SisterResult<ContextSnapshot>;

    /// Import a session from a snapshot
    fn import_session(&mut self, snapshot: ContextSnapshot) -> SisterResult<ContextId>;
}

// ═══════════════════════════════════════════════════════════════════
// WORKSPACE MANAGEMENT — Switchable named workspaces
// ═══════════════════════════════════════════════════════════════════

/// Workspace management for sisters with switchable, named contexts.
///
/// Used by: Codebase (workspaces/graphs)
///
/// Key difference from SessionManagement:
/// - Workspaces are concurrent — you can switch between them
/// - Workspaces can be created, renamed, and deleted
/// - Multiple workspaces exist simultaneously
pub trait WorkspaceManagement {
    /// Create a new workspace
    fn create_workspace(&mut self, name: &str) -> SisterResult<ContextId>;

    /// Create a new workspace with metadata
    fn create_workspace_with_metadata(
        &mut self,
        name: &str,
        metadata: Metadata,
    ) -> SisterResult<ContextId> {
        let _ = metadata;
        self.create_workspace(name)
    }

    /// Switch to a different workspace
    fn switch_workspace(&mut self, id: ContextId) -> SisterResult<()>;

    /// Get the current workspace ID
    fn current_workspace(&self) -> ContextId;

    /// Get info about the current workspace
    fn current_workspace_info(&self) -> SisterResult<ContextInfo>;

    /// List all workspaces
    fn list_workspaces(&self) -> SisterResult<Vec<ContextSummary>>;

    /// Delete a workspace.
    /// Cannot delete the current workspace — switch first
    fn delete_workspace(&mut self, id: ContextId) -> SisterResult<()>;

    /// Rename a workspace
    fn rename_workspace(&mut self, id: ContextId, new_name: &str) -> SisterResult<()>;

    /// Export workspace as snapshot
    fn export_workspace(&self, id: ContextId) -> SisterResult<ContextSnapshot>;

    /// Import workspace from snapshot
    fn import_workspace(&mut self, snapshot: ContextSnapshot) -> SisterResult<ContextId>;

    /// Get workspace info by ID
    fn get_workspace_info(&self, id: ContextId) -> SisterResult<ContextInfo> {
        self.list_workspaces()?
            .into_iter()
            .find(|w| w.id == id)
            .map(|summary| ContextInfo {
                id: summary.id,
                name: summary.name,
                created_at: summary.created_at,
                updated_at: summary.updated_at,
                item_count: summary.item_count,
                size_bytes: summary.size_bytes,
                metadata: Metadata::new(),
            })
            .ok_or_else(|| crate::errors::SisterError::context_not_found(id.to_string()))
    }

    /// Check if a workspace exists
    fn workspace_exists(&self, id: ContextId) -> bool {
        self.get_workspace_info(id).is_ok()
    }
}

/// Session context for Hydra integration (token-efficient summary).
///
/// Works for both session-based and workspace-based sisters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    /// Which sister this is from
    pub sister_type: SisterType,

    /// Current context ID (session or workspace)
    pub context_id: ContextId,

    /// Context name
    pub context_name: String,

    /// Brief summary for LLM context
    pub summary: String,

    /// Recent/relevant items (for quick reference)
    pub recent_items: Vec<String>,

    /// Additional metadata
    #[serde(default)]
    pub metadata: Metadata,
}

// Base64 serialization for binary data
mod base64_serde {
    use base64::{engine::general_purpose::STANDARD, Engine};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&STANDARD.encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        STANDARD.decode(&s).map_err(serde::de::Error::custom)
    }
}

// Hex serialization for checksums
mod hex_serde {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
        bytes
            .try_into()
            .map_err(|_| serde::de::Error::custom("invalid checksum length"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_id() {
        let id = ContextId::new();
        let s = id.to_string();
        assert!(s.starts_with("ctx_"));

        let default = ContextId::default_context();
        assert!(default.is_default());
    }

    #[test]
    fn test_context_id_from_str() {
        let id = ContextId::new();
        let s = id.to_string();
        let parsed: ContextId = s.as_str().into();
        assert!(!parsed.is_default() || id.is_default());
    }
}
