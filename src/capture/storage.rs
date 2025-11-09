//! File storage for captured malware and downloads

use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::info;

use crate::Result;

/// Storage for captured files and malware
pub struct FileStorage {
    base_path: PathBuf,
}

impl FileStorage {
    /// Create a new file storage
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// Initialize storage (create directories)
    pub async fn init(&self) -> Result<()> {
        fs::create_dir_all(&self.base_path).await?;
        info!("Initialized file storage at {:?}", self.base_path);
        Ok(())
    }

    /// Store a file and return its SHA256 hash
    pub async fn store_file(&self, content: &[u8]) -> Result<String> {
        // Calculate SHA256
        let mut hasher = Sha256::new();
        hasher.update(content);
        let hash = hasher.finalize();
        let hash_hex = hex::encode(hash);

        // Store file with hash as filename
        let file_path = self.base_path.join(&hash_hex);

        // Only write if file doesn't exist (avoid duplicates)
        if !file_path.exists() {
            let mut file = fs::File::create(&file_path).await?;
            file.write_all(content).await?;
            info!("Stored file: {} ({} bytes)", hash_hex, content.len());
        }

        Ok(hash_hex)
    }

    /// Get the path for a stored file by hash
    pub fn get_path(&self, hash: &str) -> PathBuf {
        self.base_path.join(hash)
    }

    /// Check if a file exists
    pub async fn exists(&self, hash: &str) -> bool {
        self.get_path(hash).exists()
    }
}

impl Default for FileStorage {
    fn default() -> Self {
        Self::new(PathBuf::from("./captured_files"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_file_storage() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path().to_path_buf());

        storage.init().await.unwrap();

        let content = b"test malware content";
        let hash = storage.store_file(content).await.unwrap();

        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA256 is 64 hex chars
        assert!(storage.exists(&hash).await);
    }

    #[tokio::test]
    async fn test_duplicate_storage() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path().to_path_buf());

        storage.init().await.unwrap();

        let content = b"duplicate content";
        let hash1 = storage.store_file(content).await.unwrap();
        let hash2 = storage.store_file(content).await.unwrap();

        // Same content should produce same hash
        assert_eq!(hash1, hash2);
    }
}
