use crate::models::{FileInfo, Reference, Symbol, SymbolId};
use crate::search::bm25_index::{BM25CodeIndex, CodeSearchResult};
use crate::utils::lru::LruEvictionManager;
use crate::utils::memory::MemoryManager;
use dashmap::DashMap;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct SymbolStore {
    pub symbols_by_name: DashMap<String, Vec<SymbolId>>,
    pub symbol_data: DashMap<SymbolId, Symbol>,
    pub references: DashMap<SymbolId, Vec<Reference>>,
    pub files: DashMap<PathBuf, FileInfo>,
    pub memory_usage: AtomicU64,
    pub memory_manager: Arc<MemoryManager>,
    pub lru_manager: LruEvictionManager,
    pub bm25_index: BM25CodeIndex,
}

impl SymbolStore {
    pub fn new() -> Self {
        Self::with_memory_manager(Arc::new(MemoryManager::from_env()))
    }

    pub fn with_memory_manager(memory_manager: Arc<MemoryManager>) -> Self {
        Self {
            symbols_by_name: DashMap::new(),
            symbol_data: DashMap::new(),
            references: DashMap::new(),
            files: DashMap::new(),
            memory_usage: AtomicU64::new(0),
            memory_manager,
            lru_manager: LruEvictionManager::new(),
            bm25_index: BM25CodeIndex::new(),
        }
    }

    /// O(1) symbol lookup by name
    pub fn get_symbols(&self, name: &str) -> Vec<Symbol> {
        let symbols = if let Some(symbol_ids) = self.symbols_by_name.get(name) {
            symbol_ids
                .iter()
                .filter_map(|id| self.symbol_data.get(id))
                .map(|entry| {
                    let symbol = entry.value().clone();
                    // Track file access for LRU
                    self.lru_manager.track_file_access(&symbol.location.file);
                    symbol
                })
                .collect()
        } else {
            Vec::new()
        };

        // Trigger cleanup if needed
        if self.should_cleanup_memory() {
            self.cleanup_lru_files();
        }

        symbols
    }

    /// Find symbols by prefix matching
    pub fn find_symbols_by_prefix(&self, prefix: &str) -> Vec<Symbol> {
        let mut results = Vec::new();

        for entry in self.symbols_by_name.iter() {
            if entry.key().starts_with(prefix) {
                for symbol_id in entry.value() {
                    if let Some(symbol_entry) = self.symbol_data.get(symbol_id) {
                        results.push(symbol_entry.value().clone());
                    }
                }
            }
        }

        results
    }

    /// Add a reference for a symbol
    pub fn add_reference(&self, symbol_id: SymbolId, reference: Reference) {
        self.references
            .entry(symbol_id)
            .or_insert_with(Vec::new)
            .push(reference);
    }

    /// Find symbols using fuzzy matching
    pub fn find_symbols_fuzzy(&self, query: &str) -> Vec<(Symbol, i64)> {
        let matcher = SkimMatcherV2::default();
        let mut results = Vec::new();

        for entry in self.symbols_by_name.iter() {
            if let Some(score) = matcher.fuzzy_match(entry.key(), query) {
                for symbol_id in entry.value() {
                    if let Some(symbol_entry) = self.symbol_data.get(symbol_id) {
                        results.push((symbol_entry.value().clone(), score));
                    }
                }
            }
        }

        // Sort by score (higher is better)
        results.sort_by(|a, b| b.1.cmp(&a.1));
        results
    }

    /// Insert symbol with memory tracking
    pub fn insert_symbol(&self, symbol: Symbol) -> Result<(), String> {
        let symbol_id = symbol.id;
        let name = symbol.name.clone();

        // Calculate memory usage
        let symbol_size =
            MemoryManager::estimate_symbol_size(&symbol.name, symbol.source.as_deref());

        // Check if we can allocate this symbol
        if !self.memory_manager.can_allocate(symbol_size) {
            return Err(format!("Cannot allocate symbol: would exceed memory limit"));
        }

        // Track memory allocation
        self.memory_manager.track_allocation(symbol_size);
        self.memory_usage.fetch_add(symbol_size, Ordering::Relaxed);

        // Insert symbol data
        self.symbol_data.insert(symbol_id, symbol);

        // Update name index
        self.symbols_by_name
            .entry(name)
            .or_insert_with(Vec::new)
            .push(symbol_id);

        Ok(())
    }

    /// Insert symbol with old interface for compatibility (bypasses memory checks)
    pub fn insert_symbol_unchecked(&self, symbol: Symbol) {
        let symbol_id = symbol.id;
        let name = symbol.name.clone();

        // Calculate memory usage for tracking
        let symbol_size =
            MemoryManager::estimate_symbol_size(&symbol.name, symbol.source.as_deref());

        // Track memory allocation without checking limits
        self.memory_manager.track_allocation(symbol_size);
        self.memory_usage.fetch_add(symbol_size, Ordering::Relaxed);

        // Insert symbol data
        self.symbol_data.insert(symbol_id, symbol);

        // Update name index
        self.symbols_by_name
            .entry(name)
            .or_insert_with(Vec::new)
            .push(symbol_id);
    }

    /// Insert multiple symbols efficiently
    pub fn insert_symbols(&self, symbols: Vec<Symbol>) -> Result<(), String> {
        for symbol in symbols {
            self.insert_symbol(symbol)?;
        }
        Ok(())
    }

    /// Insert multiple symbols without memory checking (for compatibility)
    pub fn insert_symbols_unchecked(&self, symbols: Vec<Symbol>) {
        for symbol in symbols {
            self.insert_symbol_unchecked(symbol);
        }
    }

    /// Get symbol by ID
    pub fn get_symbol_by_id(&self, symbol_id: &SymbolId) -> Option<Symbol> {
        self.symbol_data
            .get(symbol_id)
            .map(|entry| entry.value().clone())
    }

    /// Get references for a symbol
    pub fn get_references(&self, symbol_id: &SymbolId) -> Vec<Reference> {
        self.references
            .get(symbol_id)
            .map(|entry| entry.value().clone())
            .unwrap_or_default()
    }

    /// Get all references for symbols with a given name
    pub fn get_references_by_name(&self, name: &str) -> Vec<Reference> {
        let mut all_references = Vec::new();

        // Find all symbols with this name
        if let Some(symbol_ids) = self.symbols_by_name.get(name) {
            for symbol_id in symbol_ids.iter() {
                if let Some(refs) = self.references.get(symbol_id) {
                    all_references.extend(refs.value().clone());
                }
            }
        }

        all_references
    }

    /// Add multiple references efficiently
    pub fn add_references(&self, symbol_id: SymbolId, references: Vec<Reference>) {
        if !references.is_empty() {
            self.references
                .entry(symbol_id)
                .or_insert_with(Vec::new)
                .extend(references);
        }
    }

    /// Remove references for a specific file
    pub fn remove_file_references(&self, file_path: &PathBuf) {
        // Remove references that point to symbols in this file
        for mut entry in self.references.iter_mut() {
            entry
                .value_mut()
                .retain(|reference| reference.location.file != *file_path);
        }

        // Remove empty reference entries
        self.references.retain(|_, refs| !refs.is_empty());
    }

    /// Get reference statistics
    pub fn get_reference_stats(&self) -> (usize, usize) {
        let total_symbols_with_refs = self.references.len();
        let total_references: usize = self
            .references
            .iter()
            .map(|entry| entry.value().len())
            .sum();

        (total_symbols_with_refs, total_references)
    }

    /// Add file content to BM25 index
    pub fn index_file_content(&self, file_path: &PathBuf, content: &str, language: &str) {
        let doc_id = self.bm25_index.add_document(
            file_path.clone(),
            content.to_string(),
            language.to_string(),
        );
        tracing::debug!("BM25 add_document for {:?}: id={}", file_path, doc_id);
    }

    /// Search code content using BM25
    pub fn search_code(
        &self,
        query: &str,
        limit: usize,
        context_lines: usize,
    ) -> Vec<CodeSearchResult> {
        self.bm25_index.search(query, limit, context_lines)
    }

    /// Remove file from BM25 index
    pub fn remove_file_from_index(&self, file_path: &PathBuf) {
        let removed = self.bm25_index.remove_document(file_path);
        tracing::debug!("BM25 remove_document for {:?}: {}", file_path, removed);
    }

    /// Remove all symbols from a file
    pub fn remove_file_symbols(&self, file_path: &PathBuf) {
        // Remove from BM25 index first
        tracing::info!(
            "Removing file symbols and BM25 content for: {:?}",
            file_path
        );
        self.remove_file_from_index(file_path);

        if let Some((_, _file_info)) = self.files.remove(file_path) {
            // Collect symbols to remove
            let mut symbols_to_remove = Vec::new();

            for entry in self.symbol_data.iter() {
                if entry.value().location.file == *file_path {
                    symbols_to_remove.push(*entry.key());
                }
            }

            // Remove symbols and track memory deallocation
            for symbol_id in symbols_to_remove {
                if let Some((_, symbol)) = self.symbol_data.remove(&symbol_id) {
                    // Calculate memory to deallocate
                    let symbol_size =
                        MemoryManager::estimate_symbol_size(&symbol.name, symbol.source.as_deref());
                    self.memory_manager.track_deallocation(symbol_size);
                    self.memory_usage.fetch_sub(symbol_size, Ordering::Relaxed);

                    // Remove from name index
                    if let Some(mut name_entry) = self.symbols_by_name.get_mut(&symbol.name) {
                        name_entry.retain(|id| *id != symbol_id);

                        // Remove empty name entries
                        if name_entry.is_empty() {
                            drop(name_entry);
                            self.symbols_by_name.remove(&symbol.name);
                        }
                    }

                    // Remove references
                    self.references.remove(&symbol_id);
                }
            }
        }
    }

    /// Get current memory usage in bytes
    pub fn get_memory_usage(&self) -> u64 {
        self.memory_usage.load(Ordering::Relaxed)
    }

    /// Get symbol count
    pub fn get_symbol_count(&self) -> usize {
        self.symbol_data.len()
    }

    /// Get file count
    pub fn get_file_count(&self) -> usize {
        self.files.len()
    }

    /// Check if file exists in store
    pub fn has_file(&self, file_path: &PathBuf) -> bool {
        self.files.contains_key(file_path)
    }

    /// Add or update file info
    pub fn update_file_info(&self, file_path: PathBuf, file_info: FileInfo) {
        self.files.insert(file_path, file_info);
    }

    /// Get file info
    pub fn get_file_info(&self, file_path: &PathBuf) -> Option<FileInfo> {
        self.files.get(file_path).map(|entry| entry.value().clone())
    }

    /// Get all symbols from a specific file
    pub fn get_symbols_by_file(&self, file_path: &PathBuf) -> Vec<Symbol> {
        // Track file access for LRU
        self.lru_manager.track_file_access(file_path);

        let mut symbols = Vec::new();

        for entry in self.symbol_data.iter() {
            if entry.value().location.file == *file_path {
                symbols.push(entry.value().clone());
            }
        }

        symbols
    }

    /// Check if memory is under pressure
    pub fn is_memory_under_pressure(&self) -> bool {
        self.memory_manager.is_under_pressure()
    }

    /// Get memory statistics
    pub fn get_memory_stats(&self) -> crate::utils::memory::MemoryStats {
        self.memory_manager.get_stats()
    }

    /// Check if we should trigger memory cleanup
    pub fn should_cleanup_memory(&self) -> bool {
        self.memory_manager.should_trigger_cleanup()
    }

    /// Perform LRU-based cleanup
    pub fn cleanup_lru_files(&self) -> Vec<PathBuf> {
        self.lru_manager.evict_files_if_needed(self, 10) // Evict up to 10 files
    }

    /// Get LRU statistics
    pub fn get_lru_stats(&self) -> crate::utils::lru::LruStats {
        self.lru_manager.get_stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Location, SymbolType, Visibility};

    fn create_test_symbol(name: &str, file: &str) -> Symbol {
        let path = PathBuf::from(file);
        // Use a simple hash of the name to create unique locations for test symbols
        let line = (name.bytes().map(|b| b as u32).sum::<u32>() % 1000) + 1;
        let location = Location::new(path.clone(), line, 0, line, 10);
        let symbol_id = SymbolId::new(&path, line, 0);

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

    #[test]
    fn test_symbol_insertion_and_lookup() {
        let store = SymbolStore::new();
        let symbol = create_test_symbol("test_function", "test.rs");

        store.insert_symbol_unchecked(symbol.clone());

        let found_symbols = store.get_symbols("test_function");
        assert_eq!(found_symbols.len(), 1);
        assert_eq!(found_symbols[0].name, "test_function");
    }

    #[test]
    fn test_prefix_search() {
        let store = SymbolStore::new();
        store.insert_symbol_unchecked(create_test_symbol("test_function", "test.rs"));
        store.insert_symbol_unchecked(create_test_symbol("test_struct", "test.rs"));
        store.insert_symbol_unchecked(create_test_symbol("other_function", "test.rs"));

        let results = store.find_symbols_by_prefix("test_");
        assert!(results.len() >= 2); // Should find at least test_function and test_struct
        let names: Vec<String> = results.iter().map(|s| s.name.clone()).collect();
        assert!(names.contains(&"test_function".to_string()));
        assert!(names.contains(&"test_struct".to_string()));
    }

    #[test]
    fn test_fuzzy_search() {
        let store = SymbolStore::new();

        // Insert test symbols
        store.insert_symbol_unchecked(create_test_symbol("test_function", "test.rs"));
        store.insert_symbol_unchecked(create_test_symbol("test_struct", "test.rs"));
        store.insert_symbol_unchecked(create_test_symbol("other_function", "test.rs"));
        store.insert_symbol_unchecked(create_test_symbol("TestClass", "test.py"));

        // Test fuzzy matching
        let results = store.find_symbols_fuzzy("tst");
        assert!(!results.is_empty());

        // Should find test_function and test_struct with good scores
        let names: Vec<String> = results.iter().map(|(s, _)| s.name.clone()).collect();
        assert!(names.contains(&"test_function".to_string()));
        assert!(names.contains(&"test_struct".to_string()));

        // Test case-insensitive fuzzy matching
        let results = store.find_symbols_fuzzy("testcls");
        assert!(!results.is_empty());
        let names: Vec<String> = results.iter().map(|(s, _)| s.name.clone()).collect();
        assert!(names.contains(&"TestClass".to_string()));
    }

    #[test]
    fn test_memory_tracking() {
        let store = SymbolStore::new();
        let initial_memory = store.memory_usage.load(Ordering::Relaxed);

        let symbol = create_test_symbol("test_function", "test.rs");
        store.insert_symbol_unchecked(symbol);

        let after_memory = store.memory_usage.load(Ordering::Relaxed);
        assert!(after_memory > initial_memory);
    }

    #[test]
    fn test_file_symbol_removal() {
        let store = SymbolStore::new();
        let file_path = PathBuf::from("test.rs");
        let symbol = create_test_symbol("test_function", "test.rs");

        // Add file info first
        let file_info = FileInfo::new([0; 32], 100);
        store.files.insert(file_path.clone(), file_info);

        store.insert_symbol_unchecked(symbol);

        // Verify symbol exists
        assert_eq!(store.get_symbols("test_function").len(), 1);

        // Remove file symbols
        store.remove_file_symbols(&file_path);

        // Verify symbol is removed
        assert_eq!(store.get_symbols("test_function").len(), 0);
    }

    #[test]
    fn test_symbol_by_id_lookup() {
        let store = SymbolStore::new();
        let symbol = create_test_symbol("test_function", "test.rs");
        let symbol_id = symbol.id;

        store.insert_symbol_unchecked(symbol.clone());

        let found_symbol = store.get_symbol_by_id(&symbol_id);
        assert!(found_symbol.is_some());
        assert_eq!(found_symbol.unwrap().name, "test_function");
    }

    #[test]
    fn test_multiple_symbol_insertion() {
        let store = SymbolStore::new();
        let symbols = vec![
            create_test_symbol("func1", "test.rs"),
            create_test_symbol("func2", "test.rs"),
            create_test_symbol("func3", "test.rs"),
        ];

        for symbol in symbols {
            store.insert_symbol_unchecked(symbol);
        }

        assert!(store.get_symbol_count() >= 3); // Should have at least 3 symbols
        assert!(store.get_symbols("func1").len() == 1);
        assert!(store.get_symbols("func2").len() == 1);
        assert!(store.get_symbols("func3").len() == 1);
    }

    #[test]
    fn test_file_info_management() {
        let store = SymbolStore::new();
        let file_path = PathBuf::from("test.rs");

        // Create file info
        let file_info = FileInfo::from_file_content("fn test() {}");
        store.update_file_info(file_path.clone(), file_info.clone());

        // Verify file info exists
        assert!(store.has_file(&file_path));
        let retrieved_info = store.get_file_info(&file_path).unwrap();
        assert_eq!(retrieved_info.content_hash, file_info.content_hash);
    }

    #[test]
    fn test_file_change_detection() {
        let content1 = "fn test() {}";
        let content2 = "fn test() { println!(\"hello\"); }";

        let info1 = FileInfo::from_file_content(content1);
        let info2 = FileInfo::from_file_content(content2);

        assert!(info1.has_changed(&info2));
        assert!(!info1.has_changed(&info1));
    }

    #[test]
    fn test_reference_management() {
        use crate::models::{Location, ReferenceType};

        let store = SymbolStore::new();
        let symbol = create_test_symbol("test_function", "test.rs");
        let symbol_id = symbol.id;

        let _ = store.insert_symbol(symbol);

        // Add reference
        let reference = Reference {
            location: Location::new(PathBuf::from("other.rs"), 5, 0, 5, 10),
            reference_type: ReferenceType::Call,
            target_symbol: symbol_id,
        };

        store.add_references(symbol_id, vec![reference.clone()]);

        // Verify reference exists
        let refs = store.get_references(&symbol_id);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].reference_type, ReferenceType::Call);

        // Test references by name
        let refs_by_name = store.get_references_by_name("test_function");
        assert_eq!(refs_by_name.len(), 1);
    }
}
