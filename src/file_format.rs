//! File format specifications for 20-year compatibility.
//!
//! Sisters use their OWN binary file headers (AMEM, AVIS, ACDB, ATIM)
//! or JSON (Identity). This module provides traits and utilities that
//! unify file format operations without forcing a single header layout.
//!
//! # The 20-Year Promise
//!
//! Any .a* file created today will be readable in 2046.
//!
//! # Reality (v0.2.0)
//!
//! Each sister has its own header format:
//! - Memory:   "AMEM" magic, 64-byte header
//! - Vision:   "AVIS" magic, 64-byte header
//! - Codebase: "ACDB" magic, 128-byte header
//! - Time:     "ATIM" magic, 92-byte header
//! - Identity: JSON files (no binary header)
//!
//! The v0.1.0 `SisterFileHeader` (96-byte "AGNT" magic) was never adopted.
//! v0.2.0 replaces it with a trait-based approach that each sister
//! implements according to its actual format.

use crate::errors::{ErrorCode, SisterError, SisterResult};
use crate::types::{SisterType, Version};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Information about a file (without loading full content).
///
/// Every sister can produce this from any of its files,
/// regardless of whether the format is binary or JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// Which sister owns this file
    pub sister_type: SisterType,

    /// Format version of the file
    pub version: Version,

    /// When the file was created
    pub created_at: DateTime<Utc>,

    /// When the file was last modified
    pub updated_at: DateTime<Utc>,

    /// Content length in bytes (payload, excluding header)
    pub content_length: u64,

    /// Whether this file needs migration to the current version
    pub needs_migration: bool,

    /// The magic bytes or format identifier (e.g., "AMEM", "AVIS", "aid-v1")
    pub format_id: String,
}

/// File format reader trait for all sisters.
///
/// Each sister implements this for its own file format.
/// Memory reads .amem files, Vision reads .avis files, etc.
pub trait FileFormatReader: Sized {
    /// Read a file with version handling
    fn read_file(path: &Path) -> SisterResult<Self>;

    /// Check if a file is readable (without full parse).
    /// Returns file info for quick inspection
    fn can_read(path: &Path) -> SisterResult<FileInfo>;

    /// Get file version without full parse
    fn file_version(path: &Path) -> SisterResult<Version>;

    /// Migrate old version data to current format (in memory).
    /// Returns the migrated bytes
    fn migrate(data: &[u8], from_version: Version) -> SisterResult<Vec<u8>>;
}

/// File format writer trait for all sisters
pub trait FileFormatWriter {
    /// Write to a file path
    fn write_file(&self, path: &Path) -> SisterResult<()>;

    /// Serialize the content to bytes
    fn to_bytes(&self) -> SisterResult<Vec<u8>>;
}

/// Version compatibility rules.
///
/// These rules ensure the 20-year compatibility promise.
#[derive(Debug, Clone)]
pub struct VersionCompatibility;

impl VersionCompatibility {
    /// Check if reader version can read file version.
    ///
    /// Rule: Newer readers can always read older files
    pub fn can_read(reader_version: &Version, file_version: &Version) -> bool {
        reader_version.major >= file_version.major
    }

    /// Check if file needs migration
    pub fn needs_migration(current_version: &Version, file_version: &Version) -> bool {
        file_version.major < current_version.major
    }

    /// Check if versions are fully compatible (same major)
    pub fn is_compatible(v1: &Version, v2: &Version) -> bool {
        v1.major == v2.major
    }
}

/// Helper: Read 4-byte magic from a file path.
///
/// Useful for sisters with binary formats to quickly identify files.
pub fn read_magic_bytes(path: &Path) -> SisterResult<[u8; 4]> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic).map_err(|e| {
        SisterError::new(
            ErrorCode::StorageError,
            format!("Failed to read magic bytes: {}", e),
        )
    })?;
    Ok(magic)
}

/// Helper: Identify which sister a file belongs to by magic bytes.
///
/// Known magic bytes:
/// - "AMEM" (0x414D454D) → Memory
/// - "AVIS" (0x41564953) → Vision
/// - "ACDB" (0x41434442) → Codebase
/// - "ATIM" (0x4154494D) → Time
///
/// Returns None for JSON-based formats (Identity) or unknown formats.
pub fn identify_sister_by_magic(magic: &[u8; 4]) -> Option<SisterType> {
    match magic {
        b"AMEM" => Some(SisterType::Memory),
        b"AVIS" => Some(SisterType::Vision),
        b"ACDB" => Some(SisterType::Codebase),
        b"ATIM" => Some(SisterType::Time),
        _ => None,
    }
}

/// Helper: Check if a file is a JSON-based sister format (e.g., Identity .aid files).
///
/// Peeks at the first non-whitespace byte to see if it's '{'.
pub fn is_json_format(path: &Path) -> SisterResult<bool> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut buf = [0u8; 64];
    let n = file.read(&mut buf).map_err(|e| {
        SisterError::new(
            ErrorCode::StorageError,
            format!("Failed to read file: {}", e),
        )
    })?;
    let slice = &buf[..n];
    Ok(slice.iter().find(|b| !b.is_ascii_whitespace()) == Some(&b'{'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identify_sister_by_magic() {
        assert_eq!(identify_sister_by_magic(b"AMEM"), Some(SisterType::Memory));
        assert_eq!(identify_sister_by_magic(b"AVIS"), Some(SisterType::Vision));
        assert_eq!(
            identify_sister_by_magic(b"ACDB"),
            Some(SisterType::Codebase)
        );
        assert_eq!(identify_sister_by_magic(b"ATIM"), Some(SisterType::Time));
        assert_eq!(identify_sister_by_magic(b"XXXX"), None);
        assert_eq!(identify_sister_by_magic(b"AGNT"), None); // v0.1.0 magic, no longer used
    }

    #[test]
    fn test_version_compatibility() {
        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(2, 0, 0);
        let v1_1 = Version::new(1, 1, 0);

        assert!(VersionCompatibility::can_read(&v2, &v1));
        assert!(!VersionCompatibility::can_read(&v1, &v2));
        assert!(VersionCompatibility::is_compatible(&v1, &v1_1));
        assert!(VersionCompatibility::needs_migration(&v2, &v1));
    }
}
