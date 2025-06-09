use rayon::prelude::*;
use std::collections::HashMap;

#[derive(Clone)]
pub struct VectorEntry {
    pub id: String,
    pub vector: Vec<f32>,
    pub metadata: Option<HashMap<String, String>>,
}

pub struct BruteIndex {
    dim: usize,
    entries: Vec<VectorEntry>,
}

impl BruteIndex {
    pub fn new(dim: usize) -> Self {
        Self { dim, entries: Vec::new() }
    }

    pub fn insert(
        &mut self,
        id: String,
        vector: Vec<f32>,
        metadata: Option<HashMap<String, String>>,
    ) -> anyhow::Result<()> {
        if vector.len() != self.dim {
            anyhow::bail!("invalid dimension");
        }
        self.entries.push(VectorEntry { id, vector, metadata });
        Ok(())
    }

    fn cosine(a: &[f32], b: &[f32]) -> f32 {
        let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
        let na = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let nb = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        if na == 0.0 || nb == 0.0 { 0.0 } else { dot / (na * nb) }
    }

    pub fn search(
        &self,
        query: &[f32],
        k: usize,
    ) -> anyhow::Result<Vec<(String, f32, Option<HashMap<String, String>>)>> {
        if query.len() != self.dim {
            anyhow::bail!("invalid dimension");
        }
        let mut scored: Vec<(usize, f32)> = self
            .entries
            .par_iter()
            .enumerate()
            .map(|(i, e)| (i, Self::cosine(&e.vector, query)))
            .collect();
        scored.par_sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        Ok(scored
            .into_iter()
            .take(k)
            .map(|(i, s)| {
                let e = &self.entries[i];
                (e.id.clone(), s, e.metadata.clone())
            })
            .collect())
    }
}
