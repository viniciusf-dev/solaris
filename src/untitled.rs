use crate::config::SolarisConfig;
use crate::index::vector_index::VectorIndex;
use crate::storage::memory_storage::MemoryStorage;
use crate::types::{
    BatchInsertRequest, BatchInsertResponse, CollectionConfig, CollectionInfo, DatabaseInfo,
    DistanceMetric, IndexStats, SearchQuery, SearchResult, Vector, VectorDocument, VectorMetadata,
};
use crate::utils::filter::{apply_filter, evaluate_filter};
use crate::utils::validation::{validate_collection_config, validate_search_params, validate_vector_document};
use rayon::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, RwLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[cfg(feature = "persistence")]
use crate::storage::persistent_storage::PersistentStorage;

pub struct Database {
    name: String,
    collections: Arc<RwLock<HashMap<String, Collection>>>,
    config: SolarisConfig,
}

impl Database {
    pub fn new(name: String) -> Self {
        Database {
            name,
            collections: Arc::new(RwLock::new(HashMap::new())),
            config: SolarisConfig::default(),
        }
    }

    pub fn with_config(name: String, config: SolarisConfig) -> Self {
        Database {
            name,
            collections: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    pub fn create_collection(
        &self,
        name: &str,
        dimension: usize,
        metric: Option<DistanceMetric>,
    ) -> Result<(), Box<dyn Error>> {
        let mut config = CollectionConfig {
            name: name.to_string(),
            dimension,
            metric: metric.unwrap_or(self.config.collections.default_metric),
            max_elements: self.config.collections.max_vectors_per_collection,
            ef_construction: self.config.collections.default_ef_construction,
            m: self.config.collections.default_m,
        };

        validate_collection_config(&config)?;

        let mut collections = self.collections.write().map_err(|_| "Failed to acquire write lock")?;
        
        if collections.contains_key(name) {
            return Err(format!("Collection '{}' already exists", name).into());
        }

        if collections.len() >= self.config.database.max_collections {
            return Err(format!("Maximum number of collections ({}) reached", self.config.database.max_collections).into());
        }

        let collection = Collection::new(config, &self.config)?;
        collections.insert(name.to_string(), collection);

        Ok(())
    }

    pub fn drop_collection(&self, name: &str) -> Result<bool, Box<dyn Error>> {
        let mut collections = self.collections.write().map_err(|_| "Failed to acquire write lock")?;
        Ok(collections.remove(name).is_some())
    }

    pub fn list_collections(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let collections = self.collections.read().map_err(|_| "Failed to acquire read lock")?;
        Ok(collections.keys().cloned().collect())
    }

    pub fn get_collection_info(&self, name: &str) -> Result<CollectionInfo, Box<dyn Error>> {
        let collections = self.collections.read().map_err(|_| "Failed to acquire read lock")?;
        let collection = collections.get(name)
            .ok_or_else(|| format!("Collection '{}' not found", name))?;
        
        collection.get_info()
    }

    pub fn insert_vector(
        &self,
        collection_name: &str,
        id: String,
        vector: Vector,
        metadata: Option<VectorMetadata>,
    ) -> Result<(), Box<dyn Error>> {
        let collections = self.collections.read().map_err(|_| "Failed to acquire read lock")?;
        let collection = collections.get(collection_name)
            .ok_or_else(|| format!("Collection '{}' not found", collection_name))?;
        
        collection.insert_vector(id, vector, metadata)
    }

    pub fn batch_insert(
        &self,
        collection_name: &str,
        request: BatchInsertRequest,
    ) -> Result<BatchInsertResponse, Box<dyn Error>> {
        let start_time = Instant::now();
        let collections = self.collections.read().map_err(|_| "Failed to acquire read lock")?;
        let collection = collections.get(collection_name)
            .ok_or_else(|| format!("Collection '{}' not found", collection_name))?;
        
        let response = collection.batch_insert(request)?;
        let duration = start_time.elapsed();
        
        Ok(BatchInsertResponse {
            inserted: response.inserted,
            failed: response.failed,
            duration_ms: duration.as_millis() as u64,
        })
    }

    pub fn search_vectors(
        &self,
        collection_name: &str,
        query: SearchQuery,
    ) -> Result<Vec<SearchResult>, Box<dyn Error>> {
        validate_search_params(&query.vector, 0, query.limit, query.ef)?;
        
        let collections = self.collections.read().map_err(|_| "Failed to acquire read lock")?;
        let collection = collections.get(collection_name)
            .ok_or_else(|| format!("Collection '{}' not found", collection_name))?;
        
        collection.search_vectors(query)
    }

    pub fn get_vector(
        &self,
        collection_name: &str,
        id: &str,
    ) -> Result<Option<VectorDocument>, Box<dyn Error>> {
        let collections = self.collections.read().map_err(|_| "Failed to acquire read lock")?;
        let collection = collections.get(collection_name)
            .ok_or_else(|| format!("Collection '{}' not found", collection_name))?;
        
        collection.get_vector(id)
    }

    pub fn update_metadata(
        &self,
        collection_name: &str,
        id: &str,
        metadata: Option<VectorMetadata>,
    ) -> Result<bool, Box<dyn Error>> {
        let collections = self.collections.read().map_err(|_| "Failed to acquire read lock")?;
        let collection = collections.get(collection_name)
            .ok_or_else(|| format!("Collection '{}' not found", collection_name))?;
        
        collection.update_metadata(id, metadata)
    }

    pub fn delete_vector(&self, collection_name: &str, id: &str) -> Result<bool, Box<dyn Error>> {
        let collections = self.collections.read().map_err(|_| "Failed to acquire read lock")?;
        let collection = collections.get(collection_name)
            .ok_or_else(|| format!("Collection '{}' not found", collection_name))?;
        
        collection.delete_vector(id)
    }

    pub fn count_vectors(&self, collection_name: &str) -> Result<usize, Box<dyn Error>> {
        let collections = self.collections.read().map_err(|_| "Failed to acquire read lock")?;
        let collection = collections.get(collection_name)
            .ok_or_else(|| format!("Collection '{}' not found", collection_name))?;
        
        collection.count()
    }

    pub fn get_database_info(&self) -> Result<DatabaseInfo, Box<dyn Error>> {
        let collections = self.collections.read().map_err(|_| "Failed to acquire read lock")?;
        
        let mut collection_infos = Vec::new();
        let mut total_vectors = 0;
        let mut memory_usage = 0;

        for collection in collections.values() {
            let info = collection.get_info()?;
            total_vectors += info.vector_count;
            memory_usage += info.size_bytes;
            collection_infos.push(info);
        }

        Ok(DatabaseInfo {
            name: self.name.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            collections: collection_infos,
            total_vectors,
            memory_usage,
        })
    }

    pub fn flush_all(&self) -> Result<(), Box<dyn Error>> {
        let collections = self.collections.read().map_err(|_| "Failed to acquire read lock")?;
        
        for collection in collections.values() {
            collection.flush()?;
        }

        Ok(())
    }

    pub fn clear_collection(&self, collection_name: &str) -> Result<(), Box<dyn Error>> {
        let collections = self.collections.read().map_err(|_| "Failed to acquire read lock")?;
        let collection = collections.get(collection_name)
            .ok_or_else(|| format!("Collection '{}' not found", collection_name))?;
        
        collection.clear()
    }
}

pub struct Collection {
    config: CollectionConfig,
    storage: MemoryStorage,
    index: Arc<RwLock<VectorIndex>>,
    
    #[cfg(feature = "persistence")]
    persistent_storage: Option<PersistentStorage>,
}

impl Collection {
    pub fn new(config: CollectionConfig, db_config: &SolarisConfig) -> Result<Self, Box<dyn Error>> {
        let storage = MemoryStorage::new(config.clone());
        let index = Arc::new(RwLock::new(VectorIndex::new(config.clone())));
        
        #[cfg(feature = "persistence")]
        let persistent_storage = if db_config.database.enable_persistence {
            Some(PersistentStorage::new(config.clone(), &db_config.database.data_directory)?)
        } else {
            None
        };

        Ok(Collection {
            config,
            storage,
            index,
            
            #[cfg(feature = "persistence")]
            persistent_storage,
        })
    }

    pub fn insert_vector(
        &self,
        id: String,
        vector: Vector,
        metadata: Option<VectorMetadata>,
    ) -> Result<(), Box<dyn Error>> {
        let document = VectorDocument {
            id: id.clone(),
            vector: vector.clone(),
            metadata: metadata.clone(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        };

        validate_vector_document(&document, self.config.dimension)?;

        self.storage.store(id.clone(), vector.clone(), metadata)?;

        let mut index = self.index.write().map_err(|_| "Failed to acquire write lock")?;
        index.add_vector(id, vector)?;

        #[cfg(feature = "persistence")]
        if let Some(persistent) = &self.persistent_storage {
            persistent.store(document)?;
        }

        Ok(())
    }

    pub fn batch_insert(&self, request: BatchInsertRequest) -> Result<BatchInsertResponse, Box<dyn Error>> {
        let mut inserted = 0;
        let mut failed = Vec::new();

        let results: Vec<_> = request.vectors
            .into_par_iter()
            .map(|doc| {
                match validate_vector_document(&doc, self.config.dimension) {
                    Ok(_) => {
                        match self.insert_vector(doc.id.clone(), doc.vector, doc.metadata) {
                            Ok(_) => Ok(()),
                            Err(e) => Err((doc.id, e.to_string())),
                        }
                    }
                    Err(e) => Err((doc.id, e.to_string())),
                }
            })
            .collect();

        for result in results {
            match result {
                Ok(_) => inserted += 1,
                Err((id, error)) => failed.push((id, error)),
            }
        }

        Ok(BatchInsertResponse {
            inserted,
            failed,
            duration_ms: 0,
        })
    }

    pub fn search_vectors(&self, query: SearchQuery) -> Result<Vec<SearchResult>, Box<dyn Error>> {
        validate_search_params(&query.vector, self.config.dimension, query.limit, query.ef)?;

        let index = self.index.read().map_err(|_| "Failed to acquire read lock")?;
        let nearest_ids = if let Some(ef) = query.ef {
            index.search_with_ef(query.vector, query.limit, ef)?
        } else {
            index.search(query.vector, query.limit)?
        };

        let mut results = Vec::with_capacity(nearest_ids.len());
        
        for (id, score) in nearest_ids {
            if let Ok(Some(document)) = self.storage.get(&id) {
                if let Some(filter) = &query.filter {
                    if !evaluate_filter(&document, filter) {
                        continue;
                    }
                }
                results.push((id, score, document.metadata));
            }
        }

        Ok(results)
    }

    pub fn get_vector(&self, id: &str) -> Result<Option<VectorDocument>, Box<dyn Error>> {
        self.storage.get(id)
    }

    pub fn update_metadata(&self, id: &str, metadata: Option<VectorMetadata>) -> Result<bool, Box<dyn Error>> {
        self.storage.update_metadata(id, metadata)
    }

    pub fn delete_vector(&self, id: &str) -> Result<bool, Box<dyn Error>> {
        let removed_from_storage = self.storage.remove(id)?;
        
        if removed_from_storage {
            let mut index = self.index.write().map_err(|_| "Failed to acquire write lock")?;
            index.remove_vector(id)?;
        }

        Ok(removed_from_storage)
    }