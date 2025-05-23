# Solaris Vector Database

Solaris is a fast, lightweight vector database built from scratch in Rust. It is designed for high-performance similarity search operations across vector embeddings, making it ideal for machine learning applications, recommendation systems, and semantic search.

## Features

- **Blazing Fast**: Built with Rust for exceptional performance and memory efficiency
- **Simple API**: Easy-to-use interface for vector storage and retrieval
- **Immutable Design**: Leverages Rust's immutability guarantees for robustness
- **Modular Architecture**: Clean separation of concerns for maintainability
- **Similarity Search**: Efficient cosine and euclidean distance calculations
- **Metadata Storage**: Associate custom metadata with your vectors

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/solaris.git
cd solaris

# Build the project
cargo build --release
```

### Usage Example

```rust
use solaris::core::database::Database;

fn main() {
    // Create a new database instance
    let mut db = Database::new("my_database".to_string());
    
    // Create a collection for storing embeddings (dimension=384)
    db.create_collection("articles", 384).unwrap();
    
    // Insert vectors with metadata
    db.insert_vector(
        "articles",
        "doc1".to_string(),
        vec![0.1, 0.2, 0.3, ...],  // 384-dimensional vector
        Some(vec![("category".to_string(), "technology".to_string())]),
    ).unwrap();
    
    // Search for similar vectors
    let query_vector = vec![0.2, 0.3, 0.4, ...];  // Your query vector
    let results = db.search_vectors("articles", query_vector, 5).unwrap();
    
    // Process results
    for (id, score, metadata) in results {
        println!("ID: {}, Score: {}, Metadata: {:?}", id, score, metadata);
    }
}
```

## Architecture

Solaris follows a modular design with several key components:

- **Database**: The main entry point that manages collections
- **Collection**: Stores vectors of the same dimensionality
- **VectorIndex**: Implements efficient similarity search algorithms
- **Storage**: Handles the persistence of vectors and metadata
- **Distance Utils**: Provides distance/similarity calculations

## Performance

Solaris is optimized for:

- Fast vector insertion
- Efficient similarity search
- Low memory footprint
- Parallel processing of large vector sets

## Roadmap

- [ ] Persistent storage backend
- [ ] Advanced indexing methods (HNSW, ANNOY)
- [ ] Multi-threaded search
- [ ] Filtering by metadata
- [ ] REST API interface
- [ ] Command-line tools
- [ ] Clustering and dimensionality reduction

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.