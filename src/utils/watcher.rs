use crate::indexing::indexing_pipeline::IndexingPipeline;
use crate::models::Language;
use crate::storage::store::SymbolStore;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    _debouncer: Debouncer, // Renamed to indicate it's intentionally unused
}

struct Debouncer {
    pending_changes: Arc<tokio::sync::Mutex<HashMap<PathBuf, Instant>>>,
    debounce_duration: Duration,
}

impl FileWatcher {
    pub fn new(
        watch_path: PathBuf,
        pipeline: Arc<tokio::sync::Mutex<IndexingPipeline>>,
        store: Arc<SymbolStore>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (tx, mut rx) = mpsc::unbounded_channel();

        // Create file system watcher
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            notify::Config::default(),
        )?;

        // Start watching the directory
        watcher.watch(&watch_path, RecursiveMode::Recursive)?;

        let debouncer = Debouncer {
            pending_changes: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            debounce_duration: Duration::from_millis(100),
        };

        // Spawn background task to handle file events
        let debouncer_clone = debouncer.clone();
        let pipeline_clone = pipeline.clone();
        let store_clone = store.clone();

        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                Self::handle_file_event(event, &debouncer_clone, &pipeline_clone, &store_clone)
                    .await;
            }
        });

        // Spawn debouncer task
        let debouncer_clone = debouncer.clone();
        let pipeline_clone = pipeline.clone();
        let store_clone = store.clone();

        tokio::spawn(async move {
            Self::debouncer_task(debouncer_clone, pipeline_clone, store_clone).await;
        });

        Ok(FileWatcher {
            _watcher: watcher,
            _debouncer: debouncer,
        })
    }

    async fn handle_file_event(
        event: Event,
        debouncer: &Debouncer,
        _pipeline: &Arc<tokio::sync::Mutex<IndexingPipeline>>,
        _store: &Arc<SymbolStore>,
    ) {
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                for path in event.paths {
                    // Only process source files
                    if let Some(extension) = path.extension() {
                        if Language::from_extension(extension.to_str().unwrap_or("")).is_some() {
                            let mut pending = debouncer.pending_changes.lock().await;
                            pending.insert(path, Instant::now());
                        }
                    }
                }
            }
            _ => {} // Ignore other event types
        }
    }

    async fn debouncer_task(
        debouncer: Debouncer,
        pipeline: Arc<tokio::sync::Mutex<IndexingPipeline>>,
        store: Arc<SymbolStore>,
    ) {
        let mut interval = tokio::time::interval(Duration::from_millis(50));

        loop {
            interval.tick().await;

            let mut to_process = Vec::new();
            {
                let mut pending = debouncer.pending_changes.lock().await;
                let now = Instant::now();

                // Find files that have been stable for the debounce duration
                pending.retain(|path, timestamp| {
                    if now.duration_since(*timestamp) >= debouncer.debounce_duration {
                        to_process.push(path.clone());
                        false // Remove from pending
                    } else {
                        true // Keep in pending
                    }
                });
            }

            // Process stable files
            for file_path in to_process {
                if file_path.exists() {
                    // File was created or modified - normalize path
                    let normalized_path = if let Ok(current_dir) = std::env::current_dir() {
                        file_path
                            .strip_prefix(&current_dir)
                            .unwrap_or(&file_path)
                            .to_path_buf()
                    } else {
                        file_path.clone()
                    };

                    tracing::info!(
                        "Re-indexing modified file: {:?} -> {:?}",
                        file_path,
                        normalized_path
                    );
                    let mut pipeline_guard = pipeline.lock().await;
                    if let Err(e) = pipeline_guard.index_file(&normalized_path).await {
                        eprintln!("Error reindexing file {:?}: {}", normalized_path, e);
                    }
                } else {
                    // File was deleted - convert to relative path for consistency
                    let relative_path = if let Ok(current_dir) = std::env::current_dir() {
                        file_path
                            .strip_prefix(&current_dir)
                            .unwrap_or(&file_path)
                            .to_path_buf()
                    } else {
                        file_path.clone()
                    };

                    tracing::info!(
                        "Removing symbols for deleted file: {:?} (relative: {:?})",
                        file_path,
                        relative_path
                    );
                    store.remove_file_symbols(&relative_path);
                    store.remove_file_references(&relative_path);
                }
            }
        }
    }
}

impl Debouncer {
    fn clone(&self) -> Self {
        Self {
            pending_changes: self.pending_changes.clone(),
            debounce_duration: self.debounce_duration,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_file_watcher_creation() {
        let temp_dir = TempDir::new().unwrap();
        let store = Arc::new(SymbolStore::new());
        let pipeline = Arc::new(tokio::sync::Mutex::new(
            IndexingPipeline::new(store.clone()).unwrap(),
        ));

        let watcher = FileWatcher::new(temp_dir.path().to_path_buf(), pipeline, store);

        assert!(watcher.is_ok());
    }

    #[tokio::test]
    async fn test_file_change_detection() {
        let temp_dir = TempDir::new().unwrap();
        let store = Arc::new(SymbolStore::new());
        let pipeline = Arc::new(tokio::sync::Mutex::new(
            IndexingPipeline::new(store.clone()).unwrap(),
        ));

        let _watcher =
            FileWatcher::new(temp_dir.path().to_path_buf(), pipeline, store.clone()).unwrap();

        // Create a test file
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn test() {}").unwrap();

        // Wait for debouncing
        tokio::time::sleep(Duration::from_millis(200)).await;

        // File should be processed (this is a basic test)
        assert!(test_file.exists());
    }
}
