//! Grounding trait for evidence verification (V2 Pattern).
//!
//! v0.2.0 — Rewritten to match how all sisters ACTUALLY implement grounding.
//!
//! The real pattern across Memory, Vision, Identity, and Codebase is:
//!
//! 1. `ground(claim)` → Search-based claim verification (BM25, word-overlap, etc.)
//! 2. `evidence(query)` → Get detailed evidence for a query
//! 3. `suggest(query)` → Fuzzy fallback when exact match fails
//!
//! Key differences from v0.1.0:
//! - No `evidence_id` parameter — all sisters SEARCH for evidence
//! - `ground()` takes a claim string, not a `GroundingRequest` with evidence_id
//! - Three-status result: verified / partial / ungrounded
//! - Optional per sister (Time has no grounding)

use crate::errors::SisterResult;
use crate::types::{Metadata, SisterType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════
// GROUNDING RESULT TYPES
// ═══════════════════════════════════════════════════════════════════

/// Status of a grounding check
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroundingStatus {
    /// Claim is fully supported by evidence
    Verified,

    /// Claim is partially supported (some aspects verified, others not)
    Partial,

    /// No evidence found to support the claim
    Ungrounded,
}

impl std::fmt::Display for GroundingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Verified => write!(f, "verified"),
            Self::Partial => write!(f, "partial"),
            Self::Ungrounded => write!(f, "ungrounded"),
        }
    }
}

/// Result of a grounding check.
///
/// Mirrors the actual response shape all sisters return.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundingResult {
    /// Grounding status
    pub status: GroundingStatus,

    /// The claim that was checked
    pub claim: String,

    /// Confidence level (0.0 = no support, 1.0 = full support)
    pub confidence: f64,

    /// Evidence that supports (or fails to support) the claim
    pub evidence: Vec<GroundingEvidence>,

    /// Human-readable explanation
    pub reason: String,

    /// Suggestions for related content (when ungrounded)
    #[serde(default)]
    pub suggestions: Vec<String>,

    /// Timestamp of grounding check
    pub timestamp: DateTime<Utc>,
}

impl GroundingResult {
    /// Create a verified result
    pub fn verified(claim: impl Into<String>, confidence: f64) -> Self {
        Self {
            status: GroundingStatus::Verified,
            claim: claim.into(),
            confidence,
            evidence: vec![],
            reason: String::new(),
            suggestions: vec![],
            timestamp: Utc::now(),
        }
    }

    /// Create an ungrounded result
    pub fn ungrounded(claim: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            status: GroundingStatus::Ungrounded,
            claim: claim.into(),
            confidence: 0.0,
            evidence: vec![],
            reason: reason.into(),
            suggestions: vec![],
            timestamp: Utc::now(),
        }
    }

    /// Create a partial result
    pub fn partial(claim: impl Into<String>, confidence: f64) -> Self {
        Self {
            status: GroundingStatus::Partial,
            claim: claim.into(),
            confidence,
            evidence: vec![],
            reason: String::new(),
            suggestions: vec![],
            timestamp: Utc::now(),
        }
    }

    /// Add evidence items
    pub fn with_evidence(mut self, evidence: Vec<GroundingEvidence>) -> Self {
        self.evidence = evidence;
        self
    }

    /// Add suggestions
    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions = suggestions;
        self
    }

    /// Add reason
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = reason.into();
        self
    }

    /// Check if strongly grounded (confidence > 0.8)
    pub fn is_strongly_grounded(&self) -> bool {
        self.status == GroundingStatus::Verified && self.confidence > 0.8
    }

    /// Check if weakly grounded (confidence > 0.5)
    pub fn is_weakly_grounded(&self) -> bool {
        self.status != GroundingStatus::Ungrounded && self.confidence > 0.5
    }
}

/// A piece of evidence returned by grounding.
///
/// Intentionally flexible — each sister populates the fields
/// relevant to its domain. Memory returns nodes, Vision returns
/// observations, Identity returns trust grants + receipts, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundingEvidence {
    /// Evidence type (sister-specific: "memory_node", "observation",
    /// "trust_grant", "receipt", "code_symbol", etc.)
    pub evidence_type: String,

    /// Evidence identifier (node_id, observation_id, grant_id, etc.)
    pub id: String,

    /// Relevance score (higher = more relevant)
    pub score: f64,

    /// Human-readable summary of the evidence
    pub summary: String,

    /// Sister-specific structured data
    #[serde(default)]
    pub data: Metadata,
}

impl GroundingEvidence {
    /// Create a new evidence item
    pub fn new(
        evidence_type: impl Into<String>,
        id: impl Into<String>,
        score: f64,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            evidence_type: evidence_type.into(),
            id: id.into(),
            score,
            summary: summary.into(),
            data: Metadata::new(),
        }
    }

    /// Add structured data
    pub fn with_data(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(v) = serde_json::to_value(value) {
            self.data.insert(key.into(), v);
        }
        self
    }
}

// ═══════════════════════════════════════════════════════════════════
// EVIDENCE DETAIL TYPES (for the evidence() method)
// ═══════════════════════════════════════════════════════════════════

/// Detailed evidence item returned by the `evidence()` method.
///
/// More detailed than `GroundingEvidence` — includes full content,
/// timestamps, relationships, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceDetail {
    /// Evidence type
    pub evidence_type: String,

    /// Unique ID
    pub id: String,

    /// Relevance score
    pub score: f64,

    /// When this evidence was created
    pub created_at: DateTime<Utc>,

    /// Which sister produced this
    pub source_sister: SisterType,

    /// Full content/description
    pub content: String,

    /// Sister-specific structured data (edges, dimensions, capabilities, etc.)
    #[serde(default)]
    pub data: Metadata,
}

// ═══════════════════════════════════════════════════════════════════
// SUGGESTION TYPE (for the suggest() method)
// ═══════════════════════════════════════════════════════════════════

/// A suggestion returned when a claim doesn't match exactly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundingSuggestion {
    /// What type of item this is
    pub item_type: String,

    /// Item identifier
    pub id: String,

    /// Relevance score
    pub relevance_score: f64,

    /// Human-readable description
    pub description: String,

    /// Sister-specific data
    #[serde(default)]
    pub data: Metadata,
}

// ═══════════════════════════════════════════════════════════════════
// THE GROUNDING TRAIT
// ═══════════════════════════════════════════════════════════════════

/// Grounding capability — sisters that verify claims implement this.
///
/// Implemented by: Memory, Vision, Identity, Codebase
/// NOT implemented by: Time (no grounding concept)
///
/// The three methods mirror the actual `{sister}_ground`,
/// `{sister}_evidence`, and `{sister}_suggest` MCP tools.
pub trait Grounding {
    /// Verify a claim against stored evidence.
    ///
    /// Searches for evidence that supports or refutes the claim.
    /// Returns verified/partial/ungrounded status with confidence.
    ///
    /// # Rule: NEVER throw on missing evidence
    /// Return `GroundingStatus::Ungrounded` with `confidence: 0.0` instead.
    fn ground(&self, claim: &str) -> SisterResult<GroundingResult>;

    /// Get detailed evidence for a query.
    ///
    /// Returns matching evidence items with full content and metadata.
    /// `max_results` limits the number of items returned.
    fn evidence(&self, query: &str, max_results: usize) -> SisterResult<Vec<EvidenceDetail>>;

    /// Find similar items when an exact match fails.
    ///
    /// Returns suggestions that are close to the query,
    /// helping the LLM recover from ungrounded claims.
    fn suggest(&self, query: &str, limit: usize) -> SisterResult<Vec<GroundingSuggestion>>;
}

// ═══════════════════════════════════════════════════════════════════
// LEGACY COMPATIBILITY
// ═══════════════════════════════════════════════════════════════════

/// Type of evidence (kept for categorization, but no longer used
/// as the primary lookup mechanism).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    // Memory evidence
    MemoryNode,
    MemoryRelation,
    MemorySession,

    // Vision evidence
    Screenshot,
    DomFingerprint,
    VisualDiff,
    VisualComparison,

    // Codebase evidence
    CodeNode,
    ImpactAnalysis,
    Prophecy,
    DependencyGraph,

    // Identity evidence
    Receipt,
    TrustGrant,
    CompetenceProof,
    Signature,

    // Time evidence
    TimelineEvent,
    DurationProof,
    DeadlineCheck,

    // Contract evidence
    Agreement,
    PolicyCheck,
    BoundaryVerification,

    // Generic
    Custom(String),
}

impl std::fmt::Display for EvidenceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Custom(s) => write!(f, "{}", s),
            other => write!(f, "{:?}", other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grounding_result_verified() {
        let result = GroundingResult::verified("the sky is blue", 0.95)
            .with_evidence(vec![GroundingEvidence::new(
                "memory_node",
                "node_42",
                0.95,
                "Sky color observation from session 1",
            )])
            .with_reason("Found strong evidence in memory");

        assert_eq!(result.status, GroundingStatus::Verified);
        assert!(result.is_strongly_grounded());
        assert_eq!(result.evidence.len(), 1);
    }

    #[test]
    fn test_grounding_result_ungrounded() {
        let result = GroundingResult::ungrounded("cats can fly", "No evidence found")
            .with_suggestions(vec!["cats can jump".into(), "birds can fly".into()]);

        assert_eq!(result.status, GroundingStatus::Ungrounded);
        assert!(!result.is_strongly_grounded());
        assert!(!result.is_weakly_grounded());
        assert_eq!(result.suggestions.len(), 2);
    }

    #[test]
    fn test_grounding_evidence_builder() {
        let evidence =
            GroundingEvidence::new("trust_grant", "atrust_123", 0.8, "Deploy capability")
                .with_data("capabilities", vec!["deploy:prod"]);

        assert_eq!(evidence.evidence_type, "trust_grant");
        assert_eq!(evidence.score, 0.8);
        assert!(evidence.data.contains_key("capabilities"));
    }

    #[test]
    fn test_grounding_status_display() {
        assert_eq!(GroundingStatus::Verified.to_string(), "verified");
        assert_eq!(GroundingStatus::Partial.to_string(), "partial");
        assert_eq!(GroundingStatus::Ungrounded.to_string(), "ungrounded");
    }
}
