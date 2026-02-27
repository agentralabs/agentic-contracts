//! # Agentic Contracts v0.2.0
//!
//! Shared contracts for the AgenticOS ecosystem.
//!
//! This crate defines the traits, types, and standards that ALL sisters must implement.
//! It serves as the single source of truth for:
//!
//! - **Sister trait**: Core lifecycle management
//! - **SessionManagement / WorkspaceManagement**: Context handling (split in v0.2.0)
//! - **Grounding trait**: Query-based evidence verification (rewritten in v0.2.0)
//! - **EventEmitter trait**: Observability events
//! - **Queryable trait**: Standard query interface
//! - **FileFormat traits**: 20-year compatible file I/O (trait-based in v0.2.0)
//! - **Errors**: Two-layer error model — ProtocolError + SisterError (new in v0.2.0)
//! - **Hydra**: Placeholder traits for orchestrator integration (new in v0.2.0)
//!
//! ## What changed in v0.2.0
//!
//! v0.1.0 was written based on THEORY. v0.2.0 was validated against REALITY
//! (the actual implementations in 5 shipped sisters).
//!
//! Key changes:
//! - `SisterFileHeader` (96-byte "AGNT") → `FileFormatReader`/`FileFormatWriter` traits
//! - `ContextManagement` → split into `SessionManagement` + `WorkspaceManagement`
//! - `Grounding` → query-based (no evidence_id), three methods: ground/evidence/suggest
//! - `SisterConfig` → flexible data paths (single, multiple, or none)
//! - Added `ProtocolError` for MCP/JSON-RPC errors (separate from `SisterError`)
//! - Added `hydra` module with `HydraBridge` and `ExecutionGate` placeholders
//!
//! ## Usage
//!
//! ```toml
//! [dependencies]
//! agentic-contracts = "0.2"
//! ```
//!
//! ```rust,ignore
//! use agentic_contracts::prelude::*;
//!
//! pub struct MyNewSister {
//!     // ...
//! }
//!
//! impl Sister for MyNewSister {
//!     const SISTER_TYPE: SisterType = SisterType::Memory;
//!     const FILE_EXTENSION: &'static str = "amem";
//!     // ...
//! }
//! ```
//!
//! ## The Promise
//!
//! - ANY sister can be consumed by Hydra uniformly
//! - ANY sister can work with ANY other sister
//! - ANY file format will be readable in 20 years

pub mod context;
pub mod errors;
pub mod events;
pub mod file_format;
pub mod grounding;
pub mod hydra;
pub mod query;
pub mod receipts;
pub mod sister;
pub mod types;

// Re-export everything in prelude for convenience
pub mod prelude {
    pub use crate::context::*;
    pub use crate::errors::*;
    pub use crate::events::*;
    pub use crate::file_format::*;
    pub use crate::grounding::*;
    pub use crate::hydra::*;
    pub use crate::query::*;
    pub use crate::receipts::*;
    pub use crate::sister::*;
    pub use crate::types::*;
}

// Also re-export at crate root
pub use prelude::*;
