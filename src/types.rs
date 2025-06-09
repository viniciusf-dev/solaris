use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Vector = Vec<f32>;
pub type VectorMetadata = Vec<(String, String)>;
pub type SearchResult = (String, f32, Option<VectorMetadata>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionConfig {
    pub name: String,
    pub dimension: usize,
    pub metric: DistanceMetric,
    pub max_elements: Option<usize>,
    pub ef_construction: usize,
    pub m: usize,
}

impl Default for CollectionConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            dimension: 0,
            metric: DistanceMetric::Cosine,
            max_elements: None,
            ef_construction: 200,
            m: 16,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DistanceMetric {
    Cosine,
    Euclidean,
    Manhattan,
    DotProduct,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorDocument {
    pub id: String,
    pub vector: Vector,
    pub metadata: Option<VectorMetadata>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub vector: Vector,
    pub limit: usize,
    pub ef: Option<usize>,
    pub filter: Option<MetadataFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataFilter {
    pub conditions: Vec<FilterCondition>,
    pub operator: FilterOperator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterCondition {
    pub key: String,
    pub value: String,
    pub operation: FilterOperation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOperator {
    And,
    Or,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOperation {
    Equals,
    NotEquals,
    Contains,
    StartsWith,
    EndsWith,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseInfo {
    pub name: String,
    pub version: String,
    pub collections: Vec<CollectionInfo>,
    pub total_vectors: usize,
    pub memory_usage: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionInfo {
    pub name: String,
    pub dimension: usize,
    pub metric: DistanceMetric,
    pub vector_count: usize,
    pub size_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchInsertRequest {
    pub vectors: Vec<VectorDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchInsertResponse {
    pub inserted: usize,
    pub failed: Vec<(String, String)>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct IndexStats {
    pub total_vectors: usize,
    pub index_size: usize,
    pub avg_search_time_ms: f64,
    pub memory_usage_mb: f64,
}