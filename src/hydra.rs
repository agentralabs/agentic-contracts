//! Hydra integration placeholder traits.
//!
//! These traits define how sisters connect to the Hydra orchestrator.
//! They are PLACEHOLDERS — the real implementations will come when
//! Hydra is built. For now, they establish the contract shape so
//! sisters can prepare.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │                  HYDRA                   │
//! │  ┌─────────┐ ┌──────────┐ ┌──────────┐  │
//! │  │Execution│ │Capability│ │ Receipt  │  │
//! │  │  Gate   │ │ Engine   │ │ Ledger   │  │
//! │  └────┬────┘ └────┬─────┘ └────┬─────┘  │
//! │       │           │            │         │
//! │  ┌────┴───────────┴────────────┴──────┐  │
//! │  │         HydraBridge trait          │  │
//! │  └────────────────────────────────────┘  │
//! └───────────────┬───────────────────────────┘
//!                 │
//!    ┌────────────┼────────────┐
//!    ▼            ▼            ▼
//! Memory       Vision      Codebase  ...
//! ```

use crate::context::SessionContext;
use crate::errors::SisterResult;
use crate::types::{Metadata, SisterType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════
// HYDRA BRIDGE — How sisters connect to Hydra
// ═══════════════════════════════════════════════════════════════════

/// Summary of a sister's current state (for Hydra's context window).
///
/// This is the token-efficient summary Hydra uses to understand
/// what each sister is doing without loading full state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SisterSummary {
    /// Which sister
    pub sister_type: SisterType,

    /// Brief status line for LLM context
    pub status_line: String,

    /// Item count (memories, captures, nodes, etc.)
    pub item_count: usize,

    /// Active session/workspace name
    pub active_context: Option<String>,

    /// Additional metadata
    #[serde(default)]
    pub metadata: Metadata,
}

/// A command from Hydra to a sister
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HydraCommand {
    /// Command type (sister interprets this)
    pub command_type: String,

    /// Command parameters
    #[serde(default)]
    pub params: Metadata,

    /// Hydra run ID (for receipt chain)
    pub run_id: String,

    /// Step ID within the run
    pub step_id: u64,
}

/// Result of executing a Hydra command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    /// Whether the command succeeded
    pub success: bool,

    /// Result data
    pub data: serde_json::Value,

    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Evidence IDs produced by this command
    #[serde(default)]
    pub evidence_ids: Vec<String>,
}

/// The bridge between Hydra and individual sisters.
///
/// This is a PLACEHOLDER trait. Sisters should not implement it yet.
/// It establishes the expected contract shape for when Hydra arrives.
///
/// When Hydra is built, this trait will require:
/// `Sister + SessionManagement/WorkspaceManagement + Grounding + EventEmitter + Queryable`
pub trait HydraBridge {
    /// Get a token-efficient summary of current sister state.
    /// Hydra calls this to build its context window
    fn session_context(&self) -> SisterResult<SessionContext>;

    /// Restore sister state from a previous session context.
    /// Used when Hydra resumes a run
    fn restore_session(&mut self, context: SessionContext) -> SisterResult<()>;

    /// Get a brief summary for Hydra's context
    fn summary(&self) -> SisterResult<SisterSummary>;

    /// Execute a command from Hydra.
    /// This is the escape hatch for Hydra-specific operations
    fn execute(&mut self, command: HydraCommand) -> SisterResult<CommandResult>;
}

// ═══════════════════════════════════════════════════════════════════
// EXECUTION GATE — Hydra's safety core (placeholder types)
// ═══════════════════════════════════════════════════════════════════

/// Risk level for an action
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    /// Low risk (0.0-0.3): auto-approve
    Low,

    /// Medium risk (0.3-0.6): log and proceed
    Medium,

    /// High risk (0.6-0.8): require confirmation
    High,

    /// Critical risk (0.8-1.0): block and escalate
    Critical,
}

/// An action that needs to pass through the execution gate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatedAction {
    /// What sister is requesting this action
    pub sister_type: SisterType,

    /// Action type
    pub action_type: String,

    /// Assessed risk level
    pub risk_level: RiskLevel,

    /// Risk score (0.0-1.0)
    pub risk_score: f64,

    /// Required capability
    pub capability: String,

    /// When the action was requested
    pub requested_at: DateTime<Utc>,

    /// Action parameters
    #[serde(default)]
    pub params: Metadata,
}

/// Result of passing through the execution gate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateDecision {
    /// Whether the action is approved
    pub approved: bool,

    /// Reason for the decision
    pub reason: String,

    /// Approval ID (for receipt chain)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_id: Option<String>,

    /// Conditions imposed on the action
    #[serde(default)]
    pub conditions: Vec<String>,
}

/// The Execution Gate trait (placeholder).
///
/// Hydra implements this, NOT sisters. Sisters submit actions
/// to the gate; Hydra decides whether to approve.
pub trait ExecutionGate {
    /// Submit an action for approval
    fn check(&self, action: GatedAction) -> SisterResult<GateDecision>;

    /// Quick check if a capability is available
    fn has_capability(&self, capability: &str) -> bool;

    /// Get current risk threshold
    fn risk_threshold(&self) -> RiskLevel;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::Low < RiskLevel::Medium);
        assert!(RiskLevel::Medium < RiskLevel::High);
        assert!(RiskLevel::High < RiskLevel::Critical);
    }

    #[test]
    fn test_sister_summary() {
        let summary = SisterSummary {
            sister_type: SisterType::Memory,
            status_line: "590 nodes, session 42 active".into(),
            item_count: 590,
            active_context: Some("session_42".into()),
            metadata: Metadata::new(),
        };

        assert_eq!(summary.sister_type, SisterType::Memory);
        assert_eq!(summary.item_count, 590);
    }

    #[test]
    fn test_command_result() {
        let result = CommandResult {
            success: true,
            data: serde_json::json!({"added": 5}),
            error: None,
            evidence_ids: vec!["ev_1".into()],
        };

        assert!(result.success);
        assert_eq!(result.evidence_ids.len(), 1);
    }

    #[test]
    fn test_gate_decision() {
        let decision = GateDecision {
            approved: true,
            reason: "Low risk action, auto-approved".into(),
            approval_id: Some("approval_123".into()),
            conditions: vec![],
        };

        assert!(decision.approved);
    }
}
