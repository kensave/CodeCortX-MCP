use crate::mcp::outline_tools::OutlineTools;
use crate::models::{Reference, Symbol, SymbolType};
use crate::utils::{FileWatcher, PathResolver};
use crate::{IndexingPipeline, SymbolStore};
use rmcp::{
    model::{
        CallToolRequestParam, CallToolResult, Content, ErrorCode, ErrorData, GetPromptRequestParam,
        GetPromptResult, ListPromptsResult, ListToolsResult, PaginatedRequestParam, Prompt,
        PromptArgument, PromptMessage, PromptMessageContent, PromptMessageRole, ServerCapabilities,
        ServerInfo, Tool,
    },
    service::{RequestContext, RoleServer},
    ServerHandler,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Instant;

// Global instances (initialized once per process)
pub static SYMBOL_STORE: OnceLock<Arc<SymbolStore>> = OnceLock::new();
static INDEXING_PIPELINE: OnceLock<Arc<tokio::sync::Mutex<IndexingPipeline>>> = OnceLock::new();
static FILE_WATCHERS: OnceLock<Arc<tokio::sync::Mutex<HashMap<PathBuf, Arc<FileWatcher>>>>> =
    OnceLock::new();

pub fn get_symbol_store() -> Arc<SymbolStore> {
    SYMBOL_STORE
        .get_or_init(|| Arc::new(SymbolStore::new()))
        .clone()
}

fn get_indexing_pipeline() -> Arc<tokio::sync::Mutex<IndexingPipeline>> {
    INDEXING_PIPELINE
        .get_or_init(|| {
            let store = get_symbol_store();
            let pipeline = IndexingPipeline::new(store.clone()).unwrap();
            Arc::new(tokio::sync::Mutex::new(pipeline))
        })
        .clone()
}

fn get_file_watchers() -> Arc<tokio::sync::Mutex<HashMap<PathBuf, Arc<FileWatcher>>>> {
    FILE_WATCHERS
        .get_or_init(|| Arc::new(tokio::sync::Mutex::new(HashMap::new())))
        .clone()
}

async fn start_file_watcher(path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let watchers = get_file_watchers();
    let mut watchers_guard = watchers.lock().await;

    // Don't create duplicate watchers for the same path
    if watchers_guard.contains_key(&path) {
        return Ok(());
    }

    let store = get_symbol_store();
    // Use the existing pipeline mutex instead of creating a new one
    let pipeline = get_indexing_pipeline();

    let watcher = FileWatcher::new(path.clone(), pipeline, store)?;
    watchers_guard.insert(path.clone(), Arc::new(watcher));

    tracing::info!("Started file watcher for path: {:?}", path);
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexCodeResponse {
    pub status: String,
    pub files_indexed: u32,
    pub symbols_found: u32,
    pub errors: Vec<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetSymbolResponse {
    pub symbols: Vec<Symbol>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetSymbolReferencesResponse {
    pub references: Vec<Reference>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FindSymbolsResponse {
    pub symbols: Vec<Symbol>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct IndexCodeRequest {
    /// Path to directory or file to index
    pub path: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetSymbolRequest {
    /// Name of the symbol to search for
    pub name: String,
    /// Include source code in the response
    #[serde(default)]
    pub include_source: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetSymbolReferencesRequest {
    /// Name of the symbol to find references for
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CodeSearchRequest {
    /// Search query for code content
    pub query: String,
    /// Maximum number of results to return
    pub limit: Option<u32>,
    /// Alias for limit - maximum number of results to return
    pub max_results: Option<u32>,
    /// Number of lines of context around matches
    pub context_lines: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeSearchResponse {
    pub results: Vec<CodeSearchResult>,
    pub total_found: usize,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct FindSymbolsRequest {
    /// Search query (supports exact match and prefix matching)
    pub query: String,
    /// Optional symbol type filter
    pub symbol_type: Option<String>,
    /// Maximum number of results to return (default: 10, max: 50)
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeSearchResult {
    pub score: f32,
    pub file_path: String,
    pub language: String,
    pub content_snippet: String,
}

#[derive(Clone, Debug, Default)]
pub struct CodeAnalysisTools;

impl CodeAnalysisTools {
    pub fn new() -> Self {
        Self
    }
}

impl ServerHandler for CodeAnalysisTools {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .build(),
            instructions: Some(
                "Roberto MCP server for analyzing source code symbols and references"
                    .to_string(),
            ),
            ..Default::default()
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let tools = vec![
            Tool {
                name: "index_code".into(),
                description: Some("REQUIRED FIRST - RUN ONCE: Index source code files in a directory to build symbol table for fast lookups. Must be run once before any other tools will work. File watching will remain active for the rest of the session.".into()),
                input_schema: Arc::new(serde_json::from_value(json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to directory or file to index"
                        }
                    },
                    "required": ["path"]
                })).unwrap()),
                output_schema: None,
                annotations: None,
                icons: None,
                title: None,
            },
            Tool {
                name: "get_symbol".into(),
                description: Some("Retrieve symbol information by name with optional source code inclusion".into()),
                input_schema: Arc::new(serde_json::from_value(json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Name of the symbol to search for"
                        },
                        "include_source": {
                            "type": "boolean",
                            "description": "Include source code in the response",
                            "default": false
                        }
                    },
                    "required": ["name"]
                })).unwrap()),
                output_schema: None,
                annotations: None,
                icons: None,
                title: None,
            },
            Tool {
                name: "get_symbol_references".into(),
                description: Some("Find all references to a symbol (usages, imports, calls)".into()),
                input_schema: Arc::new(serde_json::from_value(json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Name of the symbol to find references for"
                        }
                    },
                    "required": ["name"]
                })).unwrap()),
                output_schema: None,
                annotations: None,
                icons: None,
                title: None,
            },
            Tool {
                name: "find_symbols".into(),
                description: Some("Search for symbols by name using fuzzy search with optional type filtering".into()),
                input_schema: Arc::new(serde_json::from_value(json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query (supports exact match and prefix matching)"
                        },
                        "symbol_type": {
                            "type": "string",
                            "description": "Optional symbol type filter (function, class, struct, enum, interface, constant, variable, module, import)"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results to return (default: 10, max: 50)",
                            "default": 10,
                            "minimum": 1,
                            "maximum": 50
                        }
                    },
                    "required": ["query"]
                })).unwrap()),
                output_schema: None,
                annotations: None,
                icons: None,
                title: None,
            },
            Tool {
                name: "code_search".into(),
                description: Some("Search through indexed code content using BM25 statistical search".into()),
                input_schema: Arc::new(serde_json::from_value(json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query for code content"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results to return",
                            "default": 10
                        },
                        "max_results": {
                            "type": "integer", 
                            "description": "Alias for limit - maximum number of results to return",
                            "default": 10
                        },
                        "context_lines": {
                            "type": "integer",
                            "description": "Number of lines of context around matches (default: 2)",
                            "default": 2
                        }
                    },
                    "required": ["query"]
                })).unwrap()),
                output_schema: None,
                annotations: None,
                icons: None,
                title: None,
            },
            Tool {
                name: "get_file_outline".into(),
                description: Some("Get compact symbol outline for a specific file grouped by type".into()),
                input_schema: Arc::new(serde_json::from_value(json!({
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "Path to the file to outline"
                        }
                    },
                    "required": ["file_path"]
                })).unwrap()),
                output_schema: None,
                annotations: None,
                icons: None,
                title: None,
            },
            Tool {
                name: "get_directory_outline".into(),
                description: Some("Get compact directory structure showing classes by default, with optional symbol type includes".into()),
                input_schema: Arc::new(serde_json::from_value(json!({
                    "type": "object",
                    "properties": {
                        "directory_path": {
                            "type": "string",
                            "description": "Path to directory to outline"
                        },
                        "includes": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Optional symbol types to include: methods, constants, functions, enums",
                            "default": []
                        }
                    },
                    "required": ["directory_path"]
                })).unwrap()),
                output_schema: None,
                annotations: None,
                icons: None,
                title: None,
            },
        ];

        Ok(ListToolsResult {
            tools,
            next_cursor: None,
        })
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, ErrorData> {
        let prompts = vec![
            Prompt {
                name: "explain".into(),
                description: Some("Provide a structured explanation of a symbol (function, class, etc.) by analyzing its definition, usage, and relationships".into()),
                arguments: Some(vec![
                    PromptArgument {
                        name: "symbol_name".into(),
                        description: Some("Name of the symbol to explain (e.g., 'Parser', 'parse_code')".into()),
                        required: Some(true),
                        title: None,
                    }
                ]),
                title: None,
                icons: None,
            },
            Prompt {
                name: "explore".into(),
                description: Some("Efficiently explore a codebase by analyzing top-level structures and key components with minimal token usage".into()),
                arguments: Some(vec![
                    PromptArgument {
                        name: "project_path".into(),
                        description: Some("Path to the main project directory to explore (defaults to current directory)".into()),
                        required: Some(false),
                        title: None,
                    }
                ]),
                title: None,
                icons: None,
            }
        ];

        Ok(ListPromptsResult {
            prompts,
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        match request.name.as_ref() {
            "index_code" => self.index_code(request.arguments).await,
            "get_symbol" => self.get_symbol(request.arguments).await,
            "get_symbol_references" => self.get_symbol_references(request.arguments).await,
            "find_symbols" => self.find_symbols(request.arguments).await,
            "code_search" => self.code_search(request.arguments).await,
            "get_file_outline" => OutlineTools::get_file_outline(request.arguments).await,
            "get_directory_outline" => OutlineTools::get_directory_outline(request.arguments).await,
            _ => Err(ErrorData::new(
                ErrorCode::METHOD_NOT_FOUND,
                "Method not found",
                None,
            )),
        }
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, ErrorData> {
        match request.name.as_str() {
            "explain" => {
                let symbol_name = request
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("symbol_name"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ErrorData {
                        code: ErrorCode::INVALID_PARAMS,
                        message: "Missing required argument: symbol_name".into(),
                        data: None,
                    })?;

                let prompt_text = format!(r#"I need to explain the symbol "{}" in a structured way. Please follow these steps methodically:

1. **Get Symbol Information**
   - Use get_symbol tool with name="{}" and include_source=false
   - This will show the symbol's type, location, and basic info

2. **Analyze Symbol Type and Get Additional Context**
   - If it's a CLASS/STRUCT/INTERFACE:
     * Use get_file_outline tool on the file containing this symbol
     * This shows all methods, properties, and structure
   
   - If it's a FUNCTION/METHOD:
     * Use get_symbol_references tool with name="{}"
     * This shows where and how it's used throughout the codebase

3. **Provide Structured Explanation**
   Based on the gathered information, explain:
   - **Purpose**: What this symbol does/represents
   - **Type**: Function, class, method, etc.
   - **Location**: File and line number
   - **Key Details**: Parameters, return type, visibility
   - **Usage Context**: How it's used (if method/function) or what it contains (if class)
   - **Relationships**: Dependencies or related symbols

Keep the explanation concise but comprehensive. Focus on practical understanding."#, symbol_name, symbol_name, symbol_name);

                Ok(GetPromptResult {
                    description: Some(format!("Structured explanation of symbol '{}'", symbol_name)),
                    messages: vec![PromptMessage {
                        role: PromptMessageRole::User,
                        content: PromptMessageContent::Text { text: prompt_text },
                    }],
                })
            }
            "explore" => {
                let project_path = request
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("project_path"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(".");

                let prompt_text = format!(r#"I need to efficiently explore the codebase at "{}" with MINIMAL token usage. Follow this strategic approach:

**PHASE 1: High-Level Architecture (Token-Efficient)**
1. Use get_directory_outline with path="{}" and includes=["classes", "structs", "interfaces"]
   - This shows ALL top-level classes/structs/interfaces across the project
   - Identify 3-5 most important/central components based on names and structure

**PHASE 2: Strategic Deep-Dive (Only for Key Components)**
For the 2-3 MOST IMPORTANT components identified:
2. Use get_file_outline on their files to see full structure with methods/properties
3. For CORE methods (constructors, main business logic, public APIs):
   - Use get_symbol with include_source=true (ONLY for 2-3 critical methods)
   - Use get_symbol_references to understand usage patterns

**PHASE 3: Targeted Code Search (If Needed)**
4. ONLY if unclear about key concepts, use code_search with specific queries like:
   - "main entry point" or "initialization" or "core algorithm"
   - Limit to 3-5 results max

**DELIVERABLE: Concise Architecture Summary**
Provide a brief overview covering:
- **Project Purpose**: What this codebase does
- **Key Components**: 3-5 main classes/structs and their roles  
- **Architecture Pattern**: How components interact
- **Entry Points**: Main functions or initialization paths
- **Notable Patterns**: Any interesting design decisions

BE EXTREMELY FRUGAL - Skip details that don't contribute to understanding the overall architecture."#, project_path, project_path);

                Ok(GetPromptResult {
                    description: Some(format!("Efficient exploration of project at '{}'", project_path)),
                    messages: vec![PromptMessage {
                        role: PromptMessageRole::User,
                        content: PromptMessageContent::Text { text: prompt_text },
                    }],
                })
            }
            _ => Err(ErrorData {
                code: ErrorCode::METHOD_NOT_FOUND,
                message: format!("Unknown prompt: {}", request.name).into(),
                data: None,
            }),
        }
    }
}

impl CodeAnalysisTools {
    async fn index_code(
        &self,
        arguments: Option<Map<String, Value>>,
    ) -> Result<CallToolResult, ErrorData> {
        let start_time = Instant::now();

        let args = arguments
            .ok_or_else(|| ErrorData::new(ErrorCode::INVALID_PARAMS, "Missing arguments", None))?;
        let params: IndexCodeRequest =
            serde_json::from_value(Value::Object(args)).map_err(|e| {
                ErrorData::new(
                    ErrorCode::INVALID_PARAMS,
                    format!("Invalid arguments: {}", e),
                    None,
                )
            })?;

        // Use comprehensive path resolution
        let path = PathResolver::resolve_file_or_directory_path(&params.path)?;

        let pipeline = get_indexing_pipeline();
        let mut pipeline_guard = pipeline.lock().await;

        let result = if path.is_file() {
            pipeline_guard.index_file(&path).await.map_err(|e| {
                ErrorData::new(
                    ErrorCode::INTERNAL_ERROR,
                    format!("Failed to index file: {}", e),
                    None,
                )
            })?;
            (1, get_symbol_store().get_symbol_count() as u32)
        } else {
            let index_result = pipeline_guard.index_directory(&path).await;
            (index_result.files_processed, index_result.symbols_found)
        };

        // Start file watching for directories
        if path.is_dir() {
            if let Err(e) = start_file_watcher(path.clone()).await {
                tracing::warn!("Failed to start file watcher for {:?}: {}", path, e);
                // Don't fail the indexing if file watching fails
            }
        }

        let duration = start_time.elapsed();

        let response = IndexCodeResponse {
            status: "success".to_string(),
            files_indexed: result.0,
            symbols_found: result.1,
            errors: vec![],
            duration_ms: duration.as_millis() as u64,
        };

        let response_text = serde_json::to_string_pretty(&response).map_err(|e| {
            ErrorData::new(
                ErrorCode::INTERNAL_ERROR,
                format!("Serialization error: {}", e),
                None,
            )
        })?;

        Ok(CallToolResult::success(vec![Content::text(response_text)]))
    }

    async fn get_symbol(
        &self,
        arguments: Option<Map<String, Value>>,
    ) -> Result<CallToolResult, ErrorData> {
        let args = arguments
            .ok_or_else(|| ErrorData::new(ErrorCode::INVALID_PARAMS, "Missing arguments", None))?;
        let params: GetSymbolRequest =
            serde_json::from_value(Value::Object(args)).map_err(|e| {
                ErrorData::new(
                    ErrorCode::INVALID_PARAMS,
                    format!("Invalid arguments: {}", e),
                    None,
                )
            })?;

        let store = get_symbol_store();
        let mut symbols = store.get_symbols(&params.name);

        // Add source code if requested
        if params.include_source.unwrap_or(false) {
            for symbol in &mut symbols {
                if symbol.source.is_none() {
                    // Try to read source code from file
                    if let Ok(content) = tokio::fs::read_to_string(&symbol.location.file).await {
                        let lines: Vec<&str> = content.lines().collect();
                        let start_line = (symbol.location.start_line as usize).saturating_sub(1);
                        let end_line =
                            std::cmp::min(symbol.location.end_line as usize, lines.len());

                        if start_line < lines.len() {
                            let source_lines = &lines[start_line..end_line];
                            symbol.source = Some(source_lines.join("\n"));
                        }
                    }
                }
            }
        }

        let response = GetSymbolResponse { symbols };

        let response_text = serde_json::to_string_pretty(&response).map_err(|e| {
            ErrorData::new(
                ErrorCode::INTERNAL_ERROR,
                format!("Serialization error: {}", e),
                None,
            )
        })?;

        Ok(CallToolResult::success(vec![Content::text(response_text)]))
    }

    async fn get_symbol_references(
        &self,
        arguments: Option<Map<String, Value>>,
    ) -> Result<CallToolResult, ErrorData> {
        let args = arguments
            .ok_or_else(|| ErrorData::new(ErrorCode::INVALID_PARAMS, "Missing arguments", None))?;
        let params: GetSymbolReferencesRequest = serde_json::from_value(Value::Object(args))
            .map_err(|e| {
                ErrorData::new(
                    ErrorCode::INVALID_PARAMS,
                    format!("Invalid arguments: {}", e),
                    None,
                )
            })?;

        let store = get_symbol_store();
        let references = store.get_references_by_name(&params.name);
        let response = GetSymbolReferencesResponse { references };

        let response_text = serde_json::to_string_pretty(&response).map_err(|e| {
            ErrorData::new(
                ErrorCode::INTERNAL_ERROR,
                format!("Serialization error: {}", e),
                None,
            )
        })?;

        Ok(CallToolResult::success(vec![Content::text(response_text)]))
    }

    async fn find_symbols(
        &self,
        arguments: Option<Map<String, Value>>,
    ) -> Result<CallToolResult, ErrorData> {
        let args = arguments
            .ok_or_else(|| ErrorData::new(ErrorCode::INVALID_PARAMS, "Missing arguments", None))?;
        let params: FindSymbolsRequest =
            serde_json::from_value(Value::Object(args)).map_err(|e| {
                ErrorData::new(
                    ErrorCode::INVALID_PARAMS,
                    format!("Invalid arguments: {}", e),
                    None,
                )
            })?;

        let store = get_symbol_store();

        // Apply limit with bounds checking
        let limit = params.limit.unwrap_or(10).min(50).max(1) as usize;

        let fuzzy_results = store.find_symbols_fuzzy(&params.query);
        let mut symbols: Vec<_> = fuzzy_results
            .into_iter()
            .map(|(symbol, _score)| symbol)
            .collect();

        // Filter by symbol type if specified
        if let Some(ref type_filter) = params.symbol_type {
            let target_type = match type_filter.to_lowercase().as_str() {
                "function" => Some(SymbolType::Function),
                "method" => Some(SymbolType::Method),
                "class" => Some(SymbolType::Class),
                "struct" => Some(SymbolType::Struct),
                "enum" => Some(SymbolType::Enum),
                "interface" | "trait" => Some(SymbolType::Interface),
                "constant" | "const" => Some(SymbolType::Constant),
                "variable" | "var" => Some(SymbolType::Variable),
                "module" | "mod" => Some(SymbolType::Module),
                "import" => Some(SymbolType::Import),
                _ => None,
            };

            if let Some(target_type) = target_type {
                symbols.retain(|s| s.symbol_type == target_type);
            }
        }

        // Apply limit to results
        symbols.truncate(limit);

        let response = FindSymbolsResponse { symbols };

        let response_text = serde_json::to_string_pretty(&response).map_err(|e| {
            ErrorData::new(
                ErrorCode::INTERNAL_ERROR,
                format!("Serialization error: {}", e),
                None,
            )
        })?;

        Ok(CallToolResult::success(vec![Content::text(response_text)]))
    }

    async fn code_search(
        &self,
        arguments: Option<Map<String, Value>>,
    ) -> Result<CallToolResult, ErrorData> {
        let args = arguments
            .ok_or_else(|| ErrorData::new(ErrorCode::INVALID_PARAMS, "Missing arguments", None))?;
        let params: CodeSearchRequest =
            serde_json::from_value(Value::Object(args)).map_err(|e| {
                ErrorData::new(
                    ErrorCode::INVALID_PARAMS,
                    format!("Invalid arguments: {}", e),
                    None,
                )
            })?;

        let store = get_symbol_store();
        let limit = params.max_results.or(params.limit).unwrap_or(10) as usize;
        let context_lines = params.context_lines.unwrap_or(2) as usize;

        let search_results = store.search_code(&params.query, limit, context_lines);

        // Convert to response format
        let results: Vec<CodeSearchResult> = search_results
            .into_iter()
            .map(|result| CodeSearchResult {
                score: result.score,
                file_path: result.file_path.to_string_lossy().to_string(),
                language: result.language,
                content_snippet: result.content_snippet,
            })
            .collect();

        let response = CodeSearchResponse {
            total_found: results.len(),
            results,
        };

        let response_text = serde_json::to_string_pretty(&response).map_err(|e| {
            ErrorData::new(
                ErrorCode::INTERNAL_ERROR,
                format!("Serialization error: {}", e),
                None,
            )
        })?;

        Ok(CallToolResult::success(vec![Content::text(response_text)]))
    }
}

#[cfg(test)]
mod tests {
    use crate::models::{Location, Symbol, SymbolId, SymbolType, Visibility};
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_include_source() {
        // Create a temporary file with test content
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "fn test_function() {{").unwrap();
        writeln!(temp_file, "    println!(\"Hello, world!\");").unwrap();
        writeln!(temp_file, "}}").unwrap();

        let file_path = temp_file.path().to_path_buf();

        // Create a test symbol
        let mut symbol = Symbol {
            id: SymbolId::new(&file_path, 1, 0),
            name: "test_function".to_string(),
            symbol_type: SymbolType::Function,
            location: Location {
                file: file_path.clone(),
                start_line: 1,
                start_column: 0,
                end_line: 3,
                end_column: 1,
            },
            namespace: None,
            visibility: Visibility::Public,
            source: None,
        };

        // Test source extraction
        if let Ok(content) = tokio::fs::read_to_string(&symbol.location.file).await {
            let lines: Vec<&str> = content.lines().collect();
            let start_line = (symbol.location.start_line as usize).saturating_sub(1);
            let end_line = std::cmp::min(symbol.location.end_line as usize, lines.len());

            if start_line < lines.len() {
                let source_lines = &lines[start_line..end_line];
                symbol.source = Some(source_lines.join("\n"));
            }
        }

        // Verify source was extracted
        assert!(symbol.source.is_some());
        let source = symbol.source.as_ref().unwrap();
        assert!(source.contains("test_function"));
        assert!(source.contains("println!"));
    }
}
