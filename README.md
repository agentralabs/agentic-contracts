# agentic-contracts

**Shared contracts for the Agentra ecosystem.**

This crate defines the traits, types, and standards that ALL sisters must implement. It serves as the single source of truth for the ecosystem.

## Installation

```toml
[dependencies]
agentic-contracts = "0.2"
```

## Core Traits

### Sister Trait
The foundation that all sisters implement:

```rust
use agentic_contracts::prelude::*;

pub struct MyNewSister { /* ... */ }

impl Sister for MyNewSister {
    const SISTER_TYPE: SisterType = SisterType::Memory;
    const FILE_EXTENSION: &'static str = "amem";

    fn init(config: SisterConfig) -> SisterResult<Self> { /* ... */ }
    fn health(&self) -> HealthStatus { /* ... */ }
    fn version(&self) -> Version { /* ... */ }
    fn shutdown(&mut self) -> SisterResult<()> { /* ... */ }
    fn capabilities(&self) -> Vec<Capability> { /* ... */ }
}
```

### SessionManagement Trait
Append-only sequential sessions (Memory, Vision, Identity):

```rust
impl SessionManagement for MyNewSister {
    fn start_session(&mut self, name: &str) -> SisterResult<ContextId> { /* ... */ }
    fn end_session(&mut self) -> SisterResult<()> { /* ... */ }
    fn current_session(&self) -> Option<ContextId> { /* ... */ }
    fn export_session(&self, id: ContextId) -> SisterResult<ContextSnapshot> { /* ... */ }
    fn import_session(&mut self, snapshot: ContextSnapshot) -> SisterResult<ContextId> { /* ... */ }
}
```

### WorkspaceManagement Trait
Switchable named workspaces (Codebase):

```rust
impl WorkspaceManagement for MyNewSister {
    fn create_workspace(&mut self, name: &str) -> SisterResult<ContextId> { /* ... */ }
    fn switch_workspace(&mut self, id: ContextId) -> SisterResult<()> { /* ... */ }
    fn current_workspace(&self) -> Option<ContextId> { /* ... */ }
    fn delete_workspace(&mut self, id: ContextId) -> SisterResult<()> { /* ... */ }
}
```

### Grounding Trait
Search-based claim verification:

```rust
impl Grounding for MyNewSister {
    fn ground(&self, claim: &str) -> SisterResult<GroundingResult> { /* ... */ }
    fn evidence(&self, query: &str, max_results: usize) -> SisterResult<Vec<EvidenceDetail>> { /* ... */ }
    fn suggest(&self, query: &str, limit: usize) -> SisterResult<Vec<GroundingSuggestion>> { /* ... */ }
}
```

### Queryable Trait
Standard query interface:

```rust
impl Queryable for MyNewSister {
    fn query(&self, query: Query) -> SisterResult<QueryResult> { /* ... */ }
    fn supports_query(&self, query_type: &str) -> bool { /* ... */ }
    fn query_types(&self) -> Vec<QueryTypeInfo> { /* ... */ }
}
```

### EventEmitter Trait
Observability events:

```rust
impl EventEmitter for MyNewSister {
    fn subscribe(&self, filter: EventFilter) -> EventReceiver { /* ... */ }
    fn recent_events(&self, limit: usize) -> Vec<SisterEvent> { /* ... */ }
    fn emit(&self, event: SisterEvent) { /* ... */ }
}
```

## File Format

Each sister uses its own binary header format. The contracts provide
`FileFormatReader` and `FileFormatWriter` traits for unified access:

```rust
use agentic_contracts::file_format::*;

impl FileFormatReader for MyNewSister {
    fn can_read(&self, path: &Path) -> bool { /* check magic bytes */ }
    fn file_info(&self, path: &Path) -> SisterResult<FileInfo> { /* ... */ }
    fn file_version(&self, path: &Path) -> SisterResult<Version> { /* ... */ }
}

impl FileFormatWriter for MyNewSister {
    fn write_file(&self, path: &Path) -> SisterResult<()> { /* ... */ }
    fn export_bytes(&self) -> SisterResult<Vec<u8>> { /* ... */ }
}
```

## Error Handling

Two-layer error model across all sisters:

```rust
use agentic_contracts::errors::*;

// Domain errors (tool execution failures -> isError: true in MCP)
let err = SisterError::not_found("node_123");
let err = SisterError::invalid_input("name cannot be empty");
let err = SisterError::storage("Failed to write file");

// Protocol errors (routing failures -> JSON-RPC error response)
let err = ProtocolError::tool_not_found("unknown_tool");
let err = ProtocolError::invalid_params("missing required field 'name'");
```

## Sisters Covered

| Sister | Type | Extension | Status |
|--------|------|-----------|--------|
| Memory | `SisterType::Memory` | `.amem` | ✅ Shipped |
| Vision | `SisterType::Vision` | `.avis` | ✅ Shipped |
| Codebase | `SisterType::Codebase` | `.acb` | ✅ Shipped |
| Identity | `SisterType::Identity` | `.aid` | ✅ Shipped |
| Time | `SisterType::Time` | `.atime` | ✅ Shipped |

## License

MIT
