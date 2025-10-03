# Development Guide

This guide covers the development workflow, architecture decisions, and contribution guidelines for CodeCortXMCP.

## 🚀 Quick Start

### Prerequisites
- **Rust 1.70+** with Cargo
- **Git** for version control
- **Tree-sitter CLI** (optional, for query development)

### Development Setup
```bash
# Clone the repository
git clone https://github.com/kensave/codecortex-mcp.git
cd codecortext-mcp

# Build in development mode
cargo build

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run
```

## 🏗️ Project Structure

```
codecortx-mcp/
├── src/
│   ├── main.rs                 # Entry point
│   ├── lib.rs                  # Library exports
│   ├── indexing/               # Code parsing and indexing
│   │   ├── indexer.rs          # Tree-sitter symbol extraction
│   │   └── indexing_pipeline.rs # Orchestration and caching
│   ├── storage/                # Data storage and retrieval
│   │   ├── store.rs            # Core SymbolStore implementation
│   │   └── cache.rs            # Binary persistence layer
│   ├── search/                 # Full-text search capabilities
│   │   └── bm25_index.rs       # BM25 algorithm implementation
│   ├── languages/              # Language support and models
│   │   └── models.rs           # Core data structures
│   ├── mcp/                    # MCP protocol implementation
│   │   ├── tools.rs            # Main MCP server and tool routing
│   │   └── outline_tools.rs    # File/directory outline tools
│   └── utils/                  # Utility modules
│       ├── filesystem.rs       # File system operations
│       ├── memory.rs           # Memory management
│       ├── lru.rs              # LRU eviction logic
│       ├── watcher.rs          # File change detection
│       └── error.rs            # Error handling
├── queries/                    # Tree-sitter query files
│   ├── rust-symbols.scm        # Rust symbol extraction queries
│   ├── python-symbols.scm      # Python symbol extraction queries
│   └── ...                     # Other language queries
├── tests/                      # Integration and performance tests
├── benches/                    # Criterion benchmarks
└── samples/                    # Test code samples for all languages
```

## 🔧 Development Workflow

### Building and Testing
```bash
# Development build (faster compilation, debug symbols)
cargo build

# Release build (optimized, production ready)
cargo build --release

# Run all tests
cargo test

# Run specific test module
cargo test storage::store

# Run tests with output
cargo test -- --nocapture

# Run benchmarks
cargo bench
```

### Code Quality
```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Check for unused dependencies
cargo machete

# Security audit
cargo audit
```

### Performance Profiling
```bash
# Run performance tests
cargo test --test performance_validation -- --nocapture

# Profile with perf (Linux)
perf record --call-graph=dwarf cargo test --release
perf report

# Memory profiling with valgrind
valgrind --tool=massif cargo test --release
```

## 🧪 Testing Strategy

### Test Categories

1. **Unit Tests** (`src/**/*.rs`)
   - Test individual functions and modules
   - Mock external dependencies
   - Fast execution (<100ms total)

2. **Integration Tests** (`tests/*.rs`)
   - Test complete workflows
   - Use real file system and parsing
   - Validate MCP protocol compliance

3. **Performance Tests** (`tests/performance_validation.rs`)
   - Validate performance requirements
   - Measure latency and throughput
   - Memory usage validation

4. **Benchmarks** (`benches/*.rs`)
   - Detailed performance measurements
   - Regression detection
   - Optimization guidance

### Writing Tests

```rust
// Unit test example
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_insertion() {
        let store = SymbolStore::new();
        let symbol = create_test_symbol();
        
        store.insert_symbol(symbol.clone());
        
        let retrieved = store.get_symbol(&symbol.id).unwrap();
        assert_eq!(retrieved.name, symbol.name);
    }
}

// Integration test example
#[tokio::test]
async fn test_end_to_end_indexing() {
    let temp_dir = TempDir::new().unwrap();
    create_test_files(&temp_dir);
    
    let store = Arc::new(SymbolStore::new());
    let mut pipeline = IndexingPipeline::new(store.clone()).unwrap();
    
    let result = pipeline.index_directory(&temp_dir).await;
    assert!(result.is_ok());
    
    // Validate symbols were extracted
    let symbols = store.get_symbols("test_function");
    assert!(!symbols.is_empty());
}
```

## 🔍 Adding Language Support

### 1. Add Tree-sitter Dependency
```toml
# Cargo.toml
[dependencies]
tree-sitter-newlang = "0.x.x"
```

### 2. Create Query Files
```scheme
; queries/newlang-symbols.scm
(function_declaration
  name: (identifier) @function.name) @function.definition

(class_declaration
  name: (identifier) @class.name) @class.definition
```

### 3. Update Language Enum
```rust
// src/languages/models.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Language {
    Rust,
    Python,
    NewLang, // Add here
    // ...
}

impl Language {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "rs" => Some(Language::Rust),
            "py" => Some(Language::Python),
            "newext" => Some(Language::NewLang), // Add here
            // ...
        }
    }
}
```

### 4. Add Parser Integration
```rust
// src/indexing/indexer.rs
impl SymbolIndexer {
    fn get_parser_and_query(&self, language: Language) -> Result<(Parser, Query), IndexingError> {
        let mut parser = Parser::new();
        let (lang, query_source) = match language {
            Language::Rust => (tree_sitter_rust::language(), include_str!("../../queries/rust-symbols.scm")),
            Language::Python => (tree_sitter_python::language(), include_str!("../../queries/python-symbols.scm")),
            Language::NewLang => (tree_sitter_newlang::language(), include_str!("../../queries/newlang-symbols.scm")),
            // ...
        };
        
        parser.set_language(lang)?;
        let query = Query::new(lang, query_source)?;
        Ok((parser, query))
    }
}
```

### 5. Add Tests
```rust
// tests/individual_language_tests.rs
#[tokio::test]
async fn test_newlang_integration() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.newext");
    
    fs::write(&test_file, r#"
        function testFunction() {
            return 42;
        }
        
        class TestClass {
            method testMethod() {}
        }
    "#).unwrap();

    let store = Arc::new(SymbolStore::new());
    let mut pipeline = IndexingPipeline::new(store.clone()).unwrap();
    pipeline.index_directory(temp_dir.path()).await.unwrap();

    // Validate function extraction
    let functions = store.get_symbols("testFunction");
    assert!(!functions.is_empty());
    assert_eq!(functions[0].symbol_type, SymbolType::Function);

    // Validate class extraction
    let classes = store.get_symbols("TestClass");
    assert!(!classes.is_empty());
    assert_eq!(classes[0].symbol_type, SymbolType::Class);
}
```

## 🚀 Performance Optimization

### Memory Management
- Use `Arc<T>` for shared immutable data
- Prefer `DashMap` over `Mutex<HashMap>` for concurrent access
- Implement LRU eviction for memory-bounded operations
- Monitor memory usage with `AtomicU64` counters

### Concurrency
- Use async/await for I/O bound operations
- Leverage Tokio's work-stealing scheduler
- Avoid blocking operations in async contexts
- Use channels for producer-consumer patterns

### Parsing Optimization
- Cache parsed ASTs when possible
- Use incremental parsing for file updates
- Batch file operations to reduce syscalls
- Implement early termination for large files

### Storage Optimization
- Use binary serialization (bincode) for persistence
- Implement compression for large symbol tables
- Use memory-mapped files for large datasets
- Optimize hash functions for symbol IDs

## 🔧 MCP Tool Development

### Creating New Tools

1. **Define Tool Schema**
```rust
Tool {
    name: "my_new_tool".into(),
    description: Some("Description of what the tool does".into()),
    input_schema: Arc::new(serde_json::from_value(json!({
        "type": "object",
        "properties": {
            "param1": {
                "type": "string",
                "description": "Parameter description"
            }
        },
        "required": ["param1"]
    })).unwrap()),
    output_schema: None,
    annotations: None,
    icons: None,
    title: None,
}
```

2. **Implement Tool Logic**
```rust
pub async fn my_new_tool(arguments: Option<Map<String, Value>>) -> Result<CallToolResult, ErrorData> {
    let args = arguments.ok_or_else(|| ErrorData::new(ErrorCode::INVALID_PARAMS, "Missing arguments", None))?;
    
    let param1 = args.get("param1")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::new(ErrorCode::INVALID_PARAMS, "Missing param1", None))?;

    // Tool implementation here
    let result = process_request(param1).await?;
    
    Ok(CallToolResult::success(vec![Content::text(result)]))
}
```

3. **Add to Tool Router**
```rust
// In call_tool method
match request.name.as_ref() {
    "index_code" => self.index_code(request.arguments).await,
    "my_new_tool" => self.my_new_tool(request.arguments).await,
    // ...
}
```

### Tool Best Practices
- Validate all input parameters
- Return structured error messages
- Use streaming for large responses
- Implement timeout handling
- Add comprehensive logging

## 🐛 Debugging

### Logging Configuration
```bash
# Enable all debug logs
RUST_LOG=debug cargo run

# Enable specific module logs
RUST_LOG=codecortext_mcp::indexing=trace cargo run

# Enable MCP protocol logs
RUST_LOG=rmcp=debug cargo run
```

### Common Debug Scenarios

1. **Symbol Not Found**
   - Check language detection
   - Verify query syntax
   - Validate file parsing

2. **Performance Issues**
   - Profile with `cargo bench`
   - Check memory usage
   - Analyze concurrent access patterns

3. **MCP Protocol Issues**
   - Use MCP Inspector for testing
   - Validate JSON-RPC messages
   - Check tool schema compliance

### Debug Tools
```bash
# Interactive debugging with GDB
rust-gdb target/debug/codecortext-mcp

# Memory debugging with Valgrind
valgrind --leak-check=full cargo test

# Profiling with perf
perf record cargo bench
perf report
```

## 📦 Release Process

### Version Management
```bash
# Update version in Cargo.toml
# Update CHANGELOG.md
# Create git tag
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

### Release Checklist
- [ ] All tests passing
- [ ] Performance benchmarks validated
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version bumped in Cargo.toml
- [ ] Security audit passed
- [ ] Binary size optimized

### Distribution
```bash
# Build optimized release
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-pc-windows-gnu
cargo build --release --target x86_64-apple-darwin

# Strip debug symbols
strip target/release/codecortext-mcp
```

## 🤝 Contributing Guidelines

### Code Style
- Follow Rust standard formatting (`cargo fmt`)
- Use meaningful variable names
- Add documentation for public APIs
- Keep functions focused and small

### Commit Messages
```
feat: add support for TypeScript parsing
fix: resolve memory leak in symbol storage
docs: update installation instructions
test: add integration test for Python indexing
perf: optimize symbol lookup performance
```

### Pull Request Process
1. Fork the repository
2. Create feature branch from `main`
3. Implement changes with tests
4. Run full test suite
5. Update documentation
6. Submit pull request with description

### Code Review Checklist
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] Performance impact considered
- [ ] Error handling implemented
- [ ] Security implications reviewed
- [ ] Backward compatibility maintained

---

This development guide should help you contribute effectively to CodeCortXMCP. For questions, please open an issue or start a discussion.
