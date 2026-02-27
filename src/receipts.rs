//! Receipt integration with Identity sister.
//!
//! Identity is the receipt system. All sisters that create auditable
//! actions use Identity for receipts. Hydra queries Identity for receipts.

use crate::context::ContextId;
use crate::errors::SisterResult;
use crate::types::{Metadata, SisterType, UniqueId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Unique receipt identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReceiptId(pub UniqueId);

impl ReceiptId {
    pub fn new() -> Self {
        Self(UniqueId::new())
    }
}

impl Default for ReceiptId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ReceiptId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rcpt_{}", self.0)
    }
}

/// Action outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ActionOutcome {
    /// Action succeeded.
    Success {
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<serde_json::Value>,
    },

    /// Action failed.
    Failure {
        error_code: String,
        error_message: String,
    },

    /// Action partially succeeded.
    Partial {
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<serde_json::Value>,
        warnings: Vec<String>,
    },
}

impl ActionOutcome {
    pub fn success() -> Self {
        Self::Success { result: None }
    }

    pub fn success_with(result: impl Serialize) -> Self {
        Self::Success {
            result: serde_json::to_value(result).ok(),
        }
    }

    pub fn failure(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Failure {
            error_code: code.into(),
            error_message: message.into(),
        }
    }

    pub fn partial(warnings: Vec<String>) -> Self {
        Self::Partial {
            result: None,
            warnings,
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    pub fn is_failure(&self) -> bool {
        matches!(self, Self::Failure { .. })
    }
}

/// Action record to be receipted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRecord {
    /// What sister performed this.
    pub sister_type: SisterType,

    /// What action was performed.
    pub action_type: String,

    /// Action parameters (sanitized - no secrets).
    #[serde(default)]
    pub parameters: Metadata,

    /// Outcome.
    pub outcome: ActionOutcome,

    /// Evidence pointers.
    #[serde(default)]
    pub evidence_ids: Vec<String>,

    /// Context ID where this happened.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_id: Option<ContextId>,

    /// Timestamp.
    pub timestamp: DateTime<Utc>,
}

impl ActionRecord {
    /// Create a new action record.
    pub fn new(
        sister_type: SisterType,
        action_type: impl Into<String>,
        outcome: ActionOutcome,
    ) -> Self {
        Self {
            sister_type,
            action_type: action_type.into(),
            parameters: Metadata::new(),
            outcome,
            evidence_ids: vec![],
            context_id: None,
            timestamp: Utc::now(),
        }
    }

    /// Add a parameter.
    pub fn param(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(v) = serde_json::to_value(value) {
            self.parameters.insert(key.into(), v);
        }
        self
    }

    /// Add evidence.
    pub fn evidence(mut self, evidence_id: impl Into<String>) -> Self {
        self.evidence_ids.push(evidence_id.into());
        self
    }

    /// Set context.
    pub fn in_context(mut self, context_id: ContextId) -> Self {
        self.context_id = Some(context_id);
        self
    }
}

/// A receipt (signed action record).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    /// Receipt ID.
    pub id: ReceiptId,

    /// The action that was recorded.
    pub action: ActionRecord,

    /// Signature (from Identity).
    pub signature: String,

    /// Position in the hash chain.
    pub chain_position: u64,

    /// Hash of previous receipt (for chain integrity).
    pub previous_hash: String,

    /// This receipt's hash.
    pub hash: String,

    /// When the receipt was created.
    pub created_at: DateTime<Utc>,
}

impl Receipt {
    /// Verify the receipt signature (requires Identity).
    /// This is a placeholder - actual verification happens via Identity sister.
    pub fn verify_signature(&self, _public_key: &[u8]) -> bool {
        // In practice, this would use ed25519 verification
        // For now, return true as placeholder
        !self.signature.is_empty()
    }

    /// Get the action type.
    pub fn action_type(&self) -> &str {
        &self.action.action_type
    }

    /// Check if action was successful.
    pub fn was_successful(&self) -> bool {
        self.action.outcome.is_success()
    }
}

/// Filter for querying receipts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReceiptFilter {
    /// Filter by sister type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sister_type: Option<SisterType>,

    /// Filter by action type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_type: Option<String>,

    /// Filter by context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_id: Option<ContextId>,

    /// Filter by time (after).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<DateTime<Utc>>,

    /// Filter by time (before).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<DateTime<Utc>>,

    /// Filter by outcome.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>, // "success", "failure", "partial"

    /// Limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,

    /// Offset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,
}

impl ReceiptFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn for_sister(mut self, sister_type: SisterType) -> Self {
        self.sister_type = Some(sister_type);
        self
    }

    pub fn action(mut self, action_type: impl Into<String>) -> Self {
        self.action_type = Some(action_type.into());
        self
    }

    pub fn in_context(mut self, context_id: ContextId) -> Self {
        self.context_id = Some(context_id);
        self
    }

    pub fn after(mut self, time: DateTime<Utc>) -> Self {
        self.after = Some(time);
        self
    }

    pub fn before(mut self, time: DateTime<Utc>) -> Self {
        self.before = Some(time);
        self
    }

    pub fn successful_only(mut self) -> Self {
        self.outcome = Some("success".to_string());
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// Receipt integration trait.
///
/// Sisters that create auditable actions implement this trait to
/// integrate with Identity for receipt creation.
pub trait ReceiptIntegration {
    /// Create a receipt for an action (via Identity).
    fn create_receipt(&self, action: ActionRecord) -> SisterResult<ReceiptId>;

    /// Get receipt by ID (from Identity).
    fn get_receipt(&self, id: ReceiptId) -> SisterResult<Receipt>;

    /// List receipts for this sister.
    fn list_receipts(&self, filter: ReceiptFilter) -> SisterResult<Vec<Receipt>>;

    /// Get receipt count.
    fn receipt_count(&self) -> SisterResult<u64> {
        self.list_receipts(ReceiptFilter::new())
            .map(|r| r.len() as u64)
    }

    /// Get receipts for a specific action type.
    fn receipts_for_action(&self, action_type: &str) -> SisterResult<Vec<Receipt>> {
        self.list_receipts(ReceiptFilter::new().action(action_type))
    }
}

/// Helper for creating action records easily.
pub struct ActionBuilder {
    sister_type: SisterType,
    action_type: String,
}

impl ActionBuilder {
    pub fn new(sister_type: SisterType, action_type: impl Into<String>) -> Self {
        Self {
            sister_type,
            action_type: action_type.into(),
        }
    }

    pub fn success(self) -> ActionRecord {
        ActionRecord::new(self.sister_type, self.action_type, ActionOutcome::success())
    }

    pub fn success_with(self, result: impl Serialize) -> ActionRecord {
        ActionRecord::new(
            self.sister_type,
            self.action_type,
            ActionOutcome::success_with(result),
        )
    }

    pub fn failure(self, code: impl Into<String>, message: impl Into<String>) -> ActionRecord {
        ActionRecord::new(
            self.sister_type,
            self.action_type,
            ActionOutcome::failure(code, message),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_record() {
        let record = ActionRecord::new(SisterType::Memory, "memory_add", ActionOutcome::success())
            .param("content", "test memory")
            .evidence("ev_123");

        assert_eq!(record.sister_type, SisterType::Memory);
        assert_eq!(record.action_type, "memory_add");
        assert!(record.outcome.is_success());
        assert_eq!(record.evidence_ids, vec!["ev_123"]);
    }

    #[test]
    fn test_action_builder() {
        let record = ActionBuilder::new(SisterType::Vision, "vision_capture")
            .success_with(serde_json::json!({"capture_id": "cap_123"}));

        assert_eq!(record.action_type, "vision_capture");
        assert!(record.outcome.is_success());
    }

    #[test]
    fn test_receipt_filter() {
        let filter = ReceiptFilter::new()
            .for_sister(SisterType::Memory)
            .action("memory_add")
            .successful_only()
            .limit(10);

        assert_eq!(filter.sister_type, Some(SisterType::Memory));
        assert_eq!(filter.action_type, Some("memory_add".to_string()));
        assert_eq!(filter.outcome, Some("success".to_string()));
        assert_eq!(filter.limit, Some(10));
    }
}
