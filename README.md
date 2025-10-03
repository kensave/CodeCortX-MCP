# CodeCortXMCP Server

A lightning-fast, language-agnostic code analysis MCP (Model Context Protocol) server built in Rust. Provides instant symbol lookups, reference tracking, and semantic code search for large codebases with performance as a first-class citizen.

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-83%20passing-green.svg)](#testing)

## 🚀 Features

- **⚡ High Performance**: <1ms symbol lookups, >100 files/sec indexing
- **🔒 Lock-free Concurrency**: No blocking operations, handles concurrent requests efficiently  
- **🧠 Smart Caching**: Binary persistence with <1s startup for previously indexed repositories
- **📊 Memory Management**: Automatic LRU eviction with configurable memory limits
- **🔄 Incremental Updates**: File watching with SHA-256 change detection
- **🌍 Multi-language**: 15+ languages supported with extensible architecture
- **🛡️ Error Resilient**: Graceful handling of malformed code and I/O errors
- **🔍 Full-text Search**: BM25 statistical search through all code content

## 🏗️ Architecture

- **Language**: Rust (performance + safety)
- **Parser**: Tree-sitter (consistent, incremental parsing)
- **Storage**: In-memory DashMap + binary persistence
- **Concurrency**: Lock-free data structures
- **Protocol**: MCP over JSON-RPC stdio

## 📋 MCP Tools

The server provides 7 MCP tools for comprehensive code analysis:

### 1. `index_code`
Index source code files to build symbol table for fast lookups.
```json
{
  "path": "/path/to/project"
}
```

### 2. `get_symbol`
Retrieve symbol information by name with optional source code inclusion.
```json
{
  "name": "function_name",
  "include_source": true
}
```

### 3. `get_symbol_references`
Find all references to a symbol across the codebase.
```json
{
  "name": "symbol_name"
}
```

### 4. `find_symbols`
Search symbols by query using exact match or fuzzy search with optional type filtering.
```json
{
  "query": "test_",
  "symbol_type": "function"
}
```

### 5. `code_search` 🎯
**BM25 statistical search through all indexed code content.**
```json
{
  "query": "fibonacci algorithm",
  "max_results": 10
}
```

**Perfect for finding:**
- Algorithm implementations: `"binary search algorithm"`
- Error handling patterns: `"error handling try catch"`
- Database code: `"database connection pool"`
- Specific functionality: `"file upload validation"`

### 6. `get_file_outline` 📄
**Get structured outline of symbols in a specific file.**
```json
{
  "file_path": "/path/to/file.rs"
}
```

**Returns organized view of:**
- Classes/Structs with signatures
- Functions/Methods with full signatures and parameters
- Constants, Enums, Interfaces, Modules, Imports, Variables
- Line numbers and visibility (pub/priv)

### 7. `get_directory_outline` 📁
**Get high-level overview of symbols across a directory.**
```json
{
  "directory_path": "/path/to/project",
  "includes": ["functions", "methods", "constants"]
}
```

**Perfect for:**
- Project structure understanding
- API surface discovery
- Architecture overview
- Code navigation

## 🛠️ Installation & Setup

### Prerequisites
- Rust 1.70+ with Cargo
- Git

### Building from Source
```bash
git clone https://github.com/kensave/codecortx-mcp.git
cd codecortx-mcp
cargo build --release
```

The binary will be available at `target/release/codecortx-mcp`.

## 🔧 Usage

### With Amazon Q CLI

1. **Add to Amazon Q CLI Configuration**

   Add the following to your Amazon Q CLI MCP configuration:

   ```json
   {
     "mcpServers": {
       "codecortx": {
         "command": "/path/to/codecortx-mcp/target/release/codecortx-mcp",
         "args": []
       }
     }
   }
   ```

2. **Restart Amazon Q CLI**

3. **Start Using**

   In Amazon Q CLI, you can now ask questions like:
   - "Index the code in my project directory"
   - "Find all functions that contain 'parse' in their name"
   - "Show me all references to the `SymbolStore` struct"
   - "Get the implementation of the `extract_symbols` function"
   - "Search for fibonacci algorithm implementations"
   - "Find error handling patterns in the codebase"
   - "Show me the outline of this file with all functions and their signatures"
   - "Get an overview of all classes and methods in this directory"

### Testing with MCP Inspector

MCP Inspector is a great tool for testing and debugging MCP servers.

1. **Install MCP Inspector**
   ```bash
   npx @modelcontextprotocol/inspector
   ```

2. **Test the Server**
   ```bash
   # Run the server
   ./target/release/codecortx-mcp

   # In another terminal, run MCP Inspector
   npx @modelcontextprotocol/inspector ./target/release/codecortx-mcp
   ```

3. **Explore the Tools**
   - View available tools and their schemas
   - Test tool calls with sample data
   - Inspect request/response cycles
   - Debug any integration issues

### Manual Testing via Command Line

You can also test the server manually using stdio:

```bash
# Start the server
./target/release/codecortx-mcp

# Send MCP initialization (paste this JSON)
{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test-client", "version": "1.0.0"}}}

# Send initialized notification
{"jsonrpc": "2.0", "method": "notifications/initialized"}

# List available tools
{"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}

# Index a directory
{"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "index_code", "arguments": {"path": "/path/to/your/project"}}}

# Search for symbols
{"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {"name": "find_symbols", "arguments": {"query": "main", "symbol_type": "function"}}}

# Search code content with BM25
{"jsonrpc": "2.0", "id": 5, "method": "tools/call", "params": {"name": "code_search", "arguments": {"query": "error handling", "max_results": 5}}}

# Get file outline with signatures
{"jsonrpc": "2.0", "id": 6, "method": "tools/call", "params": {"name": "get_file_outline", "arguments": {"file_path": "/path/to/file.rs"}}}

# Get directory overview
{"jsonrpc": "2.0", "id": 7, "method": "tools/call", "params": {"name": "get_directory_outline", "arguments": {"directory_path": "/path/to/project", "includes": ["functions", "classes"]}}}
```

## ⚡ Performance Benchmarks

Run the included benchmarks to validate performance on your system:

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench -- symbol_lookup

# Run performance validation tests
cargo test --test performance_validation -- --nocapture
```

**Expected Performance Targets:**
- Symbol lookups: <1ms average
- Indexing speed: >100 files/second
- Concurrent access: >50k lookups/second
- Memory usage: <1GB for large repositories

## 🧪 Testing

The project includes comprehensive test coverage:

```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --lib

# Run integration tests
cargo test --test integration_test

# Run performance validation
cargo test --test performance_validation

# Run with output for debugging
cargo test -- --nocapture
```

**Test Coverage:**
- 54 unit tests covering all core modules
- 5 integration tests for end-to-end workflows
- 5 performance tests validating requirements
- 15 language-specific tests
- 4 outline tool tests

**Total: 83 tests passing**

## 🔍 Supported Languages

Currently supports 15+ languages:
- **Rust** (.rs): Functions, structs, enums, traits, implementations, constants, modules
- **Python** (.py): Functions, classes, methods, variables, imports
- **JavaScript** (.js): Functions, classes, methods, constants, variables
- **TypeScript** (.ts): Functions, classes, interfaces, types, enums
- **Java** (.java): Classes, methods, interfaces, enums, constants
- **Go** (.go): Functions, structs, interfaces, constants, variables
- **C** (.c): Functions, structs, enums, typedefs, variables
- **C++** (.cpp, .hpp): Classes, functions, namespaces, templates
- **Ruby** (.rb): Classes, modules, methods, constants
- **PHP** (.php): Classes, functions, methods, constants
- **C#** (.cs): Classes, methods, interfaces, enums, properties
- **Kotlin** (.kt): Classes, functions, interfaces, objects
- **Scala** (.scala): Classes, objects, traits, functions
- **Swift** (.swift): Classes, structs, protocols, functions
- **Objective-C** (.m, .h): Classes, methods, protocols, categories

**Adding New Languages:**
The architecture is designed for easy extension. To add a new language:
1. Add Tree-sitter grammar dependency
2. Create query files in `queries/` directory
3. Update `Language` enum and language detection
4. Add to supported extensions

## 💾 Caching & Persistence

- **Cache Location**: Uses system cache directory (`~/.cache/codecortext-mcp/` on Unix)
- **Cache Format**: Custom binary format with bincode serialization
- **Cache Key**: Based on repository path and last modification times
- **Cache Validation**: Automatic validation on startup with incremental updates
- **Memory Management**: LRU eviction when memory pressure detected (configurable)

## 🛡️ Error Handling

The server is designed for robustness:
- **Parse Errors**: Continues indexing other files, logs issues
- **File System Errors**: Graceful degradation with partial results
- **Memory Pressure**: Automatic cleanup and eviction
- **Malformed Requests**: Proper MCP error responses
- **Concurrent Access**: Lock-free structures prevent deadlocks

## 📊 Monitoring & Logging

The server uses structured logging with different levels:

```bash
# Enable debug logging
RUST_LOG=debug ./target/release/codecortx-mcp

# Enable trace logging for specific modules
RUST_LOG=codecortx_mcp::indexer=trace ./target/release/codecortx-mcp
```

## ⚙️ Configuration

### Environment Variables
```bash
# Memory management
export CODECORTEXT_MAX_MEMORY_MB=1024
export CODECORTEXT_EVICTION_THRESHOLD=0.8

# Cache location
export CODECORTX_CACHE_DIR=~/.cache/codecortx-mcp

# Logging
export RUST_LOG=codecortx_mcp=info
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Run the test suite (`cargo test`)
4. Run benchmarks to ensure no performance regression (`cargo bench`)
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## 📝 License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## 🔧 Troubleshooting

### Common Issues

1. **"Symbol not found" errors during compilation**
   - Ensure you have the latest Rust toolchain: `rustup update`
   - Clean and rebuild: `cargo clean && cargo build`

2. **Server not responding in Amazon Q CLI**
   - Check the config file path and syntax
   - Verify the binary path is correct and executable
   - Check Amazon Q CLI logs for error messages

3. **High memory usage**
   - Configure memory limits via environment variables
   - The server will automatically evict least-recently-used files
   - Consider indexing smaller subdirectories for very large repositories

4. **Slow indexing performance**
   - Check disk I/O performance
   - Ensure no antivirus is scanning files during indexing
   - Use SSD storage for better performance

### Debug Commands

```bash
# Check server version and capabilities
./target/release/codecortx-mcp --version

# Test basic functionality
cargo test --test integration_test -- test_end_to_end_rust_indexing

# Benchmark performance
cargo test --test performance_validation -- --nocapture
```

## 📚 Documentation

- [Architecture Guide](ARCHITECTURE.md) - Detailed system architecture
- [Development Guide](DEVELOPMENT.md) - Setup and development workflow
- [API Reference](docs/api.md) - Complete MCP tool documentation

---

**Built with ❤️ in Rust for lightning-fast code analysis**
