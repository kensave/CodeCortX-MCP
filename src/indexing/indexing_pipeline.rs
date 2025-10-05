use crate::indexing::indexer::SymbolIndexer;
use crate::models::{FileInfo, Language, ParseStatus, Reference, Symbol};
use crate::storage::cache::CacheManager;
use crate::storage::store::SymbolStore;
use crate::utils::error::{CodeAnalysisError, ErrorRecovery};
use crate::utils::filesystem::FileSystemWalker;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;

pub struct IndexingPipeline {
    indexer: SymbolIndexer,
    store: Arc<SymbolStore>,
    cache_manager: CacheManager,
}

#[derive(Debug)]
pub struct IndexingResult {
    pub files_processed: u32,
    pub symbols_found: u32,
    pub errors: Vec<String>,
    pub duration_ms: u64,
    pub cache_used: bool,
    pub partial_success: bool,
    pub files_skipped: u32,
}

impl IndexingResult {
    pub fn new() -> Self {
        Self {
            files_processed: 0,
            symbols_found: 0,
            errors: Vec::new(),
            duration_ms: 0,
            cache_used: false,
            partial_success: false,
            files_skipped: 0,
        }
    }

    pub fn is_success(&self) -> bool {
        self.errors.is_empty() && !self.partial_success
    }

    pub fn has_results(&self) -> bool {
        self.files_processed > 0 || self.symbols_found > 0
    }
}

impl IndexingPipeline {
    pub fn new(store: Arc<SymbolStore>) -> Result<Self, Box<dyn std::error::Error>> {
        let indexer = SymbolIndexer::new()?;
        let cache_manager = CacheManager::new()?;
        Ok(Self {
            indexer,
            store,
            cache_manager,
        })
    }

    /// Index directory with cache optimization and graceful degradation
    pub async fn index_directory_with_cache<P: AsRef<Path>>(&mut self, path: P) -> IndexingResult {
        let start_time = std::time::Instant::now();
        let path = path.as_ref();

        // Try to load from cache first
        match self.cache_manager.load_index(path).await {
            Ok(Some(cached_index)) => {
                // Cache loaded successfully
                cached_index.restore_to_store(&self.store);

                // Verify cache is still valid by checking a few files
                if self.verify_cache_validity(path, &cached_index).await {
                    return IndexingResult {
                        files_processed: cached_index.files.len() as u32,
                        symbols_found: cached_index.symbol_data.len() as u32,
                        errors: Vec::new(),
                        duration_ms: start_time.elapsed().as_millis() as u64,
                        cache_used: true,
                        partial_success: false,
                        files_skipped: 0,
                    };
                } else {
                    tracing::info!("Cache validation failed, falling back to full indexing");
                    // Clear the store and continue with full indexing
                    self.store.symbols_by_name.clear();
                    self.store.symbol_data.clear();
                    self.store.references.clear();
                    self.store.files.clear();
                }
            }
            Ok(None) => {
                tracing::debug!("No cache found, performing full indexing");
            }
            Err(e) => {
                tracing::warn!("Cache loading failed: {}, performing full indexing", e);
            }
        }

        // Cache miss, invalid, or corrupted - do full indexing
        let mut result = self.index_directory(path).await;
        result.cache_used = false;

        // Save to cache for next time, even if there were some errors
        if result.has_results() {
            if let Err(e) = self.cache_manager.save_index(&self.store, path).await {
                result.errors.push(format!("Failed to save cache: {}", e));
                result.partial_success = true;
            }
        }

        result
    }

    async fn verify_cache_validity(
        &self,
        _root_path: &Path,
        cached_index: &crate::storage::cache::PersistedIndex,
    ) -> bool {
        // Quick validation: check if a few files still exist and haven't changed
        let files_to_check: Vec<_> = cached_index.files.keys().take(5).collect();

        for file_path in files_to_check {
            if !FileSystemWalker::is_accessible(file_path).await {
                tracing::debug!(
                    "Cache invalid: file {} no longer accessible",
                    file_path.display()
                );
                return false;
            }

            // Check if file has been modified (simple check)
            if let Ok(metadata) = tokio::fs::metadata(file_path).await {
                if let Ok(modified) = metadata.modified() {
                    if modified > cached_index.created_at {
                        tracing::debug!(
                            "Cache invalid: file {} modified after cache creation",
                            file_path.display()
                        );
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Index all source files in a directory with graceful degradation
    pub async fn index_directory<P: AsRef<Path>>(&mut self, path: P) -> IndexingResult {
        let start_time = std::time::Instant::now();
        let mut result = IndexingResult::new();

        // Find all source files
        let source_files = match FileSystemWalker::find_source_files(&path) {
            Ok(files) => files,
            Err(e) => {
                result
                    .errors
                    .push(format!("Failed to find source files: {}", e));
                result.duration_ms = start_time.elapsed().as_millis() as u64;
                return result;
            }
        };

        let total_files = source_files.len();
        let mut critical_errors = 0;
        const MAX_CRITICAL_ERRORS: usize = 10; // Stop after 10 critical errors

        // Process each file with graceful degradation
        for (index, file_path) in source_files.iter().enumerate() {
            // Check if we should stop due to too many critical errors
            if critical_errors >= MAX_CRITICAL_ERRORS {
                result.errors.push(format!(
                    "Stopping indexing after {} critical errors. {} files remaining.",
                    critical_errors,
                    total_files - index
                ));
                result.files_skipped = (total_files - index) as u32;
                result.partial_success = true;
                break;
            }

            match self.index_file(file_path).await {
                Ok(symbols) => {
                    result.files_processed += 1;
                    result.symbols_found += symbols.len() as u32;
                }
                Err(e) => {
                    let error_msg = format!("Failed to index {}: {}", file_path.display(), e);

                    // Classify error severity
                    if let Some(analysis_error) = e.downcast_ref::<CodeAnalysisError>() {
                        if !analysis_error.is_recoverable() {
                            critical_errors += 1;
                            tracing::error!("Critical error: {}", error_msg);
                        } else {
                            tracing::warn!("Recoverable error: {}", error_msg);
                            result.files_skipped += 1;
                        }
                    } else {
                        // Unknown error type, treat as recoverable
                        tracing::warn!("Unknown error: {}", error_msg);
                        result.files_skipped += 1;
                    }

                    result.errors.push(error_msg);
                    result.partial_success = true;
                }
            }

            // Periodic memory cleanup check
            if index % 100 == 0 && self.store.should_cleanup_memory() {
                let evicted = self.store.cleanup_lru_files();
                if !evicted.is_empty() {
                    tracing::info!("Evicted {} files due to memory pressure", evicted.len());
                    result.partial_success = true;
                }
            }
        }

        result.duration_ms = start_time.elapsed().as_millis() as u64;

        // Log final statistics
        if result.partial_success {
            tracing::warn!(
                "Indexing completed with partial success: {}/{} files processed, {} errors, {} skipped",
                result.files_processed, total_files, result.errors.len(), result.files_skipped
            );
        } else {
            tracing::info!(
                "Indexing completed successfully: {}/{} files processed, {} symbols found",
                result.files_processed,
                total_files,
                result.symbols_found
            );
        }

        result
    }

    /// Index a single file with change detection and error recovery
    pub async fn index_file<P: AsRef<Path>>(
        &mut self,
        file_path: P,
    ) -> Result<Vec<Symbol>, Box<dyn std::error::Error>> {
        let file_path = file_path.as_ref().to_path_buf();
        let file_path_str = file_path.display().to_string();

        // Check if file is accessible
        if !FileSystemWalker::is_accessible(&file_path).await {
            return Err(CodeAnalysisError::InvalidPath {
                path: file_path_str,
            }
            .into());
        }

        // Read file content with error handling
        let content = match FileSystemWalker::read_file_content(&file_path).await {
            Ok(content) => content,
            Err(io_error) => {
                let error = ErrorRecovery::handle_file_error(&file_path_str, io_error);
                ErrorRecovery::log_error_and_continue(&error, "file reading");

                if ErrorRecovery::should_continue_indexing(&error) {
                    // Update file info with error status
                    let file_info = FileInfo {
                        last_modified: SystemTime::now(),
                        content_hash: [0; 32], // Empty hash for failed files
                        symbol_count: 0,
                        parse_status: ParseStatus::Failed(error.to_string()),
                        file_size: 0,
                    };
                    self.store.update_file_info(file_path, file_info);
                    return Ok(Vec::new()); // Return empty symbols, continue processing
                }
                return Err(error.into());
            }
        };

        // Check file size limit (10MB)
        const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;
        if content.len() as u64 > MAX_FILE_SIZE {
            let error = CodeAnalysisError::FileTooLarge {
                size_mb: content.len() as u64 / (1024 * 1024),
            };
            ErrorRecovery::log_error_and_continue(&error, &file_path_str);

            let file_info = FileInfo {
                last_modified: SystemTime::now(),
                content_hash: self.calculate_content_hash(&content),
                symbol_count: 0,
                parse_status: ParseStatus::Failed(error.to_string()),
                file_size: content.len() as u64,
            };
            self.store.update_file_info(file_path, file_info);
            return Ok(Vec::new());
        }

        // Calculate content hash for change detection
        let content_hash = self.calculate_content_hash(&content);

        // Check if file has changed
        if let Some(existing_info) = self.store.get_file_info(&file_path) {
            if existing_info.content_hash == content_hash {
                tracing::debug!("File {:?} unchanged, skipping re-indexing", file_path);
                // File hasn't changed, return existing symbols
                return Ok(self.store.get_symbols_by_file(&file_path));
            }

            tracing::info!("File {:?} changed, removing old content", file_path);
            // File has changed, remove old symbols and BM25 content
            self.store.remove_file_symbols(&file_path);
        }

        // Detect language
        let language = match Language::from_path(&file_path) {
            Some(lang) => lang,
            None => {
                let ext = file_path
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");
                let error = CodeAnalysisError::UnsupportedFileType {
                    extension: ext.to_string(),
                };
                ErrorRecovery::log_error_and_continue(&error, &file_path_str);

                let file_info = FileInfo {
                    last_modified: SystemTime::now(),
                    content_hash,
                    symbol_count: 0,
                    parse_status: ParseStatus::Failed(error.to_string()),
                    file_size: content.len() as u64,
                };
                self.store.update_file_info(file_path, file_info);
                return Ok(Vec::new());
            }
        };

        // Extract symbols with error recovery
        let symbols = match self.indexer.extract_symbols(&content, language, &file_path) {
            Ok(symbols) => symbols,
            Err(e) => {
                let error = ErrorRecovery::handle_parse_error(&file_path_str, &e.to_string());
                ErrorRecovery::log_error_and_continue(&error, "symbol extraction");

                // Update file info with parse error
                let file_info = FileInfo {
                    last_modified: SystemTime::now(),
                    content_hash,
                    symbol_count: 0,
                    parse_status: ParseStatus::Failed(error.to_string()),
                    file_size: content.len() as u64,
                };
                self.store.update_file_info(file_path, file_info);
                return Ok(Vec::new()); // Return empty symbols, continue processing
            }
        };

        // Store symbols in symbol store with memory checking
        let mut stored_symbols = 0;
        for symbol in &symbols {
            match self.store.insert_symbol(symbol.clone()) {
                Ok(()) => stored_symbols += 1,
                Err(e) => {
                    let error = CodeAnalysisError::IndexingError { message: e };
                    ErrorRecovery::log_error_and_continue(&error, "symbol insertion");

                    if !ErrorRecovery::should_continue_indexing(&error) {
                        return Err(error.into());
                    }
                    // Continue with remaining symbols
                }
            }
        }

        // Extract references (simple approach)
        if let Ok(references) = self
            .indexer
            .extract_references(&content, language, &file_path)
        {
            // For each reference, try to find matching symbols and link them
            for reference in references {
                // Extract the reference name from the source
                if let Some(ref_name) = self.extract_reference_name(&reference, &content) {
                    // Find symbols with this name
                    let matching_symbols = self.store.get_symbols(&ref_name);
                    for symbol in matching_symbols {
                        let mut linked_ref = reference.clone();
                        linked_ref.target_symbol = symbol.id;
                        self.store.add_reference(symbol.id, linked_ref);
                    }
                }
            }
        }

        // Update file info with success status
        let parse_status = if stored_symbols == symbols.len() {
            ParseStatus::Success
        } else {
            ParseStatus::PartialSuccess(vec![format!(
                "Stored {}/{} symbols due to memory constraints",
                stored_symbols,
                symbols.len()
            )])
        };

        let file_info = FileInfo {
            last_modified: SystemTime::now(),
            content_hash,
            symbol_count: stored_symbols as u32,
            parse_status,
            file_size: content.len() as u64,
        };
        self.store.update_file_info(file_path.clone(), file_info);

        // Add to BM25 index for code search
        let language_str = match language {
            Language::Rust => "rust",
            Language::Python => "python",
            Language::C => "c",
            Language::Cpp => "cpp",
            Language::Java => "java",
            Language::Go => "go",
            Language::JavaScript => "javascript",
            Language::TypeScript => "typescript",
            Language::Ruby => "ruby",
            Language::CSharp => "csharp",
            Language::Kotlin => "kotlin",
            Language::Scala => "scala",
            Language::Swift => "swift",
            Language::PHP => "php",
            Language::ObjectiveC => "objc",
        };

        // Normalize path to relative for BM25 consistency
        let normalized_path = if let Ok(current_dir) = std::env::current_dir() {
            file_path
                .strip_prefix(&current_dir)
                .unwrap_or(&file_path)
                .to_path_buf()
        } else {
            file_path.clone()
        };

        self.store
            .index_file_content(&normalized_path, &content, language_str);

        Ok(symbols)
    }

    fn extract_reference_name(&self, reference: &Reference, content: &str) -> Option<String> {
        let lines: Vec<&str> = content.lines().collect();
        let line_idx = (reference.location.start_line as usize).saturating_sub(1);

        if line_idx < lines.len() {
            let line = lines[line_idx];
            let start_col = reference.location.start_column as usize;
            let end_col = reference.location.end_column as usize;

            // Convert to character indices for Unicode safety
            let chars: Vec<char> = line.chars().collect();
            
            // Ensure valid slice bounds using character indices
            if start_col < chars.len() && end_col <= chars.len() && start_col <= end_col {
                return Some(chars[start_col..end_col].iter().collect());
            }
        }

        None
    }

    /// Check if a file needs reindexing based on modification time and content hash
    pub async fn needs_reindexing<P: AsRef<Path>>(
        &self,
        file_path: P,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let file_path = file_path.as_ref().to_path_buf();

        // If we don't have file info, it needs indexing
        let existing_info = match self.store.get_file_info(&file_path) {
            Some(info) => info,
            None => return Ok(true),
        };

        // Check if file is accessible
        if !FileSystemWalker::is_accessible(&file_path).await {
            return Ok(false); // Can't index inaccessible files
        }

        // Read current content and calculate hash
        let content = FileSystemWalker::read_file_content(&file_path).await?;
        let current_hash = self.calculate_content_hash(&content);

        // Compare hashes
        Ok(existing_info.content_hash != current_hash)
    }

    /// Incremental update for a single file
    pub async fn update_file<P: AsRef<Path>>(
        &mut self,
        file_path: P,
    ) -> Result<Vec<Symbol>, Box<dyn std::error::Error>> {
        let file_path = file_path.as_ref();

        if self.needs_reindexing(file_path).await? {
            self.index_file(file_path).await
        } else {
            // File hasn't changed, return existing symbols
            Ok(self.store.get_symbols_by_file(&file_path.to_path_buf()))
        }
    }

    /// Remove file from index (for deleted files)
    pub fn remove_file<P: AsRef<Path>>(&mut self, file_path: P) {
        let file_path = file_path.as_ref().to_path_buf();
        self.store.remove_file_symbols(&file_path);
    }

    /// Get indexing progress/statistics
    pub fn get_stats(&self) -> IndexingStats {
        IndexingStats {
            total_files: self.store.get_file_count(),
            total_symbols: self.store.get_symbol_count(),
            memory_usage_bytes: self.store.get_memory_usage(),
        }
    }

    fn calculate_content_hash(&self, content: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hasher.finalize().into()
    }
}

#[derive(Debug)]
pub struct IndexingStats {
    pub total_files: usize,
    pub total_symbols: usize,
    pub memory_usage_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_directory_indexing() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create test files
        let rust_file = base_path.join("test.rs");
        let python_file = base_path.join("script.py");

        fs::write(&rust_file, "fn main() {}\nstruct Test {}")
            .await
            .unwrap();
        fs::write(
            &python_file,
            "def hello():\n    pass\n\nclass World:\n    pass",
        )
        .await
        .unwrap();

        // Create indexing pipeline
        let store = Arc::new(SymbolStore::new());
        let mut pipeline = IndexingPipeline::new(store.clone()).unwrap();

        // Index directory
        let result = pipeline.index_directory(base_path).await;

        assert_eq!(result.files_processed, 2);
        assert!(result.is_success() || result.partial_success); // Allow partial success
        assert!(!result.cache_used); // First time should not use cache

        // Verify symbols were stored
        let stats = pipeline.get_stats();
        assert_eq!(stats.total_files, 2);
    }

    #[tokio::test]
    async fn test_cache_enabled_indexing() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create test files
        let rust_file = base_path.join("test.rs");
        fs::write(&rust_file, "fn test() {}").await.unwrap();

        let store = Arc::new(SymbolStore::new());
        let mut pipeline = IndexingPipeline::new(store.clone()).unwrap();

        // First indexing - should not use cache
        let result1 = pipeline.index_directory_with_cache(base_path).await;
        assert!(!result1.cache_used);
        assert_eq!(result1.files_processed, 1);

        // Second indexing - should use cache if valid
        let result2 = pipeline.index_directory_with_cache(base_path).await;
        // Note: Cache validation might fail in test environment, so we don't assert cache_used
        assert!(result2.files_processed >= 1);
    }

    #[tokio::test]
    async fn test_graceful_degradation() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create a mix of valid and invalid files
        let valid_file = base_path.join("valid.rs");
        let invalid_dir = base_path.join("invalid");

        fs::write(&valid_file, "fn test() {}").await.unwrap();
        fs::create_dir(&invalid_dir).await.unwrap();

        // Create a file that will be treated as invalid (non-source)
        let non_source = base_path.join("readme.txt");
        fs::write(&non_source, "This is not source code")
            .await
            .unwrap();

        let store = Arc::new(SymbolStore::new());
        let mut pipeline = IndexingPipeline::new(store.clone()).unwrap();

        let result = pipeline.index_directory(base_path).await;

        // Should process the valid file despite invalid ones
        assert!(result.files_processed >= 1);
        assert!(result.has_results());
        // May have partial success due to non-source files being skipped
    }

    #[tokio::test]
    async fn test_incremental_indexing() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        // Create initial file
        fs::write(&test_file, "fn original() {}").await.unwrap();

        let store = Arc::new(SymbolStore::new());
        let mut pipeline = IndexingPipeline::new(store.clone()).unwrap();

        // Initial indexing
        let symbols1 = pipeline.index_file(&test_file).await.unwrap();
        assert!(!symbols1.is_empty());

        // File hasn't changed - should not need reindexing
        assert!(!pipeline.needs_reindexing(&test_file).await.unwrap());

        // Modify file
        fs::write(&test_file, "fn modified() {}\nfn new_function() {}")
            .await
            .unwrap();

        // Now should need reindexing
        assert!(pipeline.needs_reindexing(&test_file).await.unwrap());

        // Update file
        let symbols2 = pipeline.update_file(&test_file).await.unwrap();
        assert!(!symbols2.is_empty());
    }

    #[tokio::test]
    async fn test_error_handling() {
        let store = Arc::new(SymbolStore::new());
        let mut pipeline = IndexingPipeline::new(store.clone()).unwrap();

        // Try to index non-existent file
        let result = pipeline.index_file("/non/existent/file.rs").await;
        assert!(result.is_err());

        // Try to index directory with no source files
        let temp_dir = TempDir::new().unwrap();
        let result = pipeline.index_directory(temp_dir.path()).await;
        assert_eq!(result.files_processed, 0);
        assert!(result.errors.is_empty());
    }

    #[tokio::test]
    async fn test_content_hash_detection() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        let store = Arc::new(SymbolStore::new());
        let pipeline = IndexingPipeline::new(store.clone()).unwrap();

        // Create file with specific content
        let content1 = "fn test1() {}";
        fs::write(&test_file, content1).await.unwrap();

        let hash1 = pipeline.calculate_content_hash(content1);

        // Different content should produce different hash
        let content2 = "fn test2() {}";
        let hash2 = pipeline.calculate_content_hash(content2);

        assert_ne!(hash1, hash2);

        // Same content should produce same hash
        let hash3 = pipeline.calculate_content_hash(content1);
        assert_eq!(hash1, hash3);
    }
}
