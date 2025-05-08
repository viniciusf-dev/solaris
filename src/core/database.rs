use crate::index::vector_index::VectorIndex;
use crate::storage::memory_storage::MemoryStorage;
use crate::types::{CollectionConfig, SearchResult, Vector, VectorMetadata};
use std::collections::HashMap;
use std::error::Error;

pub struct Database {
    name: String,
    collections: HashMap<String, Collection>,
}

impl Database {
    pub fn new(name: String) -> Self {
        Database {
            name,
            collections: HashMap::new(),
        }
    }
    
    pub fn create_collection(&mut self, name: &str, dimension: usize) -> Result<(), Box<dyn Error>> {
        if self.collections.contains_key(name) {
            return Err(format!("Collection '{}' already exists", name).into());
        }
        
        let config = CollectionConfig {
            name: name.to_string(),
            dimension,
        };
        
        let collection = Collection::new(config);
        self.collections.insert(name.to_string(), collection);
        
        Ok(())
    }
    
    pub fn insert_vector(
        &mut self,
        collection_name: &str,
        id: String,
        vector: Vector,
        metadata: Option<VectorMetadata>,
    ) -> Result<(), Box<dyn Error>> {
        let collection = self.get_collection_mut(collection_name)?;
        collection.insert_vector(id, vector, metadata)
    }
    
    pub fn search_vectors(
        &self,
        collection_name: &str,
        query_vector: Vector,
        limit: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn Error>> {
        let collection = self.get_collection(collection_name)?;
        collection.search_vectors(query_vector, limit)
    }
    
    fn get_collection(&self, name: &str) -> Result<&Collection, Box<dyn Error>> {
        self.collections
            .get(name)
            .ok_or_else(|| format!("Collection '{}' not found", name).into())
    }
    
    fn get_collection_mut(&mut self, name: &str) -> Result<&mut Collection, Box<dyn Error>> {
        self.collections
            .get_mut(name)
            .ok_or_else(|| format!("Collection '{}' not found", name).into())
    }
}

pub struct Collection {
    config: CollectionConfig,
    storage: MemoryStorage,
    index: VectorIndex,
}

impl Collection {
    pub fn new(config: CollectionConfig) -> Self {
        Collection {
            config: config.clone(),
            storage: MemoryStorage::new(config.clone()),
            index: VectorIndex::new(config),
        }
    }
    
    pub fn insert_vector(
        &mut self,
        id: String,
        vector: Vector,
        metadata: Option<VectorMetadata>,
    ) -> Result<(), Box<dyn Error>> {
        
        if vector.len() != self.config.dimension {
            return Err(format!(
                "Vector dimension mismatch. Expected {}, got {}",
                self.config.dimension,
                vector.len()
            )
            .into());
        }
        
        self.storage.store(id.clone(), vector.clone(), metadata.clone())?;
        
       
        self.index.add_vector(id, vector)?;
        
        Ok(())
    }
    
    pub fn search_vectors(
        &self,
        query_vector: Vector,
        limit: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn Error>> {

        if query_vector.len() != self.config.dimension {
            return Err(format!(
                "Query vector dimension mismatch. Expected {}, got {}",
                self.config.dimension,
                query_vector.len()
            )
            .into());
        }
        
        let nearest_ids = self.index.search(query_vector, limit)?;
        
        let mut results = Vec::with_capacity(nearest_ids.len());
        for (id, score) in nearest_ids {
            let metadata = self.storage.get_metadata(&id)?;
            results.push((id, score, metadata));
        }
        
        Ok(results)
    }
}