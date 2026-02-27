# SISTER COMPLIANCE VERIFICATION

> **Status:** Action Required
> **Date:** February 2026
> **Purpose:** Verify existing sisters comply with contracts before building more

---

## Step A: agentic-contracts Crate ✅ COMPLETE

```
agentic-contracts/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs           # Main exports and prelude
    ├── types.rs         # SisterType, Version, Status, etc.
    ├── errors.rs        # SisterError, ErrorCode, Severity
    ├── sister.rs        # Sister trait, SisterConfig
    ├── context.rs       # ContextManagement trait, ContextId
    ├── grounding.rs     # Grounding trait, Evidence types
    ├── events.rs        # EventEmitter trait, SisterEvent
    ├── query.rs         # Queryable trait, Query/QueryResult
    ├── receipts.rs      # ReceiptIntegration trait
    └── file_format.rs   # SisterFileHeader, 20-year compatibility

TOTAL: ~95KB of Rust code defining all contracts
```

### To Publish:

```bash
# 1. Copy to your workspace
cp -r agentic-contracts /path/to/your/repos/

# 2. Verify it compiles
cd agentic-contracts
cargo check
cargo test

# 3. Publish to crates.io
cargo publish
```

---

## Step B: Verify Existing Sisters

### Memory (agentic-memory) - v0.3.2

```
COMPLIANCE STATUS:
──────────────────

□ SISTER TRAIT
  ✅ init/shutdown exist
  ✅ health exists
  ✅ version exists
  ⚠️ Need to verify exact signature matches

□ CONTEXT MANAGEMENT
  ✅ Sessions exist (memory_session_resume)
  ⚠️ Need to add: context_create, context_switch, context_list, context_delete
  ⚠️ Need to map: Session → Context

□ GROUNDING
  ✅ memory_ground exists
  ⚠️ Verify signature matches GroundingRequest → GroundingResult

□ EVENTS
  ❌ Missing: EventEmitter implementation
  ❌ Missing: Standard event emission

□ QUERY
  ✅ memory_query exists
  ⚠️ Verify matches Query → QueryResult pattern

□ RECEIPTS
  ⚠️ Need to integrate with Identity

□ FILE FORMAT
  ✅ .amem format exists
  ⚠️ Verify header matches SisterFileHeader (96 bytes)

□ MCP TOOLS
  ✅ memory_add, memory_query, memory_similar, etc.
  ⚠️ Need to add: memory_context_* tools
  ⚠️ Need to standardize error responses

CHANGES NEEDED:
───────────────
1. Add dependency: agentic-contracts = "0.1"
2. Implement ContextManagement trait (map Session to Context)
3. Add EventEmitter with EventManager
4. Add context MCP tools
5. Verify file header compatibility
6. Standardize error responses to SisterError format
```

### Vision (agentic-vision) - v0.2.2

```
COMPLIANCE STATUS:
──────────────────

□ SISTER TRAIT
  ✅ Core functionality exists
  ⚠️ Need to verify exact signature matches

□ CONTEXT MANAGEMENT
  ✅ Archives exist conceptually
  ❌ Missing: Explicit context_* methods
  ⚠️ Need to map: Archive → Context

□ GROUNDING
  ✅ vision_ground exists
  ⚠️ Verify signature matches

□ EVENTS
  ❌ Missing: EventEmitter implementation

□ QUERY
  ⚠️ Verify query interface exists

□ FILE FORMAT
  ✅ .avis format exists
  ⚠️ Verify header compatibility

□ MCP TOOLS
  ✅ vision_capture, vision_compare, etc.
  ⚠️ Need to add: vision_context_* tools

CHANGES NEEDED:
───────────────
1. Add dependency: agentic-contracts = "0.1"
2. Implement ContextManagement (Archive → Context)
3. Add EventEmitter
4. Add context MCP tools
5. Standardize errors
```

### Codebase (agentic-codebase) - v0.2.2

```
COMPLIANCE STATUS:
──────────────────

□ SISTER TRAIT
  ✅ Core functionality exists

□ CONTEXT MANAGEMENT
  ✅ Workspaces exist conceptually
  ❌ Missing: Explicit context_* methods
  ⚠️ Need to map: Workspace → Context

□ GROUNDING
  ✅ codebase_ground exists

□ EVENTS
  ❌ Missing: EventEmitter implementation

□ QUERY
  ✅ codebase_query exists

□ FILE FORMAT
  ✅ .acb format exists

□ MCP TOOLS
  ✅ codebase_query, codebase_impact, etc.
  ⚠️ Need to add: codebase_context_* tools

CHANGES NEEDED:
───────────────
1. Add dependency: agentic-contracts = "0.1"
2. Implement ContextManagement (Workspace → Context)
3. Add EventEmitter
4. Add context MCP tools
5. Standardize errors
```

### Identity (agentic-identity) - v0.2.3

```
COMPLIANCE STATUS:
──────────────────

□ SISTER TRAIT
  ✅ Core functionality exists

□ CONTEXT MANAGEMENT
  ✅ Chains exist conceptually
  ❌ Missing: Explicit context_* methods
  ⚠️ Need to map: Chain → Context

□ GROUNDING
  ✅ identity_ground exists (receipts as evidence)

□ EVENTS
  ❌ Missing: EventEmitter implementation

□ RECEIPTS
  ✅ Identity IS the receipt system
  ⚠️ Verify Receipt schema matches contract

□ FILE FORMAT
  ✅ .aid format exists

□ MCP TOOLS
  ✅ identity_sign, identity_verify, etc.
  ⚠️ Need to add: identity_context_* tools

CHANGES NEEDED:
───────────────
1. Add dependency: agentic-contracts = "0.1"
2. Implement ContextManagement (Chain → Context)
3. Add EventEmitter
4. Add context MCP tools
5. Verify Receipt schema alignment
```

---

## Step B Summary: Required Updates

```
PRIORITY ORDER:
───────────────

1. IDENTITY (First - it's the receipt system)
   □ Add agentic-contracts dependency
   □ Verify Receipt schema matches
   □ Add ContextManagement
   □ Add EventEmitter
   □ Add context MCP tools

2. MEMORY (Second - most used sister)
   □ Add agentic-contracts dependency
   □ Map Session → Context
   □ Add EventEmitter
   □ Add context MCP tools

3. VISION (Third)
   □ Add agentic-contracts dependency
   □ Map Archive → Context
   □ Add EventEmitter
   □ Add context MCP tools

4. CODEBASE (Fourth)
   □ Add agentic-contracts dependency
   □ Map Workspace → Context
   □ Add EventEmitter
   □ Add context MCP tools

ESTIMATED TIME PER SISTER:
──────────────────────────
• Add dependency + verify traits: 1-2 hours
• Implement ContextManagement: 2-3 hours
• Add EventEmitter: 1-2 hours
• Add MCP tools: 2-3 hours
• Testing: 2-3 hours

TOTAL: ~8-12 hours per sister
TOTAL ALL 4: ~2-3 days
```

---

## Step C: New Sisters Follow Template

### Template for New Sister

```rust
// new_sister/Cargo.toml
[dependencies]
agentic-contracts = "0.1"

// new_sister/src/lib.rs
use agentic_contracts::prelude::*;

pub struct AgenticTime {
    config: SisterConfig,
    context_manager: ContextManager,
    event_manager: EventManager,
    // ...
}

impl Sister for AgenticTime {
    const SISTER_TYPE: SisterType = SisterType::Time;
    const FILE_EXTENSION: &'static str = "atime";
    
    fn init(config: SisterConfig) -> SisterResult<Self> { ... }
    fn health(&self) -> HealthStatus { ... }
    fn version(&self) -> Version { Version::new(0, 1, 0) }
    fn shutdown(&mut self) -> SisterResult<()> { ... }
    fn capabilities(&self) -> Vec<Capability> { ... }
}

impl ContextManagement for AgenticTime { ... }
impl Grounding for AgenticTime { ... }
impl EventEmitter for AgenticTime { ... }
impl Queryable for AgenticTime { ... }
impl ReceiptIntegration for AgenticTime { ... }
```

### Compliance Checklist for New Sisters

```
BEFORE RELEASE, NEW SISTER MUST:
────────────────────────────────

□ Depend on agentic-contracts
□ Implement Sister trait
□ Implement ContextManagement trait
□ Implement Grounding trait
□ Implement EventEmitter trait
□ Implement Queryable trait
□ Implement ReceiptIntegration trait

□ File format uses SisterFileHeader
□ All errors use SisterError
□ All events use SisterEvent

□ MCP server exposes:
  □ {sister}_health
  □ {sister}_info
  □ {sister}_context_create
  □ {sister}_context_switch
  □ {sister}_context_current
  □ {sister}_context_list
  □ {sister}_context_delete
  □ {sister}_ground
  □ {sister}_query

□ Tests pass
□ Documentation complete
```

---

## Next Sisters (After Verification)

```
READY TO BUILD (In Order):
──────────────────────────

1. AgenticTime (.atime)
   → Temporal reasoning
   → Duration modeling
   → Deadline tracking
   → Decay modeling

2. AgenticContract (.acon)
   → Policy boundaries
   → User-defined limits
   → Approval rules
   → Risk limits

3. AgenticComm (.acomm)
   → Agent-to-agent protocol
   → Encrypted messaging
   → Capability negotiation

4. AgenticPlanning (.aplan)
   → Persistent goals
   → Progress metrics
   → Decision alignment

5. AgenticCognition (.acog)
   → User modeling
   → Decision patterns
   → Belief tracking

6. AgenticReality (.areal)
   → External world
   → Business context
   → Probabilistic reasoning

EACH FOLLOWS THE TEMPLATE FROM DAY ONE.
NO RETROFITTING NEEDED.
```

---

## Quick Reference: Contract Documents

```
CANONICAL DOCUMENTS:
────────────────────

📄 SISTER-HYDRA-INTEGRATION-CONTRACT.md
   → Native Rust integration
   → Trait definitions
   → File format spec
   → 20-year compatibility

📄 MCP-TOOL-STANDARDS.md
   → MCP tool naming
   → Required tools
   → JSON schemas
   → Error responses

📦 agentic-contracts/
   → Rust crate
   → Single source of truth
   → All sisters depend on this
```

---

*Document Version: 1.0*
*Action Required: Verify existing sisters before building more*
