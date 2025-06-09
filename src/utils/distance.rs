use crate::types::{DistanceMetric, Vector};
use rayon::prelude::*;

pub fn calculate_distance(a: &Vector, b: &Vector, metric: DistanceMetric) -> f32 {
    match metric {
        DistanceMetric::Cosine => cosine_distance(a, b),
        DistanceMetric::Euclidean => euclidean_distance(a, b),
        DistanceMetric::Manhattan => manhattan_distance(a, b),
        DistanceMetric::DotProduct => dot_product_distance(a, b),
    }
}

pub fn cosine_distance(a: &Vector, b: &Vector) -> f32 {
    let dot_product = dot_product(a, b);
    let norm_a = norm(a);
    let norm_b = norm(b);
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return 1.0;
    }
    
    1.0 - (dot_product / (norm_a * norm_b))
}

pub fn euclidean_distance(a: &Vector, b: &Vector) -> f32 {
    a.par_iter()
        .zip(b.par_iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

pub fn manhattan_distance(a: &Vector, b: &Vector) -> f32 {
    a.par_iter()
        .zip(b.par_iter())
        .map(|(x, y)| (x - y).abs())
        .sum()
}

pub fn dot_product_distance(a: &Vector, b: &Vector) -> f32 {
    1.0 - dot_product(a, b)
}

pub fn dot_product(a: &Vector, b: &Vector) -> f32 {
    a.par_iter()
        .zip(b.par_iter())
        .map(|(x, y)| x * y)
        .sum()
}

pub fn norm(vector: &Vector) -> f32 {
    vector.par_iter()
        .map(|x| x * x)
        .sum::<f32>()
        .sqrt()
}

pub fn normalize_vector(vector: &mut Vector) {
    let norm = norm(vector);
    if norm > 0.0 {
        vector.par_iter_mut().for_each(|x| *x /= norm);
    }
}

pub fn batch_distance_calculation(
    query: &Vector, 
    vectors: &[Vector], 
    metric: DistanceMetric
) -> Vec<f32> {
    vectors
        .par_iter()
        .map(|v| calculate_distance(query, v, metric))
        .collect()
}