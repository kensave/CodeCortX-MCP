# CodeCortXMCP API Reference

This document provides comprehensive API documentation for all MCP tools provided by CodeCortXMCP.

## 🔧 MCP Tools Overview

CodeCortXMCP provides 7 MCP tools for comprehensive code analysis:

| Tool | Purpose | Performance |
|------|---------|-------------|
| `index_code` | Index source code directory | >100 files/sec |
| `get_symbol` | Retrieve symbol information | <1ms lookup |
| `get_symbol_references` | Find all symbol references | <5ms search |
| `find_symbols` | Search symbols with filtering | <10ms query |
| `code_search` | BM25 full-text search | <50ms search |
| `get_file_outline` | File structure overview | <20ms analysis |
| `get_directory_outline` | Directory structure overview | <100ms scan |

## 📋 Tool Specifications

### 1. index_code

**Purpose**: Index source code files to build symbol table for fast lookups.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "path": {
      "type": "string",
      "description": "Path to directory or file to index"
    }
  },
  "required": ["path"]
}
```

**Example Request**:
```json
{
  "name": "index_code",
  "arguments": {
    "path": "/path/to/project"
  }
}
```

**Example Response**:
```json
{
  "content": [{
    "type": "text",
    "text": "{\n  \"status\": \"success\",\n  \"files_indexed\": 42,\n  \"symbols_found\": 1337,\n  \"errors\": [],\n  \"duration_ms\": 250\n}"
  }]
}
```

**Response Fields**:
- `status`: "success" | "partial" | "error"
- `files_indexed`: Number of files successfully processed
- `symbols_found`: Total symbols extracted
- `errors`: Array of error messages for failed files
- `duration_ms`: Processing time in milliseconds

**Error Conditions**:
- Invalid path: Returns error with message
- Permission denied: Continues with accessible files
- Parse errors: Logs errors, continues processing

---

### 2. get_symbol

**Purpose**: Retrieve detailed information about a specific symbol.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "name": {
      "type": "string",
      "description": "Symbol name to search for"
    },
    "include_source": {
      "type": "boolean",
      "description": "Include source code in response",
      "default": false
    }
  },
  "required": ["name"]
}
```

**Example Request**:
```json
{
  "name": "get_symbol",
  "arguments": {
    "name": "SymbolStore",
    "include_source": true
  }
}
```

**Example Response**:
```json
{
  "content": [{
    "type": "text",
    "text": "{\n  \"symbols\": [\n    {\n      \"id\": 12345,\n      \"name\": \"SymbolStore\",\n      \"symbol_type\": \"Struct\",\n      \"location\": {\n        \"file\": \"/path/to/store.rs\",\n        \"start_line\": 12,\n        \"start_column\": 0,\n        \"end_line\": 21,\n        \"end_column\": 1\n      },\n      \"namespace\": null,\n      \"visibility\": \"Public\",\n      \"source\": \"pub struct SymbolStore {\\n    // fields...\\n}\"\n    }\n  ]\n}"
  }]
}
```

**Symbol Object Fields**:
- `id`: Unique symbol identifier
- `name`: Symbol name
- `symbol_type`: Function | Class | Struct | Enum | Interface | Constant | Variable | Module | Import
- `location`: File path and position information
- `namespace`: Optional namespace/module path
- `visibility`: Public | Private
- `source`: Optional source code (if `include_source: true`)

---

### 3. get_symbol_references

**Purpose**: Find all references to a symbol across the codebase.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "name": {
      "type": "string",
      "description": "Symbol name to find references for"
    }
  },
  "required": ["name"]
}
```

**Example Request**:
```json
{
  "name": "get_symbol_references",
  "arguments": {
    "name": "SymbolStore"
  }
}
```

**Example Response**:
```json
{
  "content": [{
    "type": "text",
    "text": "{\n  \"references\": [\n    {\n      \"location\": {\n        \"file\": \"/path/to/main.rs\",\n        \"start_line\": 15,\n        \"start_column\": 8,\n        \"end_line\": 15,\n        \"end_column\": 19\n      },\n      \"reference_type\": \"Usage\"\n    }\n  ],\n  \"total_count\": 23\n}"
  }]
}
```

**Reference Object Fields**:
- `location`: File path and position
- `reference_type`: Definition | Usage | Import | Declaration

---

### 4. find_symbols

**Purpose**: Search for symbols using exact match or fuzzy search with optional type filtering.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "query": {
      "type": "string",
      "description": "Search query (supports exact match and prefix matching)"
    },
    "symbol_type": {
      "type": "string",
      "description": "Optional symbol type filter",
      "enum": ["function", "method", "class", "struct", "enum", "interface", "constant", "variable", "module", "import"]
    }
  },
  "required": ["query"]
}
```

**Example Request**:
```json
{
  "name": "find_symbols",
  "arguments": {
    "query": "test_",
    "symbol_type": "function"
  }
}
```

**Example Response**:
```json
{
  "content": [{
    "type": "text",
    "text": "{\n  \"symbols\": [\n    {\n      \"id\": 67890,\n      \"name\": \"test_function\",\n      \"symbol_type\": \"Function\",\n      \"location\": {\n        \"file\": \"/path/to/test.rs\",\n        \"start_line\": 42,\n        \"start_column\": 0,\n        \"end_line\": 45,\n        \"end_column\": 1\n      },\n      \"namespace\": null,\n      \"visibility\": \"Public\"\n    }\n  ]\n}"
  }]
}
```

**Search Behavior**:
- Exact match: `"main"` finds symbols named exactly "main"
- Prefix match: `"test_"` finds symbols starting with "test_"
- Case insensitive matching
- Results sorted by relevance

---

### 5. code_search

**Purpose**: BM25 statistical search through all indexed code content.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "query": {
      "type": "string",
      "description": "Search query for code content"
    },
    "max_results": {
      "type": "integer",
      "description": "Maximum number of results to return",
      "default": 10,
      "minimum": 1,
      "maximum": 100
    },
    "context_lines": {
      "type": "integer",
      "description": "Number of lines of context around matches",
      "default": 2,
      "minimum": 0,
      "maximum": 10
    }
  },
  "required": ["query"]
}
```

**Example Request**:
```json
{
  "name": "code_search",
  "arguments": {
    "query": "fibonacci algorithm",
    "max_results": 5,
    "context_lines": 3
  }
}
```

**Example Response**:
```json
{
  "content": [{
    "type": "text",
    "text": "{\n  \"results\": [\n    {\n      \"score\": 4.2,\n      \"file_path\": \"/path/to/math.rs\",\n      \"language\": \"rust\",\n      \"content_snippet\": \"// Calculate fibonacci number\\nfn fibonacci(n: u32) -> u32 {\\n    match n {\\n        0 => 0,\\n        1 => 1,\\n        _ => fibonacci(n-1) + fibonacci(n-2)\\n    }\\n}\"\n    }\n  ],\n  \"total_found\": 3\n}"
  }]
}
```

**Search Result Fields**:
- `score`: BM25 relevance score (higher = more relevant)
- `file_path`: Path to file containing match
- `language`: Detected programming language
- `content_snippet`: Code snippet with context lines
- `total_found`: Total number of matches found

**Search Tips**:
- Use specific terms: `"error handling"` vs `"error"`
- Combine concepts: `"database connection pool"`
- Algorithm names: `"binary search"`, `"quicksort"`
- Pattern matching: `"try catch finally"`

---

### 6. get_file_outline

**Purpose**: Get structured outline of symbols in a specific file.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "file_path": {
      "type": "string",
      "description": "Path to the file to analyze"
    }
  },
  "required": ["file_path"]
}
```

**Example Request**:
```json
{
  "name": "get_file_outline",
  "arguments": {
    "file_path": "/path/to/src/main.rs"
  }
}
```

**Example Response**:
```json
{
  "content": [{
    "type": "text",
    "text": "File: /path/to/src/main.rs\n\n📁 MODULES\n  • std (line 1)\n\n🏗️ STRUCTS\n  • Config (line 5-12)\n    - pub host: String\n    - pub port: u16\n\n⚙️ FUNCTIONS\n  • main() (line 15-25)\n  • parse_args(args: Vec<String>) -> Config (line 27-35)\n\n📊 CONSTANTS\n  • DEFAULT_PORT: u16 = 8080 (line 3)"
  }]
}
```

**Outline Sections**:
- **MODULES**: Imported modules and dependencies
- **STRUCTS/CLASSES**: Data structures with fields
- **FUNCTIONS/METHODS**: Functions with signatures
- **CONSTANTS**: Constant values
- **ENUMS**: Enumeration types
- **INTERFACES/TRAITS**: Abstract interfaces

**Features**:
- Shows line numbers for navigation
- Displays function signatures with parameters
- Indicates visibility (pub/private)
- Groups related symbols together

---

### 7. get_directory_outline

**Purpose**: Get high-level overview of symbols across a directory.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "directory_path": {
      "type": "string",
      "description": "Path to the directory to analyze"
    },
    "includes": {
      "type": "array",
      "items": {"type": "string"},
      "description": "Optional symbol types to include",
      "default": ["classes", "structs", "interfaces"]
    }
  },
  "required": ["directory_path"]
}
```

**Example Request**:
```json
{
  "name": "get_directory_outline",
  "arguments": {
    "directory_path": "/path/to/src",
    "includes": ["functions", "classes", "constants"]
  }
}
```

**Example Response**:
```json
{
  "content": [{
    "type": "text",
    "text": "Directory: /path/to/src (5 files)\n\n📁 main.rs\n  🏗️ Config\n  ⚙️ main()\n  ⚙️ parse_args()\n\n📁 server.rs\n  🏗️ Server\n  ⚙️ start()\n  ⚙️ handle_request()\n\n📁 utils.rs\n  📊 MAX_CONNECTIONS\n  ⚙️ validate_input()\n\nSummary: 2 structs, 5 functions, 1 constant"
  }]
}
```

**Include Options**:
- `"functions"`: Function and method definitions
- `"methods"`: Class/struct methods only
- `"classes"`: Class definitions
- `"structs"`: Struct definitions
- `"interfaces"`: Interface/trait definitions
- `"constants"`: Constant values
- `"enums"`: Enumeration types
- `"modules"`: Module definitions

**Default Behavior**:
- Shows classes, structs, and interfaces by default
- Provides file-by-file breakdown
- Includes summary statistics
- Filters out test files and generated code

## 🚨 Error Handling

### Common Error Codes

| Code | Description | Resolution |
|------|-------------|------------|
| `INVALID_PARAMS` | Missing or invalid parameters | Check required fields |
| `INTERNAL_ERROR` | Server-side processing error | Retry request |
| `METHOD_NOT_FOUND` | Unknown tool name | Check tool name spelling |
| `PARSE_ERROR` | File parsing failed | Check file syntax |
| `FILE_NOT_FOUND` | Specified file doesn't exist | Verify file path |

### Error Response Format
```json
{
  "error": {
    "code": "INVALID_PARAMS",
    "message": "Missing required parameter: path",
    "data": null
  }
}
```

### Handling Partial Failures
- Indexing continues on parse errors
- Search returns partial results if some files fail
- Outline tools skip inaccessible files
- Error details included in response when possible

## 📊 Performance Characteristics

### Response Times (Typical)
- `index_code`: 100-500ms per 100 files
- `get_symbol`: <1ms for cached symbols
- `get_symbol_references`: <5ms for typical projects
- `find_symbols`: <10ms for prefix searches
- `code_search`: 10-50ms depending on query complexity
- `get_file_outline`: <20ms for typical files
- `get_directory_outline`: <100ms for typical directories

### Memory Usage
- Base server: ~50MB
- Per 1000 symbols: ~10MB
- BM25 index: ~20% of source code size
- Cache overhead: ~30% of symbol data

### Scalability Limits
- Files: Tested up to 10,000 files
- Symbols: Tested up to 100,000 symbols
- Concurrent requests: >1000/second
- Memory limit: Configurable with LRU eviction

## 🔧 Configuration

### Environment Variables
```bash
# Memory management
CODECORTEXT_MAX_MEMORY_MB=1024
CODECORTEXT_EVICTION_THRESHOLD=0.8

# Performance tuning
CODECORTEXT_INDEX_BATCH_SIZE=100
CODECORTEXT_SEARCH_TIMEOUT_MS=5000

# Logging
RUST_LOG=codecortext_mcp=info
```

### Cache Behavior
- Automatic cache invalidation on file changes
- Binary serialization for fast startup
- LRU eviction when memory limits reached
- Cache location: `~/.cache/codecortext-mcp/`

---

This API reference provides complete documentation for integrating with CodeCortXMCP. For implementation examples, see the main README.md file.
