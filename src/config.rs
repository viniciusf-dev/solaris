use crate::types::DistanceMetric;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub name: String,
    pub data_directory: PathBuf,
    pub max_collections: usize,
    pub enable_persistence: bool,
    pub auto_flush_interval_seconds: u64,
    pub memory_limit_mb: Option<usize>,
    pub thread_pool_size: Option<usize>,
    pub compression_enabled: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            name: "solaris".to_string(),
            data_directory: PathBuf::from("./data"),
            max_collections: 100,
            enable_persistence: false,
            auto_flush_interval_seconds: 60,
            memory_limit_mb: None,
            thread_pool_size: None,
            compression_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionSettings {
    pub default_dimension: usize,
    pub default_metric: DistanceMetric,
    pub default_m: usize,
    pub default_ef_construction: usize,
    pub max_vectors_per_collection: Option<usize>,
    pub enable_metadata_indexing: bool,
}

impl Default for CollectionSettings {
    fn default() -> Self {
        Self {
            default_dimension: 384,
            default_metric: DistanceMetric::Cosine,
            default_m: 16,
            default_ef_construction: 200,
            max_vectors_per_collection: None,
            enable_metadata_indexing: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub search_timeout_ms: u64,
    pub batch_size: usize,
    pub parallel_search_threshold: usize,
    pub cache_size: usize,
    pub prefetch_enabled: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            search_timeout_ms: 5000,
            batch_size: 1000,
            parallel_search_threshold: 1000,
            cache_size: 10000,
            prefetch_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolarisConfig {
    pub database: DatabaseConfig,
    pub collections: CollectionSettings,
    pub performance: PerformanceConfig,
}

impl Default for SolarisConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            collections: CollectionSettings::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

impl SolarisConfig {
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: SolarisConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn from_env() -> Self {
        let mut config = SolarisConfig::default();

        if let Ok(name) = std::env::var("SOLARIS_DB_NAME") {
            config.database.name = name;
        }

        if let Ok(data_dir) = std::env::var("SOLARIS_DATA_DIR") {
            config.database.data_directory = PathBuf::from(data_dir);
        }

        if let Ok(max_collections) = std::env::var("SOLARIS_MAX_COLLECTIONS") {
            if let Ok(max) = max_collections.parse() {
                config.database.max_collections = max;
            }
        }

        if let Ok(enable_persistence) = std::env::var("SOLARIS_ENABLE_PERSISTENCE") {
            config.database.enable_persistence = enable_persistence.to_lowercase() == "true";
        }

        if let Ok(memory_limit) = std::env::var("SOLARIS_MEMORY_LIMIT_MB") {
            if let Ok(limit) = memory_limit.parse() {
                config.database.memory_limit_mb = Some(limit);
            }
        }

        if let Ok(threads) = std::env::var("SOLARIS_THREAD_POOL_SIZE") {
            if let Ok(size) = threads.parse() {
                config.database.thread_pool_size = Some(size);
            }
        }

        config
    }
}