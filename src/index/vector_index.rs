use crate::index::hnsw::HNSWIndex;
use crate::types::{CollectionConfig, Vector};
use std::error::Error;

pub struct VectorIndex {
    hnsw: HNSWIndex,
}

impl VectorIndex {
    pub fn new(config: CollectionConfig) -> Self {
        VectorIndex {
            hnsw: HNSWIndex::new(config),
        }
    }

    pub fn add_vector(&mut self, id: String, vector: Vector) -> Result<(), Box<dyn Error>> {
        self.hnsw.add_vector(id, vector)
    }

    pub fn search(&self, query: Vector, limit: usize) -> Result<Vec<(String, f32)>, Box<dyn Error>> {
        self.hnsw.search(query, limit, None)
    }

    pub fn search_with_ef(&self, query: Vector, limit: usize, ef: usize) -> Result<Vec<(String, f32)>, Box<dyn Error>> {
        self.hnsw.search(query, limit, Some(ef))
    }

    pub fn remove_vector(&mut self, id: &str) -> Result<bool, Box<dyn Error>> {
        self.hnsw.remove_vector(id)
    }

    pub fn get_stats(&self) -> (usize, usize) {
        self.hnsw.get_stats()
    }
}