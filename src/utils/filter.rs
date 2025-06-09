use crate::types::{FilterCondition, FilterOperation, FilterOperator, MetadataFilter, VectorDocument, VectorMetadata};
use rayon::prelude::*;

pub fn apply_filter(documents: &[VectorDocument], filter: &MetadataFilter) -> Vec<&VectorDocument> {
    documents
        .par_iter()
        .filter(|doc| evaluate_filter(doc, filter))
        .collect()
}

pub fn evaluate_filter(document: &VectorDocument, filter: &MetadataFilter) -> bool {
    if filter.conditions.is_empty() {
        return true;
    }

    let results: Vec<bool> = filter.conditions
        .iter()
        .map(|condition| evaluate_condition(document, condition))
        .collect();

    match filter.operator {
        FilterOperator::And => results.iter().all(|&x| x),
        FilterOperator::Or => results.iter().any(|&x| x),
    }
}

fn evaluate_condition(document: &VectorDocument, condition: &FilterCondition) -> bool {
    if let Some(metadata) = &document.metadata {
        if let Some(value) = get_metadata_value(metadata, &condition.key) {
            match condition.operation {
                FilterOperation::Equals => value == condition.value,
                FilterOperation::NotEquals => value != condition.value,
                FilterOperation::Contains => value.contains(&condition.value),
                FilterOperation::StartsWith => value.starts_with(&condition.value),
                FilterOperation::EndsWith => value.ends_with(&condition.value),
            }
        } else {
            false
        }
    } else {
        false
    }
}

fn get_metadata_value(metadata: &VectorMetadata, key: &str) -> Option<String> {
    metadata.iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.clone())
}

pub fn filter_by_metadata_key(documents: &[VectorDocument], key: &str) -> Vec<&VectorDocument> {
    documents
        .par_iter()
        .filter(|doc| {
            if let Some(metadata) = &doc.metadata {
                metadata.iter().any(|(k, _)| k == key)
            } else {
                false
            }
        })
        .collect()
}

pub fn filter_by_timestamp_range(
    documents: &[VectorDocument],
    start: u64,
    end: u64,
) -> Vec<&VectorDocument> {
    documents
        .par_iter()
        .filter(|doc| doc.timestamp >= start && doc.timestamp <= end)
        .collect()
}

pub fn create_simple_filter(key: String, value: String, operation: FilterOperation) -> MetadataFilter {
    MetadataFilter {
        conditions: vec![FilterCondition {
            key,
            value,
            operation,
        }],
        operator: FilterOperator::And,
    }
}