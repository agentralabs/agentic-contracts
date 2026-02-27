//! Query trait for standard query interface.
//!
//! All sisters support a flexible query interface that allows
//! Hydra and other clients to query data uniformly.

use crate::context::ContextId;
use crate::errors::SisterResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Standard query request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    /// Query type (e.g., "list", "search", "recent", "related", "temporal").
    pub query_type: String,

    /// Query-specific parameters.
    #[serde(default)]
    pub params: HashMap<String, serde_json::Value>,

    /// Maximum results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,

    /// Offset for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,

    /// Context to query in (None = current).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_id: Option<ContextId>,

    /// Multiple contexts for V2 multi-context queries.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_ids: Option<Vec<ContextId>>,

    /// Whether to merge results from multiple contexts.
    #[serde(default)]
    pub merge_results: bool,
}

impl Query {
    /// Create a new query.
    pub fn new(query_type: impl Into<String>) -> Self {
        Self {
            query_type: query_type.into(),
            params: HashMap::new(),
            limit: None,
            offset: None,
            context_id: None,
            context_ids: None,
            merge_results: false,
        }
    }

    /// Add a parameter.
    pub fn param(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(v) = serde_json::to_value(value) {
            self.params.insert(key.into(), v);
        }
        self
    }

    /// Set limit.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set offset.
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Set context.
    pub fn in_context(mut self, context_id: ContextId) -> Self {
        self.context_id = Some(context_id);
        self
    }

    /// Set multiple contexts (V2 multi-context).
    pub fn in_contexts(mut self, context_ids: Vec<ContextId>) -> Self {
        self.context_ids = Some(context_ids);
        self.merge_results = true;
        self
    }

    /// Get a parameter value.
    pub fn get_param<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.params
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Get a string parameter.
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.get_param(key)
    }

    /// Get an integer parameter.
    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.get_param(key)
    }

    /// Get a boolean parameter.
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get_param(key)
    }
}

// Common query types
impl Query {
    /// Create a "list" query.
    pub fn list() -> Self {
        Self::new("list")
    }

    /// Create a "search" query.
    pub fn search(text: impl Into<String>) -> Self {
        Self::new("search").param("text", text.into())
    }

    /// Create a "recent" query.
    pub fn recent(count: usize) -> Self {
        Self::new("recent").limit(count)
    }

    /// Create a "related" query.
    pub fn related(item_id: impl Into<String>) -> Self {
        Self::new("related").param("item_id", item_id.into())
    }

    /// Create a "temporal" query (by time range).
    pub fn temporal() -> Self {
        Self::new("temporal")
    }

    /// Create a "get" query (single item by ID).
    pub fn get(item_id: impl Into<String>) -> Self {
        Self::new("get").param("id", item_id.into())
    }
}

/// Query result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// The query that produced this result.
    pub query: Query,

    /// Results (structure depends on sister and query type).
    pub results: Vec<serde_json::Value>,

    /// Total count (if known).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_count: Option<usize>,

    /// Whether there are more results.
    pub has_more: bool,

    /// Query execution time.
    #[serde(with = "duration_millis")]
    pub query_time: Duration,

    /// Which contexts were queried.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queried_contexts: Option<Vec<ContextId>>,
}

impl QueryResult {
    /// Create a new query result.
    pub fn new(query: Query, results: Vec<serde_json::Value>, query_time: Duration) -> Self {
        Self {
            query,
            total_count: Some(results.len()),
            has_more: false,
            results,
            query_time,
            queried_contexts: None,
        }
    }

    /// Create an empty result.
    pub fn empty(query: Query) -> Self {
        Self {
            query,
            results: vec![],
            total_count: Some(0),
            has_more: false,
            query_time: Duration::ZERO,
            queried_contexts: None,
        }
    }

    /// Set total count and has_more.
    pub fn with_pagination(mut self, total: usize, has_more: bool) -> Self {
        self.total_count = Some(total);
        self.has_more = has_more;
        self
    }

    /// Set queried contexts.
    pub fn with_contexts(mut self, contexts: Vec<ContextId>) -> Self {
        self.queried_contexts = Some(contexts);
        self
    }

    /// Get results as typed values.
    pub fn results_as<T: for<'de> Deserialize<'de>>(&self) -> Vec<T> {
        self.results
            .iter()
            .filter_map(|v| serde_json::from_value(v.clone()).ok())
            .collect()
    }

    /// Check if results are empty.
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    /// Get number of results.
    pub fn len(&self) -> usize {
        self.results.len()
    }
}

/// Information about a supported query type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryTypeInfo {
    /// Query type name.
    pub name: String,

    /// Description.
    pub description: String,

    /// Required parameters.
    pub required_params: Vec<String>,

    /// Optional parameters.
    pub optional_params: Vec<String>,

    /// Example usage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,
}

impl QueryTypeInfo {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            required_params: vec![],
            optional_params: vec![],
            example: None,
        }
    }

    pub fn required(mut self, params: Vec<&str>) -> Self {
        self.required_params = params.into_iter().map(String::from).collect();
        self
    }

    pub fn optional(mut self, params: Vec<&str>) -> Self {
        self.optional_params = params.into_iter().map(String::from).collect();
        self
    }

    pub fn example(mut self, example: impl Serialize) -> Self {
        self.example = serde_json::to_value(example).ok();
        self
    }
}

/// Queryable trait that all sisters should implement.
pub trait Queryable {
    /// Execute a query.
    fn query(&self, query: Query) -> SisterResult<QueryResult>;

    /// Check if a query type is supported.
    fn supports_query(&self, query_type: &str) -> bool;

    /// List supported query types.
    fn query_types(&self) -> Vec<QueryTypeInfo>;

    /// Execute a simple search query.
    fn search(&self, text: &str) -> SisterResult<QueryResult> {
        self.query(Query::search(text))
    }

    /// Get recent items.
    fn recent(&self, count: usize) -> SisterResult<QueryResult> {
        self.query(Query::recent(count))
    }

    /// List items with pagination.
    fn list(&self, limit: usize, offset: usize) -> SisterResult<QueryResult> {
        self.query(Query::list().limit(limit).offset(offset))
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
    fn test_query_builder() {
        let query = Query::search("hello")
            .limit(10)
            .offset(5)
            .param("extra", "value");

        assert_eq!(query.query_type, "search");
        assert_eq!(query.limit, Some(10));
        assert_eq!(query.offset, Some(5));
        assert_eq!(query.get_string("text"), Some("hello".to_string()));
        assert_eq!(query.get_string("extra"), Some("value".to_string()));
    }

    #[test]
    fn test_common_queries() {
        let list = Query::list();
        assert_eq!(list.query_type, "list");

        let recent = Query::recent(5);
        assert_eq!(recent.query_type, "recent");
        assert_eq!(recent.limit, Some(5));

        let search = Query::search("test");
        assert_eq!(search.get_string("text"), Some("test".to_string()));
    }

    #[test]
    fn test_query_result() {
        let query = Query::list();
        let results = vec![
            serde_json::json!({"id": "1"}),
            serde_json::json!({"id": "2"}),
        ];

        let result =
            QueryResult::new(query, results, Duration::from_millis(10)).with_pagination(100, true);

        assert_eq!(result.len(), 2);
        assert!(result.has_more);
        assert_eq!(result.total_count, Some(100));
    }
}
