//! Shared types used across all sisters.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// All sister types in the ecosystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SisterType {
    // Foundation sisters
    Memory,
    Vision,
    Codebase,
    Identity,
    Time,
    Contract,

    // Cognitive sisters
    Comm,
    Planning,
    Cognition,
    Reality,

    // Future sisters (reserved)
    Attention,
    Affect,
    Motivation,
    Learning,
    Bond,
    Meaning,
    Wonder,
    Imagination,
    Conscience,
    Meta,
    Duration,
}

impl SisterType {
    /// Get the file extension for this sister type.
    pub fn file_extension(&self) -> &'static str {
        match self {
            Self::Memory => "amem",
            Self::Vision => "avis",
            Self::Codebase => "acb",
            Self::Identity => "aid",
            Self::Time => "atime",
            Self::Contract => "acon",
            Self::Comm => "acomm",
            Self::Planning => "aplan",
            Self::Cognition => "acog",
            Self::Reality => "areal",
            Self::Attention => "aatt",
            Self::Affect => "aaff",
            Self::Motivation => "amot",
            Self::Learning => "alrn",
            Self::Bond => "abond",
            Self::Meaning => "amean",
            Self::Wonder => "awond",
            Self::Imagination => "aimag",
            Self::Conscience => "acons",
            Self::Meta => "ameta",
            Self::Duration => "adur",
        }
    }

    /// Get the MCP tool prefix for this sister type.
    pub fn mcp_prefix(&self) -> &'static str {
        match self {
            Self::Memory => "memory",
            Self::Vision => "vision",
            Self::Codebase => "codebase",
            Self::Identity => "identity",
            Self::Time => "time",
            Self::Contract => "contract",
            Self::Comm => "comm",
            Self::Planning => "planning",
            Self::Cognition => "cognition",
            Self::Reality => "reality",
            Self::Attention => "attention",
            Self::Affect => "affect",
            Self::Motivation => "motivation",
            Self::Learning => "learning",
            Self::Bond => "bond",
            Self::Meaning => "meaning",
            Self::Wonder => "wonder",
            Self::Imagination => "imagination",
            Self::Conscience => "conscience",
            Self::Meta => "meta",
            Self::Duration => "duration",
        }
    }

    /// Get the byte identifier for file format headers.
    pub fn to_byte(&self) -> u8 {
        match self {
            Self::Memory => 0x01,
            Self::Vision => 0x02,
            Self::Codebase => 0x03,
            Self::Identity => 0x04,
            Self::Time => 0x05,
            Self::Contract => 0x06,
            Self::Comm => 0x07,
            Self::Planning => 0x08,
            Self::Cognition => 0x09,
            Self::Reality => 0x0A,
            Self::Attention => 0x0B,
            Self::Affect => 0x0C,
            Self::Motivation => 0x0D,
            Self::Learning => 0x0E,
            Self::Bond => 0x0F,
            Self::Meaning => 0x10,
            Self::Wonder => 0x11,
            Self::Imagination => 0x12,
            Self::Conscience => 0x13,
            Self::Meta => 0x14,
            Self::Duration => 0x15,
        }
    }

    /// Get sister type from byte identifier.
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(Self::Memory),
            0x02 => Some(Self::Vision),
            0x03 => Some(Self::Codebase),
            0x04 => Some(Self::Identity),
            0x05 => Some(Self::Time),
            0x06 => Some(Self::Contract),
            0x07 => Some(Self::Comm),
            0x08 => Some(Self::Planning),
            0x09 => Some(Self::Cognition),
            0x0A => Some(Self::Reality),
            0x0B => Some(Self::Attention),
            0x0C => Some(Self::Affect),
            0x0D => Some(Self::Motivation),
            0x0E => Some(Self::Learning),
            0x0F => Some(Self::Bond),
            0x10 => Some(Self::Meaning),
            0x11 => Some(Self::Wonder),
            0x12 => Some(Self::Imagination),
            0x13 => Some(Self::Conscience),
            0x14 => Some(Self::Meta),
            0x15 => Some(Self::Duration),
            _ => None,
        }
    }
}

impl std::fmt::Display for SisterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.mcp_prefix())
    }
}

/// Semantic version.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl Version {
    pub fn new(major: u8, minor: u8, patch: u8) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Check if this version is compatible with another.
    /// Compatible means same major version.
    pub fn is_compatible_with(&self, other: &Version) -> bool {
        self.major == other.major
    }

    /// Check if this version can read files from another version.
    /// We can always read older versions (backward compatible).
    pub fn can_read(&self, file_version: &Version) -> bool {
        self.major >= file_version.major
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl From<(u8, u8, u8)> for Version {
    fn from((major, minor, patch): (u8, u8, u8)) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

/// Sister status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Starting,
    Ready,
    Busy,
    Degraded,
    ShuttingDown,
    Error,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Starting => write!(f, "starting"),
            Self::Ready => write!(f, "ready"),
            Self::Busy => write!(f, "busy"),
            Self::Degraded => write!(f, "degraded"),
            Self::ShuttingDown => write!(f, "shutting_down"),
            Self::Error => write!(f, "error"),
        }
    }
}

/// Capability that a sister provides.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub description: String,
}

impl Capability {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
        }
    }
}

/// Resource usage metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub memory_bytes: usize,
    pub disk_bytes: usize,
    pub open_handles: usize,
}

/// Health status returned by all sisters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Is the sister operational?
    pub healthy: bool,

    /// Current status.
    pub status: Status,

    /// Time since initialization.
    #[serde(with = "duration_serde")]
    pub uptime: std::time::Duration,

    /// Resource usage.
    pub resources: ResourceUsage,

    /// Any warnings (non-fatal issues).
    pub warnings: Vec<String>,

    /// Last error if any.
    pub last_error: Option<String>,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            healthy: true,
            status: Status::Ready,
            uptime: std::time::Duration::ZERO,
            resources: ResourceUsage::default(),
            warnings: vec![],
            last_error: None,
        }
    }
}

/// Generic metadata map.
pub type Metadata = HashMap<String, serde_json::Value>;

/// Unique identifier (UUID-based).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UniqueId(pub Uuid);

impl UniqueId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn nil() -> Self {
        Self(Uuid::nil())
    }
}

impl Default for UniqueId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for UniqueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for UniqueId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// Timestamp wrapper for consistency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Timestamp(pub DateTime<Utc>);

impl Timestamp {
    pub fn now() -> Self {
        Self(Utc::now())
    }

    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_rfc3339())
    }
}

// Duration serialization helper
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_secs_f64().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = f64::deserialize(deserializer)?;
        Ok(Duration::from_secs_f64(secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sister_type_byte_roundtrip() {
        for sister in [
            SisterType::Memory,
            SisterType::Vision,
            SisterType::Codebase,
            SisterType::Identity,
        ] {
            let byte = sister.to_byte();
            let recovered = SisterType::from_byte(byte).unwrap();
            assert_eq!(sister, recovered);
        }
    }

    #[test]
    fn test_version_compatibility() {
        let v1 = Version::new(1, 0, 0);
        let v1_1 = Version::new(1, 1, 0);
        let v2 = Version::new(2, 0, 0);

        assert!(v1.is_compatible_with(&v1_1));
        assert!(!v1.is_compatible_with(&v2));
        assert!(v2.can_read(&v1));
        assert!(!v1.can_read(&v2));
    }
}
