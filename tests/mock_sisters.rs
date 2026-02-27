//! Mock sister implementations validating v0.2.0 contracts.
//!
//! These mocks prove that every trait in agentic-sdk
//! can be implemented by real sisters. Each mock mirrors
//! the actual pattern used by that sister type.
//!
//! Pattern coverage:
//! - MockMemory:   Sister + SessionManagement + Grounding + Queryable + EventEmitter
//! - MockCodebase: Sister + WorkspaceManagement + Grounding + Queryable
//! - MockIdentity: Sister + SessionManagement + Grounding + ReceiptIntegration
//! - MockTime:     Sister only (stateless — no sessions, no grounding)

use agentic_sdk::prelude::*;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

// ═══════════════════════════════════════════════════════════════════
// MOCK MEMORY — Session-based sister with grounding
// ═══════════════════════════════════════════════════════════════════

struct MockMemory {
    start_time: Instant,
    session_id: Mutex<Option<ContextId>>,
    sessions: Mutex<Vec<ContextSummary>>,
    events: EventManager,
    nodes: Mutex<Vec<(u64, String)>>, // (id, content)
    next_id: Mutex<u64>,
}

impl MockMemory {
    fn new(_config: SisterConfig) -> SisterResult<Self> {
        Ok(Self {
            start_time: Instant::now(),
            session_id: Mutex::new(None),
            sessions: Mutex::new(vec![]),
            events: EventManager::new(256),
            nodes: Mutex::new(vec![]),
            next_id: Mutex::new(1),
        })
    }

    fn add_node(&self, content: &str) -> u64 {
        let mut nodes = self.nodes.lock().unwrap();
        let mut next = self.next_id.lock().unwrap();
        let id = *next;
        *next += 1;
        nodes.push((id, content.to_string()));
        id
    }
}

impl Sister for MockMemory {
    const SISTER_TYPE: SisterType = SisterType::Memory;
    const FILE_EXTENSION: &'static str = "amem";

    fn init(config: SisterConfig) -> SisterResult<Self>
    where
        Self: Sized,
    {
        MockMemory::new(config)
    }

    fn health(&self) -> HealthStatus {
        HealthStatus {
            healthy: true,
            status: Status::Ready,
            uptime: self.start_time.elapsed(),
            resources: ResourceUsage::default(),
            warnings: vec![],
            last_error: None,
        }
    }

    fn version(&self) -> Version {
        Version::new(0, 2, 0)
    }

    fn shutdown(&mut self) -> SisterResult<()> {
        self.events
            .emit(SisterEvent::shutting_down(SisterType::Memory));
        Ok(())
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability::new("memory_add", "Add cognitive events to graph"),
            Capability::new("memory_query", "Query memory by filters"),
            Capability::new("memory_ground", "Verify claims against stored memories"),
            Capability::new("memory_similar", "Find semantically similar memories"),
        ]
    }
}

impl SessionManagement for MockMemory {
    fn start_session(&mut self, name: &str) -> SisterResult<ContextId> {
        let id = ContextId::new();
        *self.session_id.lock().unwrap() = Some(id);

        let summary = ContextSummary {
            id,
            name: name.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            item_count: 0,
            size_bytes: 0,
        };
        self.sessions.lock().unwrap().push(summary);

        self.events.emit(SisterEvent::context_created(
            SisterType::Memory,
            id,
            name.to_string(),
        ));
        Ok(id)
    }

    fn end_session(&mut self) -> SisterResult<()> {
        *self.session_id.lock().unwrap() = None;
        Ok(())
    }

    fn current_session(&self) -> Option<ContextId> {
        *self.session_id.lock().unwrap()
    }

    fn current_session_info(&self) -> SisterResult<ContextInfo> {
        let id = self
            .current_session()
            .ok_or_else(|| SisterError::new(ErrorCode::InvalidState, "No active session"))?;

        Ok(ContextInfo {
            id,
            name: "active".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            item_count: self.nodes.lock().unwrap().len(),
            size_bytes: 0,
            metadata: Metadata::new(),
        })
    }

    fn list_sessions(&self) -> SisterResult<Vec<ContextSummary>> {
        Ok(self.sessions.lock().unwrap().clone())
    }

    fn export_session(&self, _id: ContextId) -> SisterResult<ContextSnapshot> {
        let info = self.current_session_info()?;
        let data = serde_json::to_vec(&self.nodes.lock().unwrap().clone())
            .map_err(|e| SisterError::new(ErrorCode::Internal, e.to_string()))?;
        let checksum = *blake3::hash(&data).as_bytes();

        Ok(ContextSnapshot {
            sister_type: SisterType::Memory,
            version: Version::new(0, 2, 0),
            context_info: info,
            data,
            checksum,
            snapshot_at: Utc::now(),
        })
    }

    fn import_session(&mut self, snapshot: ContextSnapshot) -> SisterResult<ContextId> {
        if !snapshot.verify() {
            return Err(SisterError::new(
                ErrorCode::ChecksumMismatch,
                "Snapshot checksum verification failed",
            ));
        }
        self.start_session(&snapshot.context_info.name)
    }
}

impl Grounding for MockMemory {
    fn ground(&self, claim: &str) -> SisterResult<GroundingResult> {
        let nodes = self.nodes.lock().unwrap();
        let claim_lower = claim.to_lowercase();

        let matches: Vec<_> = nodes
            .iter()
            .filter(|(_, content)| content.to_lowercase().contains(&claim_lower))
            .collect();

        if matches.is_empty() {
            Ok(
                GroundingResult::ungrounded(claim, "No matching memories found")
                    .with_suggestions(nodes.iter().take(3).map(|(_, c)| c.clone()).collect()),
            )
        } else {
            // BM25-like: best match score matters, not ratio
            let best_score = matches
                .iter()
                .map(|(_, content)| {
                    let claim_words: Vec<&str> = claim_lower.split_whitespace().collect();
                    let content_lower = content.to_lowercase();
                    let matched = claim_words
                        .iter()
                        .filter(|w| content_lower.contains(**w))
                        .count();
                    matched as f64 / claim_words.len().max(1) as f64
                })
                .fold(0.0f64, |a, b| a.max(b));

            let evidence = matches
                .iter()
                .map(|(id, content)| {
                    GroundingEvidence::new(
                        "memory_node",
                        format!("node_{}", id),
                        best_score,
                        content,
                    )
                })
                .collect();

            if best_score > 0.5 {
                Ok(GroundingResult::verified(claim, best_score)
                    .with_evidence(evidence)
                    .with_reason("Found matching memories"))
            } else {
                Ok(GroundingResult::partial(claim, best_score)
                    .with_evidence(evidence)
                    .with_reason("Some evidence found"))
            }
        }
    }

    fn evidence(&self, query: &str, max_results: usize) -> SisterResult<Vec<EvidenceDetail>> {
        let nodes = self.nodes.lock().unwrap();
        let query_lower = query.to_lowercase();

        Ok(nodes
            .iter()
            .filter(|(_, content)| content.to_lowercase().contains(&query_lower))
            .take(max_results)
            .map(|(id, content)| EvidenceDetail {
                evidence_type: "memory_node".to_string(),
                id: format!("node_{}", id),
                score: 0.8,
                created_at: Utc::now(),
                source_sister: SisterType::Memory,
                content: content.clone(),
                data: Metadata::new(),
            })
            .collect())
    }

    fn suggest(&self, query: &str, limit: usize) -> SisterResult<Vec<GroundingSuggestion>> {
        let nodes = self.nodes.lock().unwrap();
        let _query_lower = query.to_lowercase();

        Ok(nodes
            .iter()
            .take(limit)
            .map(|(id, content)| GroundingSuggestion {
                item_type: "memory_node".to_string(),
                id: format!("node_{}", id),
                relevance_score: 0.5,
                description: content.clone(),
                data: Metadata::new(),
            })
            .collect())
    }
}

impl Queryable for MockMemory {
    fn query(&self, query: Query) -> SisterResult<QueryResult> {
        let start = Instant::now();
        let nodes = self.nodes.lock().unwrap();

        let results: Vec<serde_json::Value> = match query.query_type.as_str() {
            "list" => nodes
                .iter()
                .skip(query.offset.unwrap_or(0))
                .take(query.limit.unwrap_or(20))
                .map(|(id, content)| serde_json::json!({"id": id, "content": content}))
                .collect(),
            "search" => {
                let text = query.get_string("text").unwrap_or_default().to_lowercase();
                nodes
                    .iter()
                    .filter(|(_, content)| content.to_lowercase().contains(&text))
                    .take(query.limit.unwrap_or(20))
                    .map(|(id, content)| serde_json::json!({"id": id, "content": content}))
                    .collect()
            }
            "recent" => nodes
                .iter()
                .rev()
                .take(query.limit.unwrap_or(10))
                .map(|(id, content)| serde_json::json!({"id": id, "content": content}))
                .collect(),
            _ => vec![],
        };

        Ok(QueryResult::new(query, results, start.elapsed()))
    }

    fn supports_query(&self, query_type: &str) -> bool {
        matches!(
            query_type,
            "list" | "search" | "recent" | "related" | "temporal"
        )
    }

    fn query_types(&self) -> Vec<QueryTypeInfo> {
        vec![
            QueryTypeInfo::new("list", "List all memory nodes"),
            QueryTypeInfo::new("search", "Search memories by text").required(vec!["text"]),
            QueryTypeInfo::new("recent", "Get most recent memories"),
        ]
    }
}

impl EventEmitter for MockMemory {
    fn subscribe(&self, _filter: EventFilter) -> EventReceiver {
        self.events.subscribe()
    }

    fn recent_events(&self, limit: usize) -> Vec<SisterEvent> {
        self.events.recent(limit)
    }

    fn emit(&self, event: SisterEvent) {
        self.events.emit(event);
    }
}

// ═══════════════════════════════════════════════════════════════════
// MOCK CODEBASE — Workspace-based sister
// ═══════════════════════════════════════════════════════════════════

type SymbolList = Vec<(String, String)>; // (name, kind)
type WorkspaceData = (String, SymbolList); // (workspace_name, symbols)

struct MockCodebase {
    start_time: Instant,
    current_workspace: Mutex<ContextId>,
    workspaces: Mutex<HashMap<ContextId, WorkspaceData>>,
}

impl MockCodebase {
    fn new(_config: SisterConfig) -> SisterResult<Self> {
        let default_id = ContextId::default_context();
        let mut workspaces = HashMap::new();
        workspaces.insert(default_id, ("default".to_string(), vec![]));

        Ok(Self {
            start_time: Instant::now(),
            current_workspace: Mutex::new(default_id),
            workspaces: Mutex::new(workspaces),
        })
    }

    fn add_symbol(&self, name: &str, kind: &str) {
        let ws_id = *self.current_workspace.lock().unwrap();
        let mut workspaces = self.workspaces.lock().unwrap();
        if let Some((_, symbols)) = workspaces.get_mut(&ws_id) {
            symbols.push((name.to_string(), kind.to_string()));
        }
    }
}

impl Sister for MockCodebase {
    const SISTER_TYPE: SisterType = SisterType::Codebase;
    const FILE_EXTENSION: &'static str = "acb";

    fn init(config: SisterConfig) -> SisterResult<Self>
    where
        Self: Sized,
    {
        MockCodebase::new(config)
    }

    fn health(&self) -> HealthStatus {
        HealthStatus {
            healthy: true,
            status: Status::Ready,
            uptime: self.start_time.elapsed(),
            resources: ResourceUsage::default(),
            warnings: vec![],
            last_error: None,
        }
    }

    fn version(&self) -> Version {
        Version::new(0, 2, 0)
    }

    fn shutdown(&mut self) -> SisterResult<()> {
        Ok(())
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability::new("symbol_lookup", "Look up symbols by name"),
            Capability::new("impact_analysis", "Analyse change impact"),
            Capability::new("codebase_ground", "Verify code claims"),
        ]
    }
}

impl WorkspaceManagement for MockCodebase {
    fn create_workspace(&mut self, name: &str) -> SisterResult<ContextId> {
        let id = ContextId::new();
        self.workspaces
            .lock()
            .unwrap()
            .insert(id, (name.to_string(), vec![]));
        Ok(id)
    }

    fn switch_workspace(&mut self, id: ContextId) -> SisterResult<()> {
        let workspaces = self.workspaces.lock().unwrap();
        if !workspaces.contains_key(&id) {
            return Err(SisterError::context_not_found(id.to_string()));
        }
        drop(workspaces);
        *self.current_workspace.lock().unwrap() = id;
        Ok(())
    }

    fn current_workspace(&self) -> ContextId {
        *self.current_workspace.lock().unwrap()
    }

    fn current_workspace_info(&self) -> SisterResult<ContextInfo> {
        let ws_id = self.current_workspace();
        let workspaces = self.workspaces.lock().unwrap();
        let (name, symbols) = workspaces
            .get(&ws_id)
            .ok_or_else(|| SisterError::context_not_found(ws_id.to_string()))?;

        Ok(ContextInfo {
            id: ws_id,
            name: name.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            item_count: symbols.len(),
            size_bytes: 0,
            metadata: Metadata::new(),
        })
    }

    fn list_workspaces(&self) -> SisterResult<Vec<ContextSummary>> {
        let workspaces = self.workspaces.lock().unwrap();
        Ok(workspaces
            .iter()
            .map(|(id, (name, symbols))| ContextSummary {
                id: *id,
                name: name.clone(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                item_count: symbols.len(),
                size_bytes: 0,
            })
            .collect())
    }

    fn delete_workspace(&mut self, id: ContextId) -> SisterResult<()> {
        let current = *self.current_workspace.lock().unwrap();
        if id == current {
            return Err(SisterError::new(
                ErrorCode::InvalidState,
                "Cannot delete active workspace. Switch first",
            ));
        }
        self.workspaces.lock().unwrap().remove(&id);
        Ok(())
    }

    fn rename_workspace(&mut self, id: ContextId, new_name: &str) -> SisterResult<()> {
        let mut workspaces = self.workspaces.lock().unwrap();
        if let Some((name, _)) = workspaces.get_mut(&id) {
            *name = new_name.to_string();
            Ok(())
        } else {
            Err(SisterError::context_not_found(id.to_string()))
        }
    }

    fn export_workspace(&self, id: ContextId) -> SisterResult<ContextSnapshot> {
        let workspaces = self.workspaces.lock().unwrap();
        let (name, symbols) = workspaces
            .get(&id)
            .ok_or_else(|| SisterError::context_not_found(id.to_string()))?;

        let data = serde_json::to_vec(&symbols)
            .map_err(|e| SisterError::new(ErrorCode::Internal, e.to_string()))?;
        let checksum = *blake3::hash(&data).as_bytes();

        Ok(ContextSnapshot {
            sister_type: SisterType::Codebase,
            version: Version::new(0, 2, 0),
            context_info: ContextInfo {
                id,
                name: name.clone(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                item_count: symbols.len(),
                size_bytes: data.len(),
                metadata: Metadata::new(),
            },
            data,
            checksum,
            snapshot_at: Utc::now(),
        })
    }

    fn import_workspace(&mut self, snapshot: ContextSnapshot) -> SisterResult<ContextId> {
        if !snapshot.verify() {
            return Err(SisterError::new(
                ErrorCode::ChecksumMismatch,
                "Snapshot checksum failed",
            ));
        }
        self.create_workspace(&snapshot.context_info.name)
    }
}

impl Grounding for MockCodebase {
    fn ground(&self, claim: &str) -> SisterResult<GroundingResult> {
        let ws_id = self.current_workspace();
        let workspaces = self.workspaces.lock().unwrap();
        let (_, symbols) = workspaces
            .get(&ws_id)
            .ok_or_else(|| SisterError::context_not_found(ws_id.to_string()))?;

        let claim_lower = claim.to_lowercase();
        let matches: Vec<_> = symbols
            .iter()
            .filter(|(name, _)| claim_lower.contains(&name.to_lowercase()))
            .collect();

        if matches.is_empty() {
            Ok(GroundingResult::ungrounded(
                claim,
                "Symbol not found in graph",
            ))
        } else {
            let evidence = matches
                .iter()
                .map(|(name, kind)| {
                    GroundingEvidence::new("code_symbol", name, 0.9, format!("{}: {}", kind, name))
                })
                .collect();
            Ok(GroundingResult::verified(claim, 0.9)
                .with_evidence(evidence)
                .with_reason("Symbol found in code graph"))
        }
    }

    fn evidence(&self, query: &str, max_results: usize) -> SisterResult<Vec<EvidenceDetail>> {
        let ws_id = self.current_workspace();
        let workspaces = self.workspaces.lock().unwrap();
        let (_, symbols) = workspaces
            .get(&ws_id)
            .ok_or_else(|| SisterError::context_not_found(ws_id.to_string()))?;

        let query_lower = query.to_lowercase();
        Ok(symbols
            .iter()
            .filter(|(name, _)| name.to_lowercase().contains(&query_lower))
            .take(max_results)
            .map(|(name, kind)| EvidenceDetail {
                evidence_type: "code_symbol".to_string(),
                id: name.clone(),
                score: 0.9,
                created_at: Utc::now(),
                source_sister: SisterType::Codebase,
                content: format!("{} {}", kind, name),
                data: Metadata::new(),
            })
            .collect())
    }

    fn suggest(&self, query: &str, limit: usize) -> SisterResult<Vec<GroundingSuggestion>> {
        let ws_id = self.current_workspace();
        let workspaces = self.workspaces.lock().unwrap();
        let (_, symbols) = workspaces
            .get(&ws_id)
            .ok_or_else(|| SisterError::context_not_found(ws_id.to_string()))?;

        let _query_lower = query.to_lowercase();
        Ok(symbols
            .iter()
            .take(limit)
            .map(|(name, kind)| GroundingSuggestion {
                item_type: "code_symbol".to_string(),
                id: name.clone(),
                relevance_score: 0.5,
                description: format!("{}: {}", kind, name),
                data: Metadata::new(),
            })
            .collect())
    }
}

impl Queryable for MockCodebase {
    fn query(&self, query: Query) -> SisterResult<QueryResult> {
        let start = Instant::now();
        let ws_id = self.current_workspace();
        let workspaces = self.workspaces.lock().unwrap();
        let (_, symbols) = workspaces
            .get(&ws_id)
            .ok_or_else(|| SisterError::context_not_found(ws_id.to_string()))?;

        let results: Vec<serde_json::Value> = match query.query_type.as_str() {
            "list" => symbols
                .iter()
                .take(query.limit.unwrap_or(50))
                .map(|(name, kind)| serde_json::json!({"name": name, "kind": kind}))
                .collect(),
            "search" => {
                let text = query.get_string("text").unwrap_or_default().to_lowercase();
                symbols
                    .iter()
                    .filter(|(name, _)| name.to_lowercase().contains(&text))
                    .take(query.limit.unwrap_or(20))
                    .map(|(name, kind)| serde_json::json!({"name": name, "kind": kind}))
                    .collect()
            }
            _ => vec![],
        };

        Ok(QueryResult::new(query, results, start.elapsed()))
    }

    fn supports_query(&self, query_type: &str) -> bool {
        matches!(query_type, "list" | "search" | "get")
    }

    fn query_types(&self) -> Vec<QueryTypeInfo> {
        vec![
            QueryTypeInfo::new("list", "List all code symbols"),
            QueryTypeInfo::new("search", "Search symbols by name").required(vec!["text"]),
        ]
    }
}

// ═══════════════════════════════════════════════════════════════════
// MOCK IDENTITY — Session-based with receipts
// ═══════════════════════════════════════════════════════════════════

struct MockIdentity {
    start_time: Instant,
    session_id: Mutex<Option<ContextId>>,
    receipts: Mutex<Vec<Receipt>>,
    chain_position: Mutex<u64>,
}

impl MockIdentity {
    fn new(_config: SisterConfig) -> SisterResult<Self> {
        Ok(Self {
            start_time: Instant::now(),
            session_id: Mutex::new(None),
            receipts: Mutex::new(vec![]),
            chain_position: Mutex::new(0),
        })
    }
}

impl Sister for MockIdentity {
    const SISTER_TYPE: SisterType = SisterType::Identity;
    const FILE_EXTENSION: &'static str = "aid";

    fn init(config: SisterConfig) -> SisterResult<Self>
    where
        Self: Sized,
    {
        MockIdentity::new(config)
    }

    fn health(&self) -> HealthStatus {
        HealthStatus {
            healthy: true,
            status: Status::Ready,
            uptime: self.start_time.elapsed(),
            resources: ResourceUsage::default(),
            warnings: vec![],
            last_error: None,
        }
    }

    fn version(&self) -> Version {
        Version::new(0, 2, 0)
    }

    fn shutdown(&mut self) -> SisterResult<()> {
        Ok(())
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability::new("identity_create", "Create cryptographic identity"),
            Capability::new("action_sign", "Sign actions with receipt chain"),
            Capability::new("trust_grant", "Grant trust to other identities"),
        ]
    }
}

impl SessionManagement for MockIdentity {
    fn start_session(&mut self, _name: &str) -> SisterResult<ContextId> {
        let id = ContextId::new();
        *self.session_id.lock().unwrap() = Some(id);
        Ok(id)
    }

    fn end_session(&mut self) -> SisterResult<()> {
        *self.session_id.lock().unwrap() = None;
        Ok(())
    }

    fn current_session(&self) -> Option<ContextId> {
        *self.session_id.lock().unwrap()
    }

    fn current_session_info(&self) -> SisterResult<ContextInfo> {
        let id = self
            .current_session()
            .ok_or_else(|| SisterError::new(ErrorCode::InvalidState, "No active session"))?;
        Ok(ContextInfo {
            id,
            name: "identity_session".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            item_count: self.receipts.lock().unwrap().len(),
            size_bytes: 0,
            metadata: Metadata::new(),
        })
    }

    fn list_sessions(&self) -> SisterResult<Vec<ContextSummary>> {
        Ok(vec![])
    }

    fn export_session(&self, _id: ContextId) -> SisterResult<ContextSnapshot> {
        Err(SisterError::new(
            ErrorCode::NotImplemented,
            "Identity does not support session export",
        ))
    }

    fn import_session(&mut self, _snapshot: ContextSnapshot) -> SisterResult<ContextId> {
        Err(SisterError::new(
            ErrorCode::NotImplemented,
            "Identity does not support session import",
        ))
    }
}

impl ReceiptIntegration for MockIdentity {
    fn create_receipt(&self, action: ActionRecord) -> SisterResult<ReceiptId> {
        let receipt_id = ReceiptId::new();
        let mut position = self.chain_position.lock().unwrap();
        *position += 1;

        let receipt = Receipt {
            id: receipt_id,
            action,
            signature: "mock_ed25519_signature".to_string(),
            chain_position: *position,
            previous_hash: "0000000000000000".to_string(),
            hash: format!("hash_{}", position),
            created_at: Utc::now(),
        };

        self.receipts.lock().unwrap().push(receipt);
        Ok(receipt_id)
    }

    fn get_receipt(&self, id: ReceiptId) -> SisterResult<Receipt> {
        self.receipts
            .lock()
            .unwrap()
            .iter()
            .find(|r| r.id == id)
            .cloned()
            .ok_or_else(|| SisterError::not_found(format!("Receipt {}", id)))
    }

    fn list_receipts(&self, filter: ReceiptFilter) -> SisterResult<Vec<Receipt>> {
        let receipts = self.receipts.lock().unwrap();
        let mut results: Vec<_> = receipts
            .iter()
            .filter(|r| {
                if let Some(ref st) = filter.sister_type {
                    if r.action.sister_type != *st {
                        return false;
                    }
                }
                if let Some(ref at) = filter.action_type {
                    if r.action.action_type != *at {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        if let Some(limit) = filter.limit {
            results.truncate(limit);
        }
        Ok(results)
    }
}

impl Grounding for MockIdentity {
    fn ground(&self, claim: &str) -> SisterResult<GroundingResult> {
        let receipts = self.receipts.lock().unwrap();
        let claim_lower = claim.to_lowercase();

        let matches: Vec<_> = receipts
            .iter()
            .filter(|r| {
                r.action.action_type.to_lowercase().contains(&claim_lower)
                    || claim_lower.contains(&r.action.action_type.to_lowercase())
            })
            .collect();

        if matches.is_empty() {
            Ok(GroundingResult::ungrounded(claim, "No matching receipts"))
        } else {
            let evidence = matches
                .iter()
                .map(|r| {
                    GroundingEvidence::new(
                        "receipt",
                        r.id.to_string(),
                        0.9,
                        format!("Receipt for {}", r.action.action_type),
                    )
                })
                .collect();
            Ok(GroundingResult::verified(claim, 0.9).with_evidence(evidence))
        }
    }

    fn evidence(&self, query: &str, max_results: usize) -> SisterResult<Vec<EvidenceDetail>> {
        let receipts = self.receipts.lock().unwrap();
        let query_lower = query.to_lowercase();

        Ok(receipts
            .iter()
            .filter(|r| r.action.action_type.to_lowercase().contains(&query_lower))
            .take(max_results)
            .map(|r| EvidenceDetail {
                evidence_type: "receipt".to_string(),
                id: r.id.to_string(),
                score: 0.9,
                created_at: r.created_at,
                source_sister: SisterType::Identity,
                content: format!("{} (chain pos {})", r.action.action_type, r.chain_position),
                data: Metadata::new(),
            })
            .collect())
    }

    fn suggest(&self, _query: &str, limit: usize) -> SisterResult<Vec<GroundingSuggestion>> {
        let receipts = self.receipts.lock().unwrap();
        Ok(receipts
            .iter()
            .take(limit)
            .map(|r| GroundingSuggestion {
                item_type: "receipt".to_string(),
                id: r.id.to_string(),
                relevance_score: 0.5,
                description: format!("Action: {}", r.action.action_type),
                data: Metadata::new(),
            })
            .collect())
    }
}

// ═══════════════════════════════════════════════════════════════════
// MOCK TIME — Stateless sister (no sessions, no grounding)
// ═══════════════════════════════════════════════════════════════════

struct MockTime {
    start_time: Instant,
}

impl Sister for MockTime {
    const SISTER_TYPE: SisterType = SisterType::Time;
    const FILE_EXTENSION: &'static str = "atime";

    fn init(_config: SisterConfig) -> SisterResult<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            start_time: Instant::now(),
        })
    }

    fn health(&self) -> HealthStatus {
        HealthStatus {
            healthy: true,
            status: Status::Ready,
            uptime: self.start_time.elapsed(),
            resources: ResourceUsage::default(),
            warnings: vec![],
            last_error: None,
        }
    }

    fn version(&self) -> Version {
        Version::new(0, 2, 0)
    }

    fn shutdown(&mut self) -> SisterResult<()> {
        Ok(())
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability::new("time_now", "Get current time in any timezone"),
            Capability::new("time_duration", "Calculate duration between events"),
        ]
    }
}

// Time is stateless — no SessionManagement, no WorkspaceManagement, no Grounding

impl Queryable for MockTime {
    fn query(&self, query: Query) -> SisterResult<QueryResult> {
        let start = Instant::now();
        let results = match query.query_type.as_str() {
            "current_time" => {
                vec![serde_json::json!({"time": Utc::now().to_rfc3339()})]
            }
            _ => vec![],
        };
        Ok(QueryResult::new(query, results, start.elapsed()))
    }

    fn supports_query(&self, query_type: &str) -> bool {
        matches!(query_type, "current_time" | "duration")
    }

    fn query_types(&self) -> Vec<QueryTypeInfo> {
        vec![QueryTypeInfo::new("current_time", "Get current UTC time")]
    }
}

// ═══════════════════════════════════════════════════════════════════
// TESTS — Validate all trait compositions compile and work
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_memory_lifecycle() {
    let config = SisterConfig::new("/tmp/mock-memory");
    let mut memory = MockMemory::init(config).unwrap();

    assert!(memory.is_healthy());
    assert_eq!(memory.sister_type(), SisterType::Memory);
    assert_eq!(memory.file_extension(), "amem");
    assert_eq!(memory.mcp_prefix(), "memory");
    assert_eq!(memory.version(), Version::new(0, 2, 0));

    // SisterInfo can be created from any sister
    let info = SisterInfo::from_sister(&memory);
    assert_eq!(info.sister_type, SisterType::Memory);

    memory.shutdown().unwrap();
}

#[test]
fn test_memory_sessions() {
    let config = SisterConfig::new("/tmp/mock-memory");
    let mut memory = MockMemory::init(config).unwrap();

    assert!(memory.current_session().is_none());

    let session_id = memory.start_session("test_session").unwrap();
    assert!(memory.current_session().is_some());
    assert_eq!(memory.current_session().unwrap(), session_id);

    let info = memory.current_session_info().unwrap();
    assert_eq!(info.id, session_id);

    memory.end_session().unwrap();
    assert!(memory.current_session().is_none());
}

#[test]
fn test_memory_grounding() {
    let config = SisterConfig::new("/tmp/mock-memory");
    let mut memory = MockMemory::init(config).unwrap();
    memory.start_session("grounding_test").unwrap();

    memory.add_node("The sky is blue");
    memory.add_node("Rust is fast");
    memory.add_node("Memory sisters store cognitive events");

    // Verified claim
    let result = memory.ground("sky is blue").unwrap();
    assert_eq!(result.status, GroundingStatus::Verified);
    assert!(!result.evidence.is_empty());

    // Ungrounded claim
    let result = memory.ground("cats can teleport").unwrap();
    assert_eq!(result.status, GroundingStatus::Ungrounded);
    assert!(result.suggestions.len() <= 3);

    // Evidence query
    let evidence = memory.evidence("rust", 10).unwrap();
    assert_eq!(evidence.len(), 1);
    assert_eq!(evidence[0].source_sister, SisterType::Memory);

    // Suggest
    let suggestions = memory.suggest("anything", 5).unwrap();
    assert!(!suggestions.is_empty());
}

#[test]
fn test_memory_queryable() {
    let config = SisterConfig::new("/tmp/mock-memory");
    let mut memory = MockMemory::init(config).unwrap();
    memory.start_session("query_test").unwrap();

    memory.add_node("First memory");
    memory.add_node("Second memory");
    memory.add_node("Third memory");

    // List query
    let result = memory.query(Query::list().limit(2)).unwrap();
    assert_eq!(result.len(), 2);

    // Search query
    let result = memory.search("second").unwrap();
    assert_eq!(result.len(), 1);

    // Recent query
    let result = memory.recent(2).unwrap();
    assert_eq!(result.len(), 2);

    // Supports query
    assert!(memory.supports_query("list"));
    assert!(memory.supports_query("search"));
    assert!(!memory.supports_query("unknown"));

    // Query types
    let types = memory.query_types();
    assert_eq!(types.len(), 3);
}

#[test]
fn test_memory_events() {
    let config = SisterConfig::new("/tmp/mock-memory");
    let mut memory = MockMemory::init(config).unwrap();

    // Start session emits event
    memory.start_session("event_test").unwrap();

    let events = memory.recent_events(10);
    assert!(!events.is_empty());

    // Event filter
    let filter = EventFilter::new().for_sister(SisterType::Memory);
    assert!(filter.matches(&events[0]));

    let wrong_filter = EventFilter::new().for_sister(SisterType::Vision);
    assert!(!wrong_filter.matches(&events[0]));
}

#[test]
fn test_memory_snapshot_export_import() {
    let config = SisterConfig::new("/tmp/mock-memory");
    let mut memory = MockMemory::init(config).unwrap();

    let session_id = memory.start_session("snapshot_test").unwrap();
    memory.add_node("Important memory");

    let snapshot = memory.export_session(session_id).unwrap();
    assert!(snapshot.verify()); // BLAKE3 checksum passes
    assert_eq!(snapshot.sister_type, SisterType::Memory);
    assert_eq!(snapshot.version, Version::new(0, 2, 0));

    // Import into fresh instance
    let config2 = SisterConfig::new("/tmp/mock-memory2");
    let mut memory2 = MockMemory::init(config2).unwrap();
    let imported_id = memory2.import_session(snapshot).unwrap();
    assert!(memory2.current_session().is_some());
    assert_eq!(memory2.current_session().unwrap(), imported_id);
}

#[test]
fn test_codebase_workspaces() {
    let config = SisterConfig::default().add_path("default_graph", "/tmp/mock.acb");
    let mut codebase = MockCodebase::init(config).unwrap();

    assert_eq!(codebase.sister_type(), SisterType::Codebase);

    // Default workspace exists
    let workspaces = codebase.list_workspaces().unwrap();
    assert_eq!(workspaces.len(), 1);

    // Create and switch
    let ws2 = codebase.create_workspace("feature-branch").unwrap();
    codebase.switch_workspace(ws2).unwrap();
    assert_eq!(codebase.current_workspace(), ws2);

    // Add symbols to workspace
    codebase.add_symbol("validate_token", "function");
    codebase.add_symbol("User", "struct");

    let info = codebase.current_workspace_info().unwrap();
    assert_eq!(info.item_count, 2);

    // Rename
    codebase.rename_workspace(ws2, "renamed-branch").unwrap();
    let info2 = codebase.current_workspace_info().unwrap();
    assert_eq!(info2.name, "renamed-branch");

    // Can't delete active workspace
    assert!(codebase.delete_workspace(ws2).is_err());

    // Switch back and delete
    let default = ContextId::default_context();
    codebase.switch_workspace(default).unwrap();
    codebase.delete_workspace(ws2).unwrap();
    assert_eq!(codebase.list_workspaces().unwrap().len(), 1);
}

#[test]
fn test_codebase_grounding() {
    let config = SisterConfig::default();
    let codebase = MockCodebase::init(config).unwrap();

    codebase.add_symbol("validate_token", "function");
    codebase.add_symbol("User", "struct");

    let result = codebase.ground("validate_token exists").unwrap();
    assert_eq!(result.status, GroundingStatus::Verified);

    let result = codebase.ground("nonexistent_fn").unwrap();
    assert_eq!(result.status, GroundingStatus::Ungrounded);
}

#[test]
fn test_codebase_workspace_export() {
    let config = SisterConfig::default();
    let codebase = MockCodebase::init(config).unwrap();

    codebase.add_symbol("main", "function");

    let default_ws = codebase.current_workspace();
    let snapshot = codebase.export_workspace(default_ws).unwrap();
    assert!(snapshot.verify());
    assert_eq!(snapshot.sister_type, SisterType::Codebase);
}

#[test]
fn test_identity_receipt_chain() {
    let config = SisterConfig::with_paths({
        let mut paths = HashMap::new();
        paths.insert("identities".to_string(), "/tmp/mock-ids".into());
        paths.insert("receipts".to_string(), "/tmp/mock-receipts".into());
        paths
    });
    let mut identity = MockIdentity::init(config).unwrap();
    identity.start_session("receipt_test").unwrap();

    assert_eq!(identity.sister_type(), SisterType::Identity);
    assert_eq!(identity.file_extension(), "aid");

    // Create receipts
    let action1 = ActionRecord::new(SisterType::Memory, "memory_add", ActionOutcome::success())
        .param("content", "test memory");

    let action2 = ActionBuilder::new(SisterType::Vision, "vision_capture")
        .success_with(serde_json::json!({"capture_id": 42}));

    let receipt_id1 = identity.create_receipt(action1).unwrap();
    let receipt_id2 = identity.create_receipt(action2).unwrap();

    // Get receipt
    let receipt = identity.get_receipt(receipt_id1).unwrap();
    assert_eq!(receipt.action_type(), "memory_add");
    assert!(receipt.was_successful());

    // List receipts with filter
    let all = identity.list_receipts(ReceiptFilter::new()).unwrap();
    assert_eq!(all.len(), 2);

    let memory_only = identity
        .list_receipts(ReceiptFilter::new().for_sister(SisterType::Memory))
        .unwrap();
    assert_eq!(memory_only.len(), 1);

    // Receipt count
    let count = identity.receipt_count().unwrap();
    assert_eq!(count, 2);

    // Receipts for action
    let captures = identity.receipts_for_action("vision_capture").unwrap();
    assert_eq!(captures.len(), 1);
    assert_eq!(captures[0].id, receipt_id2);

    // Grounding verifies receipts
    let result = identity.ground("memory_add").unwrap();
    assert_eq!(result.status, GroundingStatus::Verified);
}

#[test]
fn test_time_stateless() {
    // Time uses stateless config — no data path needed
    let config = SisterConfig::stateless();
    let mut time = MockTime::init(config).unwrap();

    assert_eq!(time.sister_type(), SisterType::Time);
    assert_eq!(time.file_extension(), "atime");
    assert!(time.is_healthy());

    // Time is queryable but doesn't need sessions or grounding
    let result = time.query(Query::new("current_time")).unwrap();
    assert_eq!(result.len(), 1);

    assert!(time.supports_query("current_time"));
    assert!(!time.supports_query("search"));

    time.shutdown().unwrap();
}

#[test]
fn test_sister_config_patterns() {
    // Pattern 1: Single data path (Memory, Vision)
    let config1 = SisterConfig::new("/data/memory.amem")
        .read_only(false)
        .memory_budget(512)
        .option("auto_session", true);

    assert_eq!(
        config1.primary_path(),
        std::path::PathBuf::from("/data/memory.amem")
    );
    assert!(!config1.read_only);
    assert_eq!(config1.memory_budget_mb, Some(512));
    assert_eq!(config1.get_option::<bool>("auto_session"), Some(true));

    // Pattern 2: Stateless (Time)
    let config2 = SisterConfig::stateless();
    assert!(config2.data_path.is_none());
    assert!(config2.data_paths.is_empty());
    assert_eq!(config2.primary_path(), std::path::PathBuf::from("."));

    // Pattern 3: Multiple named paths (Identity)
    let config3 = SisterConfig::default()
        .add_path("identities", "/data/identities")
        .add_path("receipts", "/data/receipts")
        .add_path("trust", "/data/trust")
        .create_if_missing(true);

    assert_eq!(
        config3.get_path("identities"),
        Some(&std::path::PathBuf::from("/data/identities"))
    );
    assert_eq!(config3.data_paths.len(), 3);

    // Pattern 4: From HashMap (Codebase)
    let mut paths = HashMap::new();
    paths.insert("default_graph".to_string(), "/data/project.acb".into());
    let config4 = SisterConfig::with_paths(paths);
    assert!(config4.get_path("default_graph").is_some());
}

#[test]
fn test_error_model_two_layers() {
    // Layer 1: Protocol errors → JSON-RPC error response
    let proto_err = ProtocolError::tool_not_found("memory_nonexistent");
    assert_eq!(proto_err.json_rpc_code(), -32803);
    assert!(proto_err.message.contains("memory_nonexistent"));

    let proto_err2 = ProtocolError::invalid_params("missing required field: claim");
    assert_eq!(proto_err2.json_rpc_code(), -32602);

    // Layer 2: Domain errors → isError: true
    let domain_err = SisterError::not_found("node 42");
    assert_eq!(domain_err.code, ErrorCode::NotFound);
    assert!(domain_err.recoverable);
    let mcp_msg = domain_err.to_mcp_message();
    assert!(mcp_msg.contains("node 42"));
    assert!(mcp_msg.contains("Try:"));

    // Storage error with retry suggestion
    let storage_err = SisterError::storage("disk write failed");
    let msg = storage_err.to_mcp_message();
    assert!(msg.contains("Retry after"));
}

#[test]
fn test_version_compatibility_rules() {
    let v020 = Version::new(0, 2, 0);
    let v010 = Version::new(0, 1, 0);
    let v100 = Version::new(1, 0, 0);

    // Same major = compatible
    assert!(v020.is_compatible_with(&v010));

    // Different major = incompatible
    assert!(!v100.is_compatible_with(&v020));

    // Newer can read older
    assert!(v100.can_read(&v020));
    assert!(!v020.can_read(&v100));

    // VersionCompatibility utility
    assert!(VersionCompatibility::can_read(&v100, &v020));
    assert!(VersionCompatibility::needs_migration(&v100, &v020));
    assert!(VersionCompatibility::is_compatible(&v020, &v010));
}

#[test]
fn test_file_format_magic_identification() {
    assert_eq!(identify_sister_by_magic(b"AMEM"), Some(SisterType::Memory));
    assert_eq!(identify_sister_by_magic(b"AVIS"), Some(SisterType::Vision));
    assert_eq!(
        identify_sister_by_magic(b"ACDB"),
        Some(SisterType::Codebase)
    );
    assert_eq!(identify_sister_by_magic(b"ATIM"), Some(SisterType::Time));
    assert_eq!(identify_sister_by_magic(b"XXXX"), None);

    // v0.1.0 "AGNT" magic is NOT recognized (correctly)
    assert_eq!(identify_sister_by_magic(b"AGNT"), None);
}

#[test]
fn test_hydra_placeholder_types() {
    // SisterSummary composes correctly
    let summary = SisterSummary {
        sister_type: SisterType::Memory,
        status_line: "590 nodes, session 42 active".to_string(),
        item_count: 590,
        active_context: Some("session_42".to_string()),
        metadata: Metadata::new(),
    };
    assert_eq!(summary.sister_type, SisterType::Memory);

    // HydraCommand composes correctly
    let cmd = HydraCommand {
        command_type: "summarize_recent".to_string(),
        params: Metadata::new(),
        run_id: "run_001".to_string(),
        step_id: 1,
    };
    assert_eq!(cmd.command_type, "summarize_recent");

    // CommandResult composes correctly
    let result = CommandResult {
        success: true,
        data: serde_json::json!({"summary": "3 new facts"}),
        error: None,
        evidence_ids: vec!["ev_1".to_string()],
    };
    assert!(result.success);

    // GatedAction + RiskLevel
    let action = GatedAction {
        sister_type: SisterType::Identity,
        action_type: "trust_grant".to_string(),
        risk_level: RiskLevel::High,
        risk_score: 0.7,
        capability: "trust:grant".to_string(),
        requested_at: Utc::now(),
        params: Metadata::new(),
    };
    assert!(action.risk_level >= RiskLevel::Medium);

    // GateDecision
    let decision = GateDecision {
        approved: false,
        reason: "Risk too high without user confirmation".to_string(),
        approval_id: None,
        conditions: vec!["Requires user approval".to_string()],
    };
    assert!(!decision.approved);
}

#[test]
fn test_multi_context_query() {
    // V2 multi-context queries
    let ctx1 = ContextId::new();
    let ctx2 = ContextId::new();

    let query = Query::search("deploy")
        .in_contexts(vec![ctx1, ctx2])
        .limit(20);

    assert!(query.merge_results);
    assert_eq!(query.context_ids.as_ref().unwrap().len(), 2);
}

#[test]
fn test_action_outcomes() {
    let success = ActionOutcome::success();
    assert!(success.is_success());
    assert!(!success.is_failure());

    let with_result = ActionOutcome::success_with(serde_json::json!({"id": 42}));
    assert!(with_result.is_success());

    let failure = ActionOutcome::failure("VALIDATION_ERROR", "Name is required");
    assert!(failure.is_failure());
    assert!(!failure.is_success());

    let partial = ActionOutcome::partial(vec!["Field X was truncated".to_string()]);
    assert!(!partial.is_success());
    assert!(!partial.is_failure());
}
