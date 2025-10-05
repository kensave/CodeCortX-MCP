use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug, Clone)]
struct AccessInfo {
    last_accessed: Instant,
    access_count: u64,
}

pub struct LruTracker {
    file_access: Arc<Mutex<HashMap<PathBuf, AccessInfo>>>,
    max_entries: usize,
}

impl LruTracker {
    pub fn new(max_entries: usize) -> Self {
        Self {
            file_access: Arc::new(Mutex::new(HashMap::new())),
            max_entries,
        }
    }

    pub fn track_access(&self, file_path: &PathBuf) {
        let mut access_map = self.file_access.lock().unwrap();

        let info = access_map.entry(file_path.clone()).or_insert(AccessInfo {
            last_accessed: Instant::now(),
            access_count: 0,
        });

        info.last_accessed = Instant::now();
        info.access_count += 1;

        // Limit the number of tracked files
        if access_map.len() > self.max_entries {
            self.cleanup_old_entries(&mut access_map);
        }
    }

    pub fn get_lru_files(&self, count: usize) -> Vec<PathBuf> {
        let access_map = self.file_access.lock().unwrap();

        let mut files: Vec<(PathBuf, Instant)> = access_map
            .iter()
            .map(|(path, info)| (path.clone(), info.last_accessed))
            .collect();

        // Sort by last accessed time (oldest first)
        files.sort_by_key(|(_, time)| *time);

        files
            .into_iter()
            .take(count)
            .map(|(path, _)| path)
            .collect()
    }

    pub fn remove_file(&self, file_path: &PathBuf) {
        let mut access_map = self.file_access.lock().unwrap();
        access_map.remove(file_path);
    }

    pub fn get_file_count(&self) -> usize {
        let access_map = self.file_access.lock().unwrap();
        access_map.len()
    }

    fn cleanup_old_entries(&self, access_map: &mut HashMap<PathBuf, AccessInfo>) {
        let target_size = self.max_entries * 3 / 4; // Remove 25% when full

        if access_map.len() <= target_size {
            return;
        }

        let mut entries: Vec<(PathBuf, Instant)> = access_map
            .iter()
            .map(|(path, info)| (path.clone(), info.last_accessed))
            .collect();

        entries.sort_by_key(|(_, time)| *time);

        let to_remove = access_map.len() - target_size;
        for (path, _) in entries.into_iter().take(to_remove) {
            access_map.remove(&path);
        }
    }
}

pub struct LruEvictionManager {
    lru_tracker: LruTracker,
}

impl LruEvictionManager {
    pub fn new() -> Self {
        Self {
            lru_tracker: LruTracker::new(10000), // Track up to 10k files
        }
    }

    pub fn track_file_access(&self, file_path: &PathBuf) {
        self.lru_tracker.track_access(file_path);
    }

    pub fn evict_files_if_needed(
        &self,
        store: &crate::storage::store::SymbolStore,
        target_files: usize,
    ) -> Vec<PathBuf> {
        if !store.should_cleanup_memory() {
            return Vec::new();
        }

        let lru_files = self.lru_tracker.get_lru_files(target_files);
        let mut evicted = Vec::new();

        for file_path in lru_files {
            // Check if file still exists in store
            if store.has_file(&file_path) {
                store.remove_file_symbols(&file_path);
                self.lru_tracker.remove_file(&file_path);
                evicted.push(file_path);

                // Stop if memory pressure is relieved
                if !store.is_memory_under_pressure() {
                    break;
                }
            }
        }

        evicted
    }

    pub fn get_stats(&self) -> LruStats {
        LruStats {
            tracked_files: self.lru_tracker.get_file_count(),
        }
    }
}

#[derive(Debug)]
pub struct LruStats {
    pub tracked_files: usize,
}

impl Default for LruEvictionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_lru_tracking() {
        let tracker = LruTracker::new(100);

        let file1 = PathBuf::from("file1.rs");
        let file2 = PathBuf::from("file2.rs");
        let file3 = PathBuf::from("file3.rs");

        // Access files in order
        tracker.track_access(&file1);
        thread::sleep(Duration::from_millis(1));
        tracker.track_access(&file2);
        thread::sleep(Duration::from_millis(1));
        tracker.track_access(&file3);

        // Get LRU files - should return in order of access (oldest first)
        let lru_files = tracker.get_lru_files(2);
        assert_eq!(lru_files.len(), 2);
        assert_eq!(lru_files[0], file1); // Oldest
        assert_eq!(lru_files[1], file2);
    }

    #[test]
    fn test_lru_access_update() {
        let tracker = LruTracker::new(100);

        let file1 = PathBuf::from("file1.rs");
        let file2 = PathBuf::from("file2.rs");

        // Initial access
        tracker.track_access(&file1);
        thread::sleep(Duration::from_millis(1));
        tracker.track_access(&file2);

        // Access file1 again (should update its timestamp)
        thread::sleep(Duration::from_millis(1));
        tracker.track_access(&file1);

        // Now file2 should be the LRU
        let lru_files = tracker.get_lru_files(1);
        assert_eq!(lru_files[0], file2);
    }

    #[test]
    fn test_file_removal() {
        let tracker = LruTracker::new(100);

        let file1 = PathBuf::from("file1.rs");
        tracker.track_access(&file1);

        assert_eq!(tracker.get_file_count(), 1);

        tracker.remove_file(&file1);
        assert_eq!(tracker.get_file_count(), 0);
    }

    #[test]
    fn test_eviction_manager() {
        use crate::storage::store::SymbolStore;
        use crate::utils::memory::MemoryManager;
        use std::sync::Arc;

        // Create store with small memory limit for testing
        let memory_manager = Arc::new(MemoryManager::new(1)); // 1MB limit
        let store = SymbolStore::with_memory_manager(memory_manager);
        let eviction_manager = LruEvictionManager::new();

        let file1 = PathBuf::from("file1.rs");
        let file2 = PathBuf::from("file2.rs");

        eviction_manager.track_file_access(&file1);
        eviction_manager.track_file_access(&file2);

        // Test eviction (won't actually evict since no files in store)
        let evicted = eviction_manager.evict_files_if_needed(&store, 1);
        assert!(evicted.is_empty()); // No files to evict
    }
}
