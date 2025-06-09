use crate::types::{CollectionConfig, VectorDocument};
use serde_json;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

pub struct PersistentStorage {
    file_path: PathBuf,
    config: CollectionConfig,
    buffer: Arc<RwLock<Vec<VectorDocument>>>,
    buffer_size: usize,
}

impl PersistentStorage {
    pub fn new(config: CollectionConfig, data_dir: &Path) -> Result<Self, Box<dyn Error>> {
        let file_path = data_dir.join(format!("{}.jsonl", config.name));
        
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        Ok(PersistentStorage {
            file_path,
            config,
            buffer: Arc::new(RwLock::new(Vec::new())),
            buffer_size: 1000,
        })
    }

    pub fn store(&self, document: VectorDocument) -> Result<(), Box<dyn Error>> {
        let mut buffer = self.buffer.write().map_err(|_| "Failed to acquire write lock")?;
        buffer.push(document);

        if buffer.len() >= self.buffer_size {
            self.flush_buffer(&mut buffer)?;
        }

        Ok(())
    }

    pub fn flush(&self) -> Result<(), Box<dyn Error>> {
        let mut buffer = self.buffer.write().map_err(|_| "Failed to acquire write lock")?;
        self.flush_buffer(&mut buffer)
    }

    fn flush_buffer(&self, buffer: &mut Vec<VectorDocument>) -> Result<(), Box<dyn Error>> {
        if buffer.is_empty() {
            return Ok(());
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)?;

        let mut writer = BufWriter::new(file);

        for document in buffer.drain(..) {
            let json = serde_json::to_string(&document)?;
            writeln!(writer, "{}", json)?;
        }

        writer.flush()?;
        Ok(())
    }

    pub fn load_all(&self) -> Result<Vec<VectorDocument>, Box<dyn Error>> {
        if !self.file_path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut documents = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if !line.trim().is_empty() {
                match serde_json::from_str::<VectorDocument>(&line) {
                    Ok(document) => documents.push(document),
                    Err(e) => {
                        log::warn!("Failed to parse line in storage file: {}", e);
                        continue;
                    }
                }
            }
        }

        Ok(documents)
    }

    pub fn clear(&self) -> Result<(), Box<dyn Error>> {
        if self.file_path.exists() {
            std::fs::remove_file(&self.file_path)?;
        }

        let mut buffer = self.buffer.write().map_err(|_| "Failed to acquire write lock")?;
        buffer.clear();

        Ok(())
    }

    pub fn backup(&self, backup_path: &Path) -> Result<(), Box<dyn Error>> {
        self.flush()?;
        
        if self.file_path.exists() {
            std::fs::copy(&self.file_path, backup_path)?;
        }

        Ok(())
    }

    pub fn restore(&self, backup_path: &Path) -> Result<(), Box<dyn Error>> {
        if backup_path.exists() {
            std::fs::copy(backup_path, &self.file_path)?;
        }

        Ok(())
    }

    pub fn compact(&self) -> Result<usize, Box<dyn Error>> {
        let documents = self.load_all()?;
        
        self.clear()?;
        
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.file_path)?;

        let mut writer = BufWriter::new(file);
        let mut written = 0;

        for document in documents {
            let json = serde_json::to_string(&document)?;
            writeln!(writer, "{}", json)?;
            written += 1;
        }

        writer.flush()?;
        Ok(written