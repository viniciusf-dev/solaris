use crate::types::{CollectionConfig, Vector, VectorDocument};
use std::error::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Vector dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
    
    #[error("Vector ID cannot be empty")]
    EmptyId,
    
    #[error("Vector contains invalid values (NaN or Infinity)")]
    InvalidValues,
    
    #[error("Collection name cannot be empty")]
    EmptyCollectionName,
    
    #[error("Invalid collection dimension: {0}")]
    InvalidDimension(usize),
    
    #[error("Invalid HNSW parameters: M={m}, ef_construction={ef_construction}")]
    InvalidHNSWParams { m: usize, ef_construction: usize },
    
    #[error("Metadata key cannot be empty")]
    EmptyMetadataKey,
    
    #[error("Vector ID too long: maximum 256 characters")]
    IdTooLong,
    
    #[error("Too many metadata entries: maximum 100")]
    TooManyMetadataEntries,
}

pub fn validate_vector(vector: &Vector, expected_dimension: usize) -> Result<(), ValidationError> {
    if vector.len() != expected_dimension {
        return Err(ValidationError::DimensionMismatch {
            expected: expected_dimension,
            actual: vector.len(),
        });
    }

    for &value in vector {
        if !value.is_finite() {
            return Err(ValidationError::InvalidValues);
        }
    }

    Ok(())
}

pub fn validate_vector_id(id: &str) -> Result<(), ValidationError> {
    if id.is_empty() {
        return Err(ValidationError::EmptyId);
    }

    if id.len() > 256 {
        return Err(ValidationError::IdTooLong);
    }

    Ok(())
}

pub fn validate_collection_config(config: &CollectionConfig) -> Result<(), ValidationError> {
    if config.name.is_empty() {
        return Err(ValidationError::EmptyCollectionName);
    }

    if config.dimension == 0 || config.dimension > 10000 {
        return Err(ValidationError::InvalidDimension(config.dimension));
    }

    if config.m == 0 || config.m > 100 || config.ef_construction < config.m {
        return Err(ValidationError::InvalidHNSWParams {
            m: config.m,
            ef_construction: config.ef_construction,
        });
    }

    Ok(())
}

pub fn validate_vector_document(
    document: &VectorDocument,
    expected_dimension: usize,
) -> Result<(), ValidationError> {
    validate_vector_id(&document.id)?;
    validate_vector(&document.vector, expected_dimension)?;

    if let Some(metadata) = &document.metadata {
        if metadata.len() > 100 {
            return Err(ValidationError::TooManyMetadataEntries);
        }

        for (key, _) in metadata {
            if key.is_empty() {
                return Err(ValidationError::EmptyMetadataKey);
            }
        }
    }

    Ok(())
}

pub fn validate_search_params(
    query_vector: &Vector,
    expected_dimension: usize,
    limit: usize,
    ef: Option<usize>,
) -> Result<(), Box<dyn Error>> {
    validate_vector(query_vector, expected_dimension)?;

    if limit == 0 {
        return Err("Search limit must be greater than 0".into());
    }

    if limit > 10000 {
        return Err("Search limit too large: maximum 10000".into());
    }

    if let Some(ef_value) = ef {
        if ef_value < limit {
            return Err("EF parameter must be greater than or equal to limit".into());
        }
        if ef_value > 10000 {
            return Err("EF parameter too large: maximum 10000".into());
        }
    }

    Ok(())
}

pub fn sanitize_collection_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect::<String>()
        .to_lowercase()
}

pub fn validate_batch_size(size: usize) -> Result<(), Box<dyn Error>> {
    if size == 0 {
        return Err("Batch size must be greater than 0".into());
    }

    if size > 10000 {
        return Err("Batch size too large: maximum 10000".into());
    }

    Ok(())
}