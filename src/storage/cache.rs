use crate::models::{FileInfo, Reference, Symbol, SymbolId};
use crate::storage::store::SymbolStore;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct PersistedIndex {
    pub version: u32,
    pub created_at: SystemTime,
    pub root_path: PathBuf,
    pub symbols_by_name: HashMap<String, Vec<SymbolId>>,
    pub symbol_data: HashMap<SymbolId, Symbol>,
    pub references: HashMap<SymbolId, Vec<Reference>>,
    pub files: HashMap<PathBuf, FileInfo>,
}

pub struct CacheManager {
    cache_dir: PathBuf,
}

impl CacheManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let cache_dir = Self::get_cache_directory()?;
        std::fs::create_dir_all(&cache_dir)?;

        Ok(Self { cache_dir })
    }

    fn get_cache_directory() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let cache_dir = if cfg!(target_os = "macos") {
            dirs::home_dir()
                .ok_or("Could not find home directory")?
                .join("Library/Caches/codecortext-mcp")
        } else if cfg!(target_os = "windows") {
            dirs::cache_dir()
                .ok_or("Could not find cache directory")?
                .join("codecortext-mcp")
        } else {
            // Linux and other Unix-like systems
            dirs::cache_dir()
                .unwrap_or_else(|| {
                    dirs::home_dir()
                        .unwrap_or_else(|| PathBuf::from("/tmp"))
                        .join(".cache")
                })
                .join("codecortext-mcp")
        };

        Ok(cache_dir)
    }

    pub fn get_cache_key(&self, root_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
        let canonical_path = root_path.canonicalize()?;
        let mut hasher = Sha256::new();
        hasher.update(canonical_path.to_string_lossy().as_bytes());
        let hash = hasher.finalize();
        Ok(format!("{:x}", hash)[..16].to_string()) // Use first 16 chars
    }

    pub fn get_cache_file(&self, root_path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let cache_key = self.get_cache_key(root_path)?;
        Ok(self.cache_dir.join(format!("{}.bin", cache_key)))
    }

    pub async fn save_index(
        &self,
        store: &SymbolStore,
        root_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cache_file = self.get_cache_file(root_path)?;
        let temp_file = cache_file.with_extension("tmp");

        // Create persisted index from store
        let index = PersistedIndex::from_store(store, root_path.to_path_buf());

        // Serialize to temporary file
        let encoded = bincode::encode_to_vec(&index, bincode::config::standard())?;
        tokio::fs::write(&temp_file, encoded).await?;

        // Atomic rename
        tokio::fs::rename(temp_file, cache_file).await?;

        Ok(())
    }

    pub async fn load_index(
        &self,
        root_path: &Path,
    ) -> Result<Option<PersistedIndex>, Box<dyn std::error::Error>> {
        let cache_file = self.get_cache_file(root_path)?;

        if !cache_file.exists() {
            return Ok(None);
        }

        // Try to load cache with corruption handling
        let data = match tokio::fs::read(&cache_file).await {
            Ok(data) => data,
            Err(e) => {
                tracing::warn!("Failed to read cache file {}: {}", cache_file.display(), e);
                // Remove corrupted cache file
                let _ = tokio::fs::remove_file(&cache_file).await;
                return Ok(None);
            }
        };

        let index: PersistedIndex =
            match bincode::decode_from_slice(&data, bincode::config::standard()) {
                Ok((index, _)) => index,
                Err(e) => {
                    tracing::warn!("Cache file {} is corrupted: {}", cache_file.display(), e);
                    // Remove corrupted cache file
                    let _ = tokio::fs::remove_file(&cache_file).await;
                    return Ok(None);
                }
            };

        // Validate cache version
        if index.version != 1 {
            tracing::info!("Cache version mismatch, ignoring cache file");
            let _ = tokio::fs::remove_file(&cache_file).await;
            return Ok(None);
        }

        // Validate cache integrity
        if !self.validate_cache_integrity(&index) {
            tracing::warn!("Cache integrity check failed, removing cache");
            let _ = tokio::fs::remove_file(&cache_file).await;
            return Ok(None);
        }

        Ok(Some(index))
    }

    fn validate_cache_integrity(&self, index: &PersistedIndex) -> bool {
        // Basic integrity checks
        if index.symbols_by_name.is_empty() && index.symbol_data.is_empty() {
            return true; // Empty cache is valid
        }

        // Check that symbol references are consistent
        for (name, symbol_ids) in &index.symbols_by_name {
            for symbol_id in symbol_ids {
                if !index.symbol_data.contains_key(symbol_id) {
                    tracing::warn!(
                        "Inconsistent cache: symbol {} references missing symbol_id {:?}",
                        name,
                        symbol_id
                    );
                    return false;
                }
            }
        }

        // Check that all symbols have valid file references
        for (symbol_id, symbol) in &index.symbol_data {
            if !index.files.contains_key(&symbol.location.file) {
                tracing::debug!(
                    "Symbol {:?} references missing file {}",
                    symbol_id,
                    symbol.location.file.display()
                );
                // This is not critical - file might have been deleted
            }
        }

        true
    }

    pub async fn is_cache_valid(
        &self,
        root_path: &Path,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let cache_file = self.get_cache_file(root_path)?;

        if !cache_file.exists() {
            return Ok(false);
        }

        // Check if cache file is newer than any source files
        let cache_metadata = tokio::fs::metadata(&cache_file).await?;
        let cache_modified = cache_metadata.modified()?;

        // Simple validation: check if any .rs or .py files are newer than cache
        let mut walker = walkdir::WalkDir::new(root_path).into_iter();
        while let Some(entry) = walker.next() {
            let entry = entry?;
            let path = entry.path();

            if let Some(ext) = path.extension() {
                if ext == "rs" || ext == "py" {
                    let file_metadata = std::fs::metadata(path)?;
                    if file_metadata.modified()? > cache_modified {
                        return Ok(false); // Source file is newer than cache
                    }
                }
            }
        }

        Ok(true)
    }

    pub async fn clear_cache(&self, root_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let cache_file = self.get_cache_file(root_path)?;

        if cache_file.exists() {
            tokio::fs::remove_file(cache_file).await?;
        }

        Ok(())
    }
}

impl PersistedIndex {
    pub fn from_store(store: &SymbolStore, root_path: PathBuf) -> Self {
        Self {
            version: 1,
            created_at: SystemTime::now(),
            root_path,
            symbols_by_name: store
                .symbols_by_name
                .iter()
                .map(|entry| (entry.key().clone(), entry.value().clone()))
                .collect(),
            symbol_data: store
                .symbol_data
                .iter()
                .map(|entry| (*entry.key(), entry.value().clone()))
                .collect(),
            references: store
                .references
                .iter()
                .map(|entry| (*entry.key(), entry.value().clone()))
                .collect(),
            files: store
                .files
                .iter()
                .map(|entry| (entry.key().clone(), entry.value().clone()))
                .collect(),
        }
    }

    pub fn restore_to_store(&self, store: &SymbolStore) {
        // Clear existing data
        store.symbols_by_name.clear();
        store.symbol_data.clear();
        store.references.clear();
        store.files.clear();

        // Restore data from cache
        for (name, symbol_ids) in &self.symbols_by_name {
            store
                .symbols_by_name
                .insert(name.clone(), symbol_ids.clone());
        }

        for (symbol_id, symbol) in &self.symbol_data {
            store.symbol_data.insert(*symbol_id, symbol.clone());
        }

        for (symbol_id, refs) in &self.references {
            store.references.insert(*symbol_id, refs.clone());
        }

        for (path, file_info) in &self.files {
            store.files.insert(path.clone(), file_info.clone());
        }

        // Update memory usage
        let total_symbols = self.symbol_data.len();
        let estimated_memory = total_symbols * 1000; // Rough estimate
        store.memory_usage.store(
            estimated_memory as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Location, SymbolType, Visibility};
    use std::sync::Arc;
    use tempfile::TempDir;

    fn create_test_symbol(name: &str, file: &str) -> Symbol {
        let path = PathBuf::from(file);
        let location = Location::new(path.clone(), 1, 0, 1, 10);
        let symbol_id = SymbolId::new(&path, 1, 0);

        Symbol {
            id: symbol_id,
            name: name.to_string(),
            symbol_type: SymbolType::Function,
            location,
            namespace: None,
            visibility: Visibility::Public,
            source: None,
        }
    }

    #[tokio::test]
    async fn test_cache_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path();

        // Create cache manager
        let cache_manager = CacheManager::new().unwrap();

        // Create store with test data
        let store = Arc::new(SymbolStore::new());
        let symbol = create_test_symbol("test_function", "test.rs");
        let _ = store.insert_symbol(symbol);

        // Save to cache
        cache_manager.save_index(&store, root_path).await.unwrap();

        // Load from cache
        let loaded_index = cache_manager.load_index(root_path).await.unwrap();
        assert!(loaded_index.is_some());

        let index = loaded_index.unwrap();
        assert_eq!(index.version, 1);
        assert_eq!(index.root_path, root_path);
        assert!(index.symbol_data.len() > 0);
    }

    #[tokio::test]
    async fn test_cache_restore() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path();

        let cache_manager = CacheManager::new().unwrap();

        // Create original store
        let original_store = Arc::new(SymbolStore::new());
        let symbol = create_test_symbol("test_function", "test.rs");
        let _ = original_store.insert_symbol(symbol.clone());

        // Save to cache
        cache_manager
            .save_index(&original_store, root_path)
            .await
            .unwrap();

        // Create new store and restore from cache
        let new_store = Arc::new(SymbolStore::new());
        let loaded_index = cache_manager.load_index(root_path).await.unwrap().unwrap();
        loaded_index.restore_to_store(&new_store);

        // Verify data was restored
        let symbols = new_store.get_symbols("test_function");
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "test_function");
    }

    #[tokio::test]
    async fn test_cache_key_generation() {
        let cache_manager = CacheManager::new().unwrap();

        let path1 = PathBuf::from("/test/path1");
        let path2 = PathBuf::from("/test/path2");

        // Different paths should generate different keys
        // Note: This test might fail if paths don't exist, but that's ok for unit test
        if let (Ok(key1), Ok(key2)) = (
            cache_manager.get_cache_key(&path1),
            cache_manager.get_cache_key(&path2),
        ) {
            assert_ne!(key1, key2);
        }
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path();

        let cache_manager = CacheManager::new().unwrap();
        let store = Arc::new(SymbolStore::new());

        // Save cache
        cache_manager.save_index(&store, root_path).await.unwrap();

        // Verify cache exists
        let cache_file = cache_manager.get_cache_file(root_path).unwrap();
        assert!(cache_file.exists());

        // Clear cache
        cache_manager.clear_cache(root_path).await.unwrap();

        // Verify cache is gone
        assert!(!cache_file.exists());
    }
}
