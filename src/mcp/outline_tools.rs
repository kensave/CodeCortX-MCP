use crate::mcp::tools::get_symbol_store;
use crate::models::{Symbol, SymbolType, Visibility};
use crate::utils::PathResolver;
use rmcp::model::{CallToolResult, Content, ErrorCode, ErrorData};
use serde_json::{Map, Value};

pub struct OutlineTools;

impl OutlineTools {
    pub async fn get_file_outline(
        arguments: Option<Map<String, Value>>,
    ) -> Result<CallToolResult, ErrorData> {
        let args = arguments
            .ok_or_else(|| ErrorData::new(ErrorCode::INVALID_PARAMS, "Missing arguments", None))?;
        let file_path = args
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ErrorData::new(ErrorCode::INVALID_PARAMS, "Missing file_path", None))?;

        // Use comprehensive path resolution
        let canonical_path = PathResolver::resolve_file_path(file_path)?;

        let store = get_symbol_store();
        let symbols = store.get_symbols_by_file(&canonical_path);

        // Check if file has symbols (is indexed)
        if symbols.is_empty() {
            return Err(ErrorData::new(
                ErrorCode::INVALID_PARAMS, 
                format!("No symbols found for file '{}'. Make sure the file is indexed first.", file_path), 
                None
            ));
        }

        let mut outline = std::collections::BTreeMap::new();
        for mut symbol in symbols {
            Self::extract_source_if_needed(&mut symbol).await;

            let type_name = Self::get_symbol_category(&symbol.symbol_type);
            let display_name = Self::format_symbol_display(&symbol);

            outline
                .entry(type_name)
                .or_insert_with(Vec::new)
                .push(format!(
                    "  ├─ {} ({}:{}) {}",
                    display_name,
                    symbol.location.start_line,
                    symbol.location.start_column,
                    Self::format_visibility(&symbol.visibility)
                ));
        }

        let result = Self::format_outline(outline);
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    pub async fn get_directory_outline(
        arguments: Option<Map<String, Value>>,
    ) -> Result<CallToolResult, ErrorData> {
        let args = arguments
            .ok_or_else(|| ErrorData::new(ErrorCode::INVALID_PARAMS, "Missing arguments", None))?;
        let directory_path = args
            .get("directory_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ErrorData::new(ErrorCode::INVALID_PARAMS, "Missing directory_path", None)
            })?;

        // Use comprehensive path resolution
        let canonical_path = PathResolver::resolve_directory_path(directory_path)?;

        let includes: Vec<String> = args
            .get("includes")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_else(|| {
                vec![
                    "functions".to_string(),
                    "classes".to_string(),
                    "structs".to_string(),
                ]
            });

        let store = get_symbol_store();
        let mut file_symbols = std::collections::BTreeMap::new();

        for entry in store.symbol_data.iter() {
            let symbol = entry.value();
            let file_path = &symbol.location.file;

            // Check if file is in the target directory using canonical paths
            if file_path.starts_with(&canonical_path) {
                let should_include = match symbol.symbol_type {
                    SymbolType::Class => includes.contains(&"classes".to_string()),
                    SymbolType::Struct => includes.contains(&"structs".to_string()),
                    SymbolType::Enum => includes.contains(&"enums".to_string()),
                    SymbolType::Function => includes.contains(&"functions".to_string()),
                    SymbolType::Method => includes.contains(&"methods".to_string()),
                    SymbolType::Constant => includes.contains(&"constants".to_string()),
                    SymbolType::Variable => includes.contains(&"variables".to_string()),
                    SymbolType::Module => includes.contains(&"modules".to_string()),
                    _ => false,
                };

                if should_include {
                    let rel_path = file_path.strip_prefix(&canonical_path).unwrap_or(file_path);
                    let file_name = rel_path.to_string_lossy().to_string();

                    file_symbols
                        .entry(file_name)
                        .or_insert_with(Vec::new)
                        .push(format!(
                            "{} ({})",
                            symbol.name,
                            Self::get_symbol_category(&symbol.symbol_type).to_lowercase()
                        ));
                }
            }
        }

        let mut result = format!(
            "Directory: {} ({} files)\n\n",
            directory_path,
            file_symbols.len()
        );
        let total_symbols: usize = file_symbols.values().map(|v| v.len()).sum();
        let file_count = file_symbols.len();

        for (file_name, symbols) in file_symbols {
            result.push_str(&format!("📁 {}\n", file_name));
            for symbol in symbols {
                result.push_str(&format!("  🏗️ {}\n", symbol));
            }
            result.push('\n');
        }

        // Add summary
        result.push_str(&format!(
            "Summary: {} symbols across {} files",
            total_symbols, file_count
        ));

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    async fn extract_source_if_needed(symbol: &mut Symbol) {
        if matches!(
            symbol.symbol_type,
            SymbolType::Function | SymbolType::Method
        ) && symbol.source.is_none()
        {
            if let Ok(content) = tokio::fs::read_to_string(&symbol.location.file).await {
                let lines: Vec<&str> = content.lines().collect();
                let start_line = (symbol.location.start_line as usize).saturating_sub(1);
                let end_line = std::cmp::min(symbol.location.end_line as usize, lines.len());

                if start_line < lines.len() {
                    let source_lines = &lines[start_line..end_line];
                    symbol.source = Some(source_lines.join("\n"));
                }
            }
        }
    }

    fn get_symbol_category(symbol_type: &SymbolType) -> &'static str {
        match symbol_type {
            SymbolType::Class | SymbolType::Struct => "Classes",
            SymbolType::Function | SymbolType::Method => "Functions",
            SymbolType::Constant => "Constants",
            SymbolType::Enum => "Enums",
            SymbolType::Interface => "Interfaces",
            SymbolType::Module => "Modules",
            SymbolType::Import => "Imports",
            SymbolType::Variable => "Variables",
        }
    }

    fn format_symbol_display(symbol: &Symbol) -> String {
        if matches!(
            symbol.symbol_type,
            SymbolType::Function | SymbolType::Method
        ) {
            if let Some(source) = &symbol.source {
                let first_line = source.lines().next().unwrap_or(&symbol.name).trim();
                if first_line.len() > 80 {
                    format!("{}...", &first_line[..77])
                } else {
                    first_line.to_string()
                }
            } else {
                format!("{}()", symbol.name)
            }
        } else {
            symbol.name.clone()
        }
    }

    fn format_visibility(visibility: &Visibility) -> &'static str {
        match visibility {
            Visibility::Public => "pub",
            Visibility::Private => "priv",
            Visibility::Protected => "prot",
            Visibility::Internal => "int",
        }
    }

    fn format_outline(outline: std::collections::BTreeMap<&'static str, Vec<String>>) -> String {
        let mut result = String::new();
        for (type_name, items) in outline {
            result.push_str(&format!("{}:\n", type_name));
            for item in items {
                result.push_str(&format!("{}\n", item));
            }
            result.push('\n');
        }
        result
    }
}
