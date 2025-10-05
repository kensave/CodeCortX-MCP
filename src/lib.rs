pub mod indexing;
pub mod languages;
pub mod mcp;
pub mod search;
pub mod storage;
pub mod utils;

// Re-export main types to avoid conflicts
pub use indexing::{IndexingPipeline, SymbolIndexer};
pub use languages::*;
pub use mcp::CodeAnalysisTools;
pub use search::BM25CodeIndex;
pub use storage::{CacheManager, SymbolStore};
pub use utils::{
    CodeAnalysisError, FileSystemWalker, FileWatcher, LruEvictionManager, MemoryManager,
};
