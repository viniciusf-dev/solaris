use crate::types::{CollectionConfig, Vector, VectorDocument, VectorMetadata};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct MemoryStorage {
    data: Arc<RwLock<HashMap<String, VectorDocument>>>,
    config: CollectionConfig,
}

impl MemoryStorage {
    pub fn new(config: CollectionConfig) -> Self {
        MemoryStorage {
            data: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    pub fn store(
        &self,
        id: String,
        vector: Vector,
        metadata: Option<VectorMetadata>,
    ) -> Result<(), Box<dyn Error>> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        let document = VectorDocument {
            id: id.clone(),
            vector,
            metadata,
            timestamp,
        };

        let mut data = self.data.write().map_err(|_| "Failed to acquire write lock")?;
        data.insert(id, document);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<Option<VectorDocument>, Box<dyn Error>> {
        let data = self.data.read().map_err(|_| "Failed to acquire read lock")?;
        Ok(data.get(id).cloned())
    }

    pub fn get_vector(&self, id: &str) -> Result<Option<Vector>, Box<dyn Error>> {
        let data = self.data.read().map_err(|_| "Failed to acquire read lock")?;
        Ok(data.get(id).map(|doc| doc.vector.clone()))
    }

    pub fn get_metadata(&self, id: &str) -> Result<Option<VectorMetadata>, Box<dyn Error>> {
        let data = self.data.read().map_err(|_| "Failed to acquire read lock")?;
        Ok(data.get(id).and_then(|doc| doc.metadata.clone()))
    }

    pub fn remove(&self, id: &str) -> Result<bool, Box<dyn Error>> {
        let mut data = self.data.write().map_err(|_| "Failed to acquire write lock")?;
        Ok(data.remove(id).is_some())
    }

    pub fn list_ids(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let data = self.data.read().map_err(|_| "Failed to acquire read lock")?;
        Ok(data.keys().cloned().collect())
    }

    pub fn count(&self) -> Result<usize, Box<dyn Error>> {
        let data = self.data.read().map_err(|_| "Failed to acquire read lock")?;
        Ok(data.len())
    }

    pub fn get_all_documents(&self) -> Result<Vec<VectorDocument>, Box<dyn Error>> {
        let data = self.data.read().map_err(|_| "Failed to acquire read lock")?;
        Ok(data.values().cloned().collect())
    }

    pub fn update_metadata(
        &self,
        id: &str,
        metadata: Option<VectorMetadata>,
    ) -> Result<bool, Box<dyn Error>> {
        let mut data = self.data.write().map_err(|_| "Failed to acquire write lock")?;
        if let Some(document) = data.get_mut(id) {
            document.metadata = metadata;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn batch_insert(
        &self,
        documents: Vec<VectorDocument>,
    ) -> Result<usize, Box<dyn Error>> {
        let mut data = self.data.write().map_err(|_| "Failed to acquire write lock")?;
        let mut inserted = 0;

        for document in documents {
            data.insert(document.id.clone(), document);
            inserted += 1;
        }

        Ok(inserted)
    }

    pub fn clear(&self) -> Result<(), Box<dyn Error>> {
        let mut data = self.data.write().map_err(|_| "Failed to acquire write lock")?;
        data.clear();
        Ok(())
    }

    pub fn size_bytes(&self) -> Result<usize, Box<dyn Error>> {
        let data = self.data.read().map_err(|_| "Failed to acquire read lock")?;
        let mut size = 0;
        
        for document in data.values() {
            size += document.id.len();
            size += document.vector.len() * std::mem::size_of::<f32>();
            if let Some(metadata) = &document.metadata {
                for (key, value) in metadata {
                    size += key.len() + value.len();
                }
            }
            size += std::mem::size_of::<u64>();
        }
        
        Ok(size)
    }
}