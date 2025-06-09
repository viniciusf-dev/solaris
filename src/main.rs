use std::error::Error;

mod config;
mod core;
mod index;
mod storage;
mod types;
mod utils;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting Solaris Vector Database...");
    
    // Criar uma instância do database
    let mut db = core::database::Database::new("solaris_test".to_string());
    
    let collection_name = "test_collection";
    db.create_collection(collection_name, 128)?;
    println!("Created collection '{}'", collection_name);
    
    let test_vectors = vec![
        (
            "doc1".to_string(),
            vec![0.1; 128], 
            Some(vec![("type".to_string(), "article".to_string())]),
        ),
        (
            "doc2".to_string(),
            vec![0.2; 128], 
            Some(vec![("type".to_string(), "video".to_string())]),
        ),
        (
            "doc3".to_string(),
            vec![0.3; 128], 
            Some(vec![("type".to_string(), "article".to_string())]),
        ),
    ];
    
    for (id, vector, metadata) in test_vectors {
        db.insert_vector(collection_name, id, vector, metadata)?;
    }
    println!("Inserted test vectors into collection");
    
    let query_vector = vec![0.15; 128]; 
    let results = db.search_vectors(collection_name, query_vector, 2)?;
    
    println!("Search results:");
    for (id, score, metadata) in results {
        println!("ID: {}, Score: {:.6}, Metadata: {:?}", id, score, metadata);
    }
    
    Ok(())
}