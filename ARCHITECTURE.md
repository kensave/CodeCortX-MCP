# CodeCortXMCP Architecture

## 🏗️ System Overview

CodeCortXMCP is a high-performance, language-agnostic code analysis MCP server built in Rust. It provides instant symbol lookups, reference tracking, and semantic code search across large codebases.

### Core Design Principles
- **Performance First**: <1ms symbol lookups, >100 files/sec indexing
- **Lock-free Concurrency**: No blocking operations using DashMap
- **Memory Efficient**: LRU eviction with configurable limits
- **Language Agnostic**: Tree-sitter based parsing for 15+ languages
- **MCP Native**: Built specifically for Model Context Protocol

## 📊 Architecture Diagram

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   MCP Client    │    │  Tree-sitter    │    │   File System   │
│  (Amazon Q)     │    │   Parsers       │    │    Watcher      │
└─────────┬───────┘    └─────────┬───────┘    └─────────┬───────┘
          │                      │                      │
          │ JSON-RPC             │ AST                  │ Events
          │                      │                      │
┌─────────▼──────────────────────▼──────────────────────▼───────┐
│                     MCP Tools Layer                           │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐             │
│  │ Code Search │ │   Outline   │ │  Analysis   │             │
│  │    Tools    │ │    Tools    │ │    Tools    │             │
│  └─────────────┘ └─────────────┘ └─────────────┘             │
└─────────────────────────┬─────────────────────────────────────┘
                          │
┌─────────────────────────▼─────────────────────────────────────┐
│                  Core Engine Layer                            │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐             │
│  │  Indexing   │ │   Storage   │ │   Search    │             │
│  │  Pipeline   │ │   Engine    │ │   Engine    │             │
│  └─────────────┘ └─────────────┘ └─────────────┘             │
└───────────────────────────────────────────────────────────────┘
```

## 🔧 Core Components

### 1. Storage Engine (`src/storage/`)

**SymbolStore** - Central data structure using lock-free concurrent maps:
```rust
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
```

**Key Features:**
- **Lock-free**: DashMap provides concurrent access without locks
- **Memory Management**: Automatic LRU eviction when memory pressure detected
- **Persistence**: Binary cache with bincode serialization
- **Change Detection**: SHA-256 based incremental updates

### 2. Indexing Pipeline (`src/indexing/`)

**Multi-stage Processing:**
1. **File Discovery**: Recursive directory traversal with language detection
2. **Parsing**: Tree-sitter AST generation per language
3. **Symbol Extraction**: Query-based symbol identification
4. **Reference Tracking**: Cross-file reference resolution
5. **Storage**: Atomic insertion into SymbolStore

**Performance Characteristics:**
- **Parallel Processing**: Tokio async for I/O bound operations
- **Incremental Updates**: Only reprocess changed files
- **Error Resilience**: Continue processing on parse failures
- **Memory Bounded**: Configurable memory limits with eviction

### 3. Search Engine (`src/search/`)

**BM25 Statistical Search:**
- **Full-text Indexing**: All code content indexed for semantic search
- **Relevance Scoring**: BM25 algorithm for ranking results
- **Context Extraction**: Surrounding lines for search results
- **Language Aware**: Syntax highlighting and structure preservation

### 4. Language Support (`src/languages/`)

**Currently Supported (15 languages):**
- Rust, Python, JavaScript, TypeScript, Java, Go, C, C++
- Ruby, PHP, C#, Kotlin, Scala, Swift, Objective-C

**Symbol Types Extracted:**
- Functions, Methods, Classes, Structs, Enums, Interfaces
- Constants, Variables, Modules, Imports, Traits

**Query-based Extraction:**
```scheme
; Example Rust queries
(function_item name: (identifier) @function.name) @function.definition
(struct_item name: (type_identifier) @struct.name) @struct.definition
(const_item name: (identifier) @const.name) @const.definition
```

## 🚀 Performance Architecture

### Memory Management
- **Configurable Limits**: Environment variable controlled
- **LRU Eviction**: Least recently used files removed first
- **Memory Tracking**: Real-time usage monitoring
- **Atomic Operations**: Lock-free memory counters

### Concurrency Model
- **Lock-free Data Structures**: DashMap for all shared state
- **Async I/O**: Tokio runtime for file operations
- **Parallel Indexing**: Multiple files processed concurrently
- **Non-blocking Queries**: Read operations never block

### Caching Strategy
- **Binary Persistence**: Fast startup with cached indexes
- **Incremental Updates**: Only changed files reprocessed
- **Cache Validation**: Automatic invalidation on file changes
- **Compression**: Efficient storage with bincode

## 🔌 MCP Integration

### Tool Architecture
```rust
pub trait MCPTool {
    async fn call(arguments: Option<Map<String, Value>>) -> Result<CallToolResult, ErrorData>;
}
```

**Available Tools (7):**
1. `index_code` - Index source code directory
2. `get_symbol` - Retrieve symbol information
3. `get_symbol_references` - Find all symbol references
4. `find_symbols` - Search symbols with filtering
5. `code_search` - BM25 full-text search
6. `get_file_outline` - Structured file overview
7. `get_directory_outline` - Project structure navigation

### Error Handling
- **Graceful Degradation**: Partial results on errors
- **Structured Errors**: MCP-compliant error responses
- **Logging**: Structured logging with tracing
- **Recovery**: Automatic retry on transient failures

## 📈 Performance Targets

### Validated Requirements
- **Symbol Lookup**: <1ms average response time
- **Indexing Speed**: >100 files/second
- **Concurrent Access**: >50k lookups/second
- **Memory Usage**: <1GB for large repositories
- **Startup Time**: <1s with cached index

### Benchmarking
```bash
# Run performance validation
cargo test --test performance_validation -- --nocapture

# Run criterion benchmarks
cargo bench
```

## 🧪 Testing Strategy

### Test Coverage (83 tests total)
- **Unit Tests**: 54 tests covering all core modules
- **Integration Tests**: 5 end-to-end workflow tests
- **Performance Tests**: 5 performance requirement validations
- **Language Tests**: 15 language-specific integration tests
- **Tool Tests**: 4 MCP tool interface tests

### Test Categories
```bash
# Core functionality
cargo test --lib

# Integration workflows
cargo test --test integration_test

# Performance validation
cargo test --test performance_validation

# Language support
cargo test --test individual_language_tests
```

## 🔧 Configuration

### Environment Variables
```bash
# Memory management
CODECORTEXT_MAX_MEMORY_MB=1024
CODECORTEXT_EVICTION_THRESHOLD=0.8

# Logging
RUST_LOG=codecortx_mcp=info

# Cache location
CODECORTX_CACHE_DIR=~/.cache/codecortx-mcp
```

### Build Profiles
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true
overflow-checks = false
```

## 🚀 Deployment

### Binary Distribution
```bash
# Build optimized release
cargo build --release

# Binary location
./target/release/codecortx-mcp
```

### Amazon Q CLI Integration
```json
{
  "mcpServers": {
    "codecortx": {
      "command": "/path/to/codecortx-mcp",
      "args": []
    }
  }
}
```

## 🔮 Future Architecture Considerations

### Scalability
- **Distributed Storage**: Multi-node symbol storage
- **Streaming Updates**: Real-time file change processing
- **Query Optimization**: Advanced indexing strategies

### Language Support
- **Dynamic Loading**: Runtime language parser loading
- **Custom Queries**: User-defined symbol extraction
- **Semantic Analysis**: Type-aware symbol resolution

### Integration
- **LSP Support**: Language Server Protocol compatibility
- **IDE Plugins**: Direct editor integration
- **API Gateway**: REST/GraphQL query interface

---

This architecture provides a solid foundation for high-performance code analysis while maintaining simplicity and reliability.
