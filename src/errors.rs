//! Standard error types for all sisters.
//!
//! Two error layers:
//!
//! 1. **ProtocolError** — MCP/JSON-RPC protocol errors (wrong method, bad params, unknown tool).
//!    These become JSON-RPC error responses.
//!
//! 2. **SisterError** — Domain/business logic errors (node not found, invalid state).
//!    These become `{isError: true}` tool results per MCP spec.
//!
//! # MCP Error Handling Rule
//!
//! If the tool was found and invoked, errors go through `isError: true`.
//! JSON-RPC errors are only for protocol/routing failures.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

// ═══════════════════════════════════════════════════════════════════
// LAYER 1: MCP Protocol Errors (JSON-RPC error responses)
// ═══════════════════════════════════════════════════════════════════

/// Standard JSON-RPC / MCP protocol error codes.
///
/// These are used ONLY for protocol-level failures.
/// Tool execution errors should use `SisterError` + `isError: true`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProtocolErrorCode {
    /// JSON parse error (-32700)
    ParseError = -32700,

    /// Invalid JSON-RPC request (-32600)
    InvalidRequest = -32600,

    /// Method not found (-32601)
    MethodNotFound = -32601,

    /// Invalid method parameters (-32602)
    InvalidParams = -32602,

    /// Internal JSON-RPC error (-32603)
    InternalError = -32603,

    /// Tool not found (-32803) — MCP extension
    ToolNotFound = -32803,
}

impl ProtocolErrorCode {
    /// Get the numeric JSON-RPC error code
    pub fn code(&self) -> i32 {
        *self as i32
    }
}

impl std::fmt::Display for ProtocolErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError => write!(f, "Parse error"),
            Self::InvalidRequest => write!(f, "Invalid request"),
            Self::MethodNotFound => write!(f, "Method not found"),
            Self::InvalidParams => write!(f, "Invalid params"),
            Self::InternalError => write!(f, "Internal error"),
            Self::ToolNotFound => write!(f, "Tool not found"),
        }
    }
}

/// MCP protocol error — becomes a JSON-RPC error response.
///
/// Use this for:
/// - Parse errors
/// - Invalid requests
/// - Unknown methods
/// - Unknown tools (code -32803, NOT -32602)
/// - Invalid parameters (before the tool is invoked)
#[derive(Debug, Clone, Error)]
#[error("[{code}] {message}")]
pub struct ProtocolError {
    /// JSON-RPC error code
    pub code: ProtocolErrorCode,

    /// Human-readable error message
    pub message: String,

    /// Optional structured data
    pub data: Option<serde_json::Value>,
}

impl ProtocolError {
    /// Create a new protocol error
    pub fn new(code: ProtocolErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Add structured data to the error
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    /// Create a "tool not found" error (code -32803)
    pub fn tool_not_found(tool_name: &str) -> Self {
        Self::new(
            ProtocolErrorCode::ToolNotFound,
            format!("Tool not found: {}", tool_name),
        )
    }

    /// Create an "invalid params" error (code -32602)
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::new(ProtocolErrorCode::InvalidParams, message)
    }

    /// Create a "parse error" (code -32700)
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::new(ProtocolErrorCode::ParseError, message)
    }

    /// Create a "method not found" error (code -32601)
    pub fn method_not_found(method: &str) -> Self {
        Self::new(
            ProtocolErrorCode::MethodNotFound,
            format!("Method not found: {}", method),
        )
    }

    /// Check if this is a protocol-level error (should be JSON-RPC error)
    pub fn is_protocol_error(&self) -> bool {
        true // All ProtocolErrors are protocol-level by definition
    }

    /// Get the numeric error code for JSON-RPC response
    pub fn json_rpc_code(&self) -> i32 {
        self.code.code()
    }
}

// ═══════════════════════════════════════════════════════════════════
// LAYER 2: Domain/Business Logic Errors (isError: true in MCP)
// ═══════════════════════════════════════════════════════════════════

/// Standard error type for ALL sisters — domain/business logic errors.
///
/// These errors occur AFTER a tool is found and invoked.
/// In MCP, they become `{isError: true}` in the tool result,
/// NOT JSON-RPC error responses.
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[error("[{code}] {message}")]
pub struct SisterError {
    /// Error code (machine-readable)
    pub code: ErrorCode,

    /// Severity level
    pub severity: Severity,

    /// Human-readable message (should be actionable for LLMs)
    pub message: String,

    /// Additional context (for debugging)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<HashMap<String, serde_json::Value>>,

    /// Is this recoverable?
    pub recoverable: bool,

    /// Suggested action for recovery
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_action: Option<SuggestedAction>,
}

impl SisterError {
    /// Create a new error
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        let severity = code.default_severity();
        let recoverable = code.is_typically_recoverable();

        Self {
            code,
            severity,
            message: message.into(),
            context: None,
            recoverable,
            suggested_action: None,
        }
    }

    /// Add context to the error
    pub fn with_context(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        let context = self.context.get_or_insert_with(HashMap::new);
        if let Ok(v) = serde_json::to_value(value) {
            context.insert(key.into(), v);
        }
        self
    }

    /// Set recoverable flag
    pub fn recoverable(mut self, recoverable: bool) -> Self {
        self.recoverable = recoverable;
        self
    }

    /// Set suggested action
    pub fn with_suggestion(mut self, action: SuggestedAction) -> Self {
        self.suggested_action = Some(action);
        self
    }

    /// Set severity
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Format as an MCP-friendly error message.
    ///
    /// Includes what went wrong AND what to try instead,
    /// so the LLM can reason about recovery.
    pub fn to_mcp_message(&self) -> String {
        let mut msg = format!("Error: {}", self.message);
        if let Some(ref action) = self.suggested_action {
            match action {
                SuggestedAction::Retry { after_ms } => {
                    msg.push_str(&format!(". Retry after {}ms", after_ms));
                }
                SuggestedAction::Alternative { description } => {
                    msg.push_str(&format!(". Try: {}", description));
                }
                SuggestedAction::UserAction { description } => {
                    msg.push_str(&format!(". User action needed: {}", description));
                }
                SuggestedAction::Restart => {
                    msg.push_str(". Try restarting the sister");
                }
                SuggestedAction::CheckConfig { key } => {
                    msg.push_str(&format!(". Check config key: {}", key));
                }
                SuggestedAction::ReportBug => {
                    msg.push_str(". This may be a bug — please report it");
                }
            }
        }
        msg
    }

    // ═══════════════════════════════════════════════════════════
    // Common error constructors
    // ═══════════════════════════════════════════════════════════

    /// Not found error
    pub fn not_found(resource: impl Into<String>) -> Self {
        let resource = resource.into();
        Self::new(ErrorCode::NotFound, format!("{} not found", resource)).with_suggestion(
            SuggestedAction::Alternative {
                description: "Check the ID or use a query/list tool to find available items".into(),
            },
        )
    }

    /// Invalid input error
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidInput, message)
    }

    /// Permission denied error
    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::PermissionDenied, message).recoverable(false)
    }

    /// Internal error (bug)
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Internal, message)
            .with_severity(Severity::Fatal)
            .recoverable(false)
            .with_suggestion(SuggestedAction::ReportBug)
    }

    /// Storage error
    pub fn storage(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::StorageError, message)
            .with_suggestion(SuggestedAction::Retry { after_ms: 1000 })
    }

    /// Context/session not found error
    pub fn context_not_found(context_id: impl Into<String>) -> Self {
        Self::new(
            ErrorCode::ContextNotFound,
            format!("Context {} not found", context_id.into()),
        )
        .with_suggestion(SuggestedAction::Alternative {
            description: "List available contexts/sessions or create a new one".into(),
        })
    }

    /// Evidence not found error
    pub fn evidence_not_found(evidence_id: impl Into<String>) -> Self {
        Self::new(
            ErrorCode::EvidenceNotFound,
            format!("Evidence {} not found", evidence_id.into()),
        )
        .recoverable(false)
    }
}

impl Default for SisterError {
    fn default() -> Self {
        Self::new(ErrorCode::Internal, "Unknown error")
    }
}

/// Standard error codes across ALL sisters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // ═══════════════════════════════════════════════════════
    // COMMON ERRORS (All sisters use these)
    // ═══════════════════════════════════════════════════════
    /// Resource not found
    NotFound,

    /// Invalid input provided
    InvalidInput,

    /// Operation not permitted
    PermissionDenied,

    /// Storage error (read/write failed)
    StorageError,

    /// Network error
    NetworkError,

    /// Operation timed out
    Timeout,

    /// Resource limits exceeded
    ResourceExhausted,

    /// Internal error (bug)
    Internal,

    /// Not implemented yet
    NotImplemented,

    /// Context/session not found
    ContextNotFound,

    /// Evidence not found
    EvidenceNotFound,

    /// Grounding failed
    GroundingFailed,

    /// Version mismatch
    VersionMismatch,

    /// Checksum mismatch (corruption)
    ChecksumMismatch,

    /// Already exists
    AlreadyExists,

    /// Invalid state for operation
    InvalidState,

    // ═══════════════════════════════════════════════════════
    // SISTER-SPECIFIC ERROR PREFIXES
    // ═══════════════════════════════════════════════════════
    /// Memory-specific error
    MemoryError,

    /// Vision-specific error
    VisionError,

    /// Codebase-specific error
    CodebaseError,

    /// Identity-specific error
    IdentityError,

    /// Time-specific error
    TimeError,

    /// Contract-specific error
    ContractError,
}

impl ErrorCode {
    /// Get default severity for this error code
    pub fn default_severity(&self) -> Severity {
        match self {
            Self::Internal | Self::ChecksumMismatch => Severity::Fatal,
            Self::PermissionDenied | Self::VersionMismatch => Severity::Error,
            Self::NotFound | Self::InvalidInput | Self::AlreadyExists => Severity::Error,
            Self::Timeout | Self::NetworkError | Self::StorageError => Severity::Error,
            Self::ResourceExhausted => Severity::Warning,
            _ => Severity::Error,
        }
    }

    /// Check if this error is typically recoverable
    pub fn is_typically_recoverable(&self) -> bool {
        match self {
            Self::Internal | Self::ChecksumMismatch | Self::VersionMismatch => false,
            Self::NotFound | Self::EvidenceNotFound => true, // Can try different ID
            Self::Timeout | Self::NetworkError | Self::StorageError => true, // Can retry
            Self::ResourceExhausted => true,                 // Can wait
            Self::InvalidInput | Self::InvalidState => true, // Can fix input
            Self::AlreadyExists => true,                     // Can use existing
            _ => true,
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::NotFound => "NOT_FOUND",
            Self::InvalidInput => "INVALID_INPUT",
            Self::PermissionDenied => "PERMISSION_DENIED",
            Self::StorageError => "STORAGE_ERROR",
            Self::NetworkError => "NETWORK_ERROR",
            Self::Timeout => "TIMEOUT",
            Self::ResourceExhausted => "RESOURCE_EXHAUSTED",
            Self::Internal => "INTERNAL",
            Self::NotImplemented => "NOT_IMPLEMENTED",
            Self::ContextNotFound => "CONTEXT_NOT_FOUND",
            Self::EvidenceNotFound => "EVIDENCE_NOT_FOUND",
            Self::GroundingFailed => "GROUNDING_FAILED",
            Self::VersionMismatch => "VERSION_MISMATCH",
            Self::ChecksumMismatch => "CHECKSUM_MISMATCH",
            Self::AlreadyExists => "ALREADY_EXISTS",
            Self::InvalidState => "INVALID_STATE",
            Self::MemoryError => "MEMORY_ERROR",
            Self::VisionError => "VISION_ERROR",
            Self::CodebaseError => "CODEBASE_ERROR",
            Self::IdentityError => "IDENTITY_ERROR",
            Self::TimeError => "TIME_ERROR",
            Self::ContractError => "CONTRACT_ERROR",
        };
        write!(f, "{}", s)
    }
}

/// Severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Informational, not really an error
    Info,

    /// Warning, operation succeeded but with issues
    Warning,

    /// Error, operation failed but recoverable
    Error,

    /// Fatal, sister is in bad state
    Fatal,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
            Self::Fatal => write!(f, "fatal"),
        }
    }
}

/// Suggested actions for error recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SuggestedAction {
    /// Retry the operation
    Retry {
        /// Milliseconds to wait before retry
        after_ms: u64,
    },

    /// Use a different approach
    Alternative {
        /// Description of the alternative
        description: String,
    },

    /// User intervention needed
    UserAction {
        /// Description of what the user should do
        description: String,
    },

    /// Restart the sister
    Restart,

    /// Check configuration
    CheckConfig {
        /// Configuration key to check
        key: String,
    },

    /// Contact support / report bug
    ReportBug,
}

// Implement From for common error types

impl From<std::io::Error> for SisterError {
    fn from(e: std::io::Error) -> Self {
        SisterError::new(ErrorCode::StorageError, format!("I/O error: {}", e))
            .with_context("io_error_kind", format!("{:?}", e.kind()))
            .with_suggestion(SuggestedAction::Retry { after_ms: 1000 })
    }
}

impl From<serde_json::Error> for SisterError {
    fn from(e: serde_json::Error) -> Self {
        SisterError::new(ErrorCode::InvalidInput, format!("JSON error: {}", e))
    }
}

/// Result type alias for sister operations (domain errors)
pub type SisterResult<T> = Result<T, SisterError>;

/// Result type alias for protocol operations
pub type ProtocolResult<T> = Result<T, ProtocolError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = SisterError::not_found("node_123");
        assert_eq!(err.code, ErrorCode::NotFound);
        assert!(err.recoverable);
        assert!(err.message.contains("node_123"));
    }

    #[test]
    fn test_error_with_context() {
        let err = SisterError::invalid_input("bad param")
            .with_context("field", "name")
            .with_context("provided", "");

        assert!(err.context.is_some());
        let ctx = err.context.unwrap();
        assert_eq!(ctx.get("field").unwrap(), "name");
    }

    #[test]
    fn test_error_serialization() {
        let err = SisterError::not_found("test");
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("NOT_FOUND"));

        let recovered: SisterError = serde_json::from_str(&json).unwrap();
        assert_eq!(recovered.code, ErrorCode::NotFound);
    }

    #[test]
    fn test_protocol_error_codes() {
        let err = ProtocolError::tool_not_found("memory_foo");
        assert_eq!(err.json_rpc_code(), -32803);
        assert!(err.is_protocol_error());
        assert!(err.message.contains("memory_foo"));

        let err2 = ProtocolError::invalid_params("missing field: claim");
        assert_eq!(err2.json_rpc_code(), -32602);

        let err3 = ProtocolError::method_not_found("tools/unknown");
        assert_eq!(err3.json_rpc_code(), -32601);
    }

    #[test]
    fn test_mcp_message_formatting() {
        let err = SisterError::not_found("node 42");
        let msg = err.to_mcp_message();
        assert!(msg.contains("node 42 not found"));
        assert!(msg.contains("Try:"));

        let err2 = SisterError::storage("disk full");
        let msg2 = err2.to_mcp_message();
        assert!(msg2.contains("Retry after"));
    }

    #[test]
    fn test_protocol_error_code_values() {
        // Verify exact JSON-RPC error codes per spec
        assert_eq!(ProtocolErrorCode::ParseError.code(), -32700);
        assert_eq!(ProtocolErrorCode::InvalidRequest.code(), -32600);
        assert_eq!(ProtocolErrorCode::MethodNotFound.code(), -32601);
        assert_eq!(ProtocolErrorCode::InvalidParams.code(), -32602);
        assert_eq!(ProtocolErrorCode::InternalError.code(), -32603);
        assert_eq!(ProtocolErrorCode::ToolNotFound.code(), -32803);
    }
}
