//! Event emission trait for observability.
//!
//! All sisters emit standardized events that Hydra can subscribe to
//! for monitoring, logging, and orchestration.

use crate::context::ContextId;
use crate::errors::SisterError;
use crate::grounding::EvidenceType;
use crate::types::{SisterType, Status, UniqueId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::broadcast;

/// Unique event identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(pub UniqueId);

impl EventId {
    pub fn new() -> Self {
        Self(UniqueId::new())
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "evt_{}", self.0)
    }
}

/// Event types that ALL sisters emit.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum EventType {
    // ═══════════════════════════════════════════════════════
    // LIFECYCLE EVENTS
    // ═══════════════════════════════════════════════════════
    /// Sister initialized and ready.
    Ready,

    /// Sister shutting down.
    ShuttingDown,

    /// Sister status changed.
    StatusChanged { from: Status, to: Status },

    // ═══════════════════════════════════════════════════════
    // CONTEXT EVENTS
    // ═══════════════════════════════════════════════════════
    /// Context created.
    ContextCreated { context_id: ContextId, name: String },

    /// Context switched.
    ContextSwitched { from: ContextId, to: ContextId },

    /// Context deleted.
    ContextDeleted { context_id: ContextId },

    // ═══════════════════════════════════════════════════════
    // OPERATION EVENTS
    // ═══════════════════════════════════════════════════════
    /// Operation started.
    OperationStarted {
        operation_id: String,
        operation_type: String,
    },

    /// Operation completed successfully.
    OperationCompleted {
        operation_id: String,
        #[serde(with = "duration_millis")]
        duration: Duration,
    },

    /// Operation failed.
    OperationFailed {
        operation_id: String,
        error_code: String,
        error_message: String,
    },

    // ═══════════════════════════════════════════════════════
    // EVIDENCE EVENTS
    // ═══════════════════════════════════════════════════════
    /// Evidence created.
    EvidenceCreated {
        evidence_id: String,
        evidence_type: EvidenceType,
    },

    /// Grounding performed.
    GroundingPerformed {
        grounding_id: String,
        grounded: bool,
        confidence: f64,
    },

    // ═══════════════════════════════════════════════════════
    // RESOURCE EVENTS
    // ═══════════════════════════════════════════════════════
    /// Memory pressure warning.
    MemoryPressure { usage_percent: f64 },

    /// Storage pressure warning.
    StoragePressure { usage_percent: f64 },

    // ═══════════════════════════════════════════════════════
    // CUSTOM EVENTS
    // ═══════════════════════════════════════════════════════
    /// Sister-specific custom event.
    Custom {
        name: String,
        data: serde_json::Value,
    },
}

/// Event emitted by a sister.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SisterEvent {
    /// Unique event ID.
    pub id: EventId,

    /// Which sister emitted this.
    pub sister_type: SisterType,

    /// Event type and data.
    #[serde(flatten)]
    pub event_type: EventType,

    /// Timestamp.
    pub timestamp: DateTime<Utc>,

    /// Context this event occurred in (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_id: Option<ContextId>,
}

impl SisterEvent {
    /// Create a new event.
    pub fn new(sister_type: SisterType, event_type: EventType) -> Self {
        Self {
            id: EventId::new(),
            sister_type,
            event_type,
            timestamp: Utc::now(),
            context_id: None,
        }
    }

    /// Add context ID.
    pub fn in_context(mut self, context_id: ContextId) -> Self {
        self.context_id = Some(context_id);
        self
    }

    // Event constructors

    pub fn ready(sister_type: SisterType) -> Self {
        Self::new(sister_type, EventType::Ready)
    }

    pub fn shutting_down(sister_type: SisterType) -> Self {
        Self::new(sister_type, EventType::ShuttingDown)
    }

    pub fn status_changed(sister_type: SisterType, from: Status, to: Status) -> Self {
        Self::new(sister_type, EventType::StatusChanged { from, to })
    }

    pub fn context_created(sister_type: SisterType, context_id: ContextId, name: String) -> Self {
        Self::new(sister_type, EventType::ContextCreated { context_id, name })
    }

    pub fn context_switched(sister_type: SisterType, from: ContextId, to: ContextId) -> Self {
        Self::new(sister_type, EventType::ContextSwitched { from, to })
    }

    pub fn operation_started(
        sister_type: SisterType,
        operation_id: impl Into<String>,
        operation_type: impl Into<String>,
    ) -> Self {
        Self::new(
            sister_type,
            EventType::OperationStarted {
                operation_id: operation_id.into(),
                operation_type: operation_type.into(),
            },
        )
    }

    pub fn operation_completed(
        sister_type: SisterType,
        operation_id: impl Into<String>,
        duration: Duration,
    ) -> Self {
        Self::new(
            sister_type,
            EventType::OperationCompleted {
                operation_id: operation_id.into(),
                duration,
            },
        )
    }

    pub fn operation_failed(
        sister_type: SisterType,
        operation_id: impl Into<String>,
        error: &SisterError,
    ) -> Self {
        Self::new(
            sister_type,
            EventType::OperationFailed {
                operation_id: operation_id.into(),
                error_code: error.code.to_string(),
                error_message: error.message.clone(),
            },
        )
    }

    pub fn evidence_created(
        sister_type: SisterType,
        evidence_id: impl Into<String>,
        evidence_type: EvidenceType,
    ) -> Self {
        Self::new(
            sister_type,
            EventType::EvidenceCreated {
                evidence_id: evidence_id.into(),
                evidence_type,
            },
        )
    }

    pub fn grounding_performed(
        sister_type: SisterType,
        grounding_id: impl Into<String>,
        grounded: bool,
        confidence: f64,
    ) -> Self {
        Self::new(
            sister_type,
            EventType::GroundingPerformed {
                grounding_id: grounding_id.into(),
                grounded,
                confidence,
            },
        )
    }
}

/// Filter for subscribing to events.
#[derive(Debug, Clone, Default)]
pub struct EventFilter {
    /// Filter by sister type.
    pub sister_type: Option<SisterType>,

    /// Filter by event type names.
    pub event_types: Option<Vec<String>>,

    /// Filter by context.
    pub context_id: Option<ContextId>,
}

impl EventFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn for_sister(mut self, sister_type: SisterType) -> Self {
        self.sister_type = Some(sister_type);
        self
    }

    pub fn in_context(mut self, context_id: ContextId) -> Self {
        self.context_id = Some(context_id);
        self
    }

    /// Check if an event matches this filter.
    pub fn matches(&self, event: &SisterEvent) -> bool {
        if let Some(st) = &self.sister_type {
            if event.sister_type != *st {
                return false;
            }
        }

        if let Some(ctx) = &self.context_id {
            if event.context_id.as_ref() != Some(ctx) {
                return false;
            }
        }

        true
    }
}

/// Event receiver (broadcast channel).
pub type EventReceiver = broadcast::Receiver<SisterEvent>;

/// Event sender (broadcast channel).
pub type EventSender = broadcast::Sender<SisterEvent>;

/// Event emitter trait for observability.
pub trait EventEmitter {
    /// Subscribe to events with optional filter.
    fn subscribe(&self, filter: EventFilter) -> EventReceiver;

    /// Get recent events.
    fn recent_events(&self, limit: usize) -> Vec<SisterEvent>;

    /// Emit an event (for internal use).
    fn emit(&self, event: SisterEvent);
}

/// Helper struct for managing event emission.
pub struct EventManager {
    sender: EventSender,
    recent: std::sync::Mutex<Vec<SisterEvent>>,
    max_recent: usize,
}

impl EventManager {
    /// Create a new event manager.
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender,
            recent: std::sync::Mutex::new(Vec::new()),
            max_recent: 100,
        }
    }

    /// Emit an event.
    pub fn emit(&self, event: SisterEvent) {
        // Store in recent
        {
            let mut recent = self.recent.lock().unwrap();
            recent.push(event.clone());
            if recent.len() > self.max_recent {
                recent.remove(0);
            }
        }

        // Broadcast (ignore errors if no subscribers)
        let _ = self.sender.send(event);
    }

    /// Subscribe to events.
    pub fn subscribe(&self) -> EventReceiver {
        self.sender.subscribe()
    }

    /// Get recent events.
    pub fn recent(&self, limit: usize) -> Vec<SisterEvent> {
        let recent = self.recent.lock().unwrap();
        recent.iter().rev().take(limit).cloned().collect()
    }
}

impl Default for EventManager {
    fn default() -> Self {
        Self::new(256)
    }
}

// Duration serialization as milliseconds
mod duration_millis {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_millis() as u64)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ms = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(ms))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = SisterEvent::ready(SisterType::Memory);
        assert!(matches!(event.event_type, EventType::Ready));
        assert_eq!(event.sister_type, SisterType::Memory);
    }

    #[test]
    fn test_event_filter() {
        let event = SisterEvent::ready(SisterType::Memory);
        let filter = EventFilter::new().for_sister(SisterType::Memory);
        assert!(filter.matches(&event));

        let filter2 = EventFilter::new().for_sister(SisterType::Vision);
        assert!(!filter2.matches(&event));
    }

    #[test]
    fn test_event_manager() {
        let manager = EventManager::new(10);

        manager.emit(SisterEvent::ready(SisterType::Memory));
        manager.emit(SisterEvent::ready(SisterType::Vision));

        let recent = manager.recent(10);
        assert_eq!(recent.len(), 2);
    }
}
