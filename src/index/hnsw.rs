use crate::types::{CollectionConfig, DistanceMetric, Vector};
use crate::utils::distance::calculate_distance;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;
use std::error::Error;
use rayon::prelude::*;
use rand::Rng;

#[derive(Clone)]
struct Node {
    id: String,
    vector: Vector,
    connections: Vec<Vec<String>>,
    level: usize,
}

#[derive(PartialEq)]
struct SearchCandidate {
    id: String,
    distance: f32,
}

impl Eq for SearchCandidate {}

impl PartialOrd for SearchCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.distance.partial_cmp(&other.distance)
    }
}

impl Ord for SearchCandidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

pub struct HNSWIndex {
    nodes: HashMap<String, Node>,
    entry_point: Option<String>,
    max_level: usize,
    level_multiplier: f64,
    config: CollectionConfig,
    rng: rand::rngs::ThreadRng,
}

impl HNSWIndex {
    pub fn new(config: CollectionConfig) -> Self {
        HNSWIndex {
            nodes: HashMap::new(),
            entry_point: None,
            max_level: 0,
            level_multiplier: 1.0 / (2.0_f64).ln(),
            config,
            rng: rand::thread_rng(),
        }
    }

    pub fn add_vector(&mut self, id: String, vector: Vector) -> Result<(), Box<dyn Error>> {
        let level = self.get_random_level();
        
        let mut connections = vec![Vec::new(); level + 1];
        
        let node = Node {
            id: id.clone(),
            vector: vector.clone(),
            connections,
            level,
        };

        if self.entry_point.is_none() {
            self.entry_point = Some(id.clone());
            self.max_level = level;
            self.nodes.insert(id, node);
            return Ok(());
        }

        let mut current_closest = vec![self.entry_point.as_ref().unwrap().clone()];
        
        for lc in (level + 1..=self.max_level).rev() {
            current_closest = self.search_layer(&vector, &current_closest, 1, lc)?;
        }

        for lc in (0..=level.min(self.max_level)).rev() {
            let candidates = self.search_layer(&vector, &current_closest, self.config.ef_construction, lc)?;
            
            let selected = self.select_neighbors_heuristic(&vector, &candidates, self.config.m)?;
            
            for neighbor_id in &selected {
                if let Some(neighbor) = self.nodes.get_mut(neighbor_id) {
                    if neighbor.level >= lc {
                        neighbor.connections[lc].push(id.clone());
                    }
                }
            }
            
            if let Some(node) = self.nodes.get_mut(&id) {
                node.connections[lc] = selected.clone();
            }
            
            current_closest = selected;
        }

        if level > self.max_level {
            self.max_level = level;
            self.entry_point = Some(id.clone());
        }

        self.nodes.insert(id, node);
        Ok(())
    }

    pub fn search(&self, query: Vector, k: usize, ef: Option<usize>) -> Result<Vec<(String, f32)>, Box<dyn Error>> {
        if self.entry_point.is_none() {
            return Ok(Vec::new());
        }

        let ef = ef.unwrap_or(k.max(50));
        let mut current_closest = vec![self.entry_point.as_ref().unwrap().clone()];

        for lc in (1..=self.max_level).rev() {
            current_closest = self.search_layer(&query, &current_closest, 1, lc)?;
        }

        let candidates = self.search_layer(&query, &current_closest, ef, 0)?;
        
        let mut result: Vec<_> = candidates.into_par_iter()
            .filter_map(|id| {
                self.nodes.get(&id).map(|node| {
                    let distance = calculate_distance(&query, &node.vector, self.config.metric);
                    (id, distance)
                })
            })
            .collect();
        
        result.par_sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
        result.truncate(k);
        
        Ok(result)
    }

    fn search_layer(
        &self,
        query: &Vector,
        entry_points: &[String],
        num_closest: usize,
        level: usize,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let mut visited = HashSet::new();
        let mut candidates = BinaryHeap::new();
        let mut w = BinaryHeap::new();

        for ep in entry_points {
            if let Some(node) = self.nodes.get(ep) {
                let distance = calculate_distance(query, &node.vector, self.config.metric);
                candidates.push(SearchCandidate {
                    id: ep.clone(),
                    distance: -distance,
                });
                w.push(SearchCandidate {
                    id: ep.clone(),
                    distance,
                });
                visited.insert(ep.clone());
            }
        }

        while let Some(current) = candidates.pop() {
            let current_id = current.id;
            let current_distance = -current.distance;

            if let Some(furthest) = w.peek() {
                if current_distance > furthest.distance {
                    break;
                }
            }

            if let Some(current_node) = self.nodes.get(&current_id) {
                if level < current_node.connections.len() {
                    for neighbor_id in &current_node.connections[level] {
                        if !visited.contains(neighbor_id) {
                            visited.insert(neighbor_id.clone());
                            
                            if let Some(neighbor_node) = self.nodes.get(neighbor_id) {
                                let distance = calculate_distance(query, &neighbor_node.vector, self.config.metric);
                                
                                if w.len() < num_closest {
                                    candidates.push(SearchCandidate {
                                        id: neighbor_id.clone(),
                                        distance: -distance,
                                    });
                                    w.push(SearchCandidate {
                                        id: neighbor_id.clone(),
                                        distance,
                                    });
                                } else if let Some(furthest) = w.peek() {
                                    if distance < furthest.distance {
                                        candidates.push(SearchCandidate {
                                            id: neighbor_id.clone(),
                                            distance: -distance,
                                        });
                                        w.pop();
                                        w.push(SearchCandidate {
                                            id: neighbor_id.clone(),
                                            distance,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(w.into_iter().map(|c| c.id).collect())
    }

    fn select_neighbors_heuristic(
        &self,
        vector: &Vector,
        candidates: &[String],
        m: usize,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        if candidates.len() <= m {
            return Ok(candidates.to_vec());
        }

        let mut selected = Vec::new();
        let mut remaining: Vec<_> = candidates.iter().collect();

        while selected.len() < m && !remaining.is_empty() {
            let mut best_idx = 0;
            let mut best_score = f32::INFINITY;

            for (idx, candidate_id) in remaining.iter().enumerate() {
                if let Some(candidate_node) = self.nodes.get(*candidate_id) {
                    let distance_to_query = calculate_distance(vector, &candidate_node.vector, self.config.metric);
                    
                    let mut min_distance_to_selected = f32::INFINITY;
                    for selected_id in &selected {
                        if let Some(selected_node) = self.nodes.get(selected_id) {
                            let distance = calculate_distance(&candidate_node.vector, &selected_node.vector, self.config.metric);
                            min_distance_to_selected = min_distance_to_selected.min(distance);
                        }
                    }

                    let score = if selected.is_empty() {
                        distance_to_query
                    } else {
                        distance_to_query - min_distance_to_selected
                    };

                    if score < best_score {
                        best_score = score;
                        best_idx = idx;
                    }
                }
            }

            selected.push(remaining.remove(best_idx).clone());
        }

        Ok(selected)
    }

    fn get_random_level(&mut self) -> usize {
        let mut level = 0;
        while self.rng.gen::<f64>() < 0.5 && level < 16 {
            level += 1;
        }
        level
    }

    pub fn get_stats(&self) -> (usize, usize) {
        let total_connections: usize = self.nodes.values()
            .map(|node| node.connections.iter().map(|level| level.len()).sum::<usize>())
            .sum();
        
        (self.nodes.len(), total_connections)
    }

    pub fn remove_vector(&mut self, id: &str) -> Result<bool, Box<dyn Error>> {
        if let Some(node) = self.nodes.remove(id) {
            for level in 0..=node.level {
                for neighbor_id in &node.connections[level] {
                    if let Some(neighbor) = self.nodes.get_mut(neighbor_id) {
                        if level < neighbor.connections.len() {
                            neighbor.connections[level].retain(|x| x != id);
                        }
                    }
                }
            }

            if self.entry_point.as_ref() == Some(&id.to_string()) {
                self.entry_point = self.nodes.keys().next().cloned();
                if let Some(new_entry) = &self.entry_point {
                    if let Some(new_entry_node) = self.nodes.get(new_entry) {
                        self.max_level = new_entry_node.level;
                    }
                }
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }
}