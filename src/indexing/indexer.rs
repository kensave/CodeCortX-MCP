use crate::models::{
    Language, Location, Reference, ReferenceType, Symbol, SymbolId, SymbolType, Visibility,
};
use std::collections::HashMap;
use std::path::PathBuf;
use tree_sitter::{Parser, Query, StreamingIterator};

const RUST_QUERY: &str = include_str!("../../queries/rust-symbols.scm");
const PYTHON_QUERY: &str = include_str!("../../queries/python-symbols.scm");
const C_QUERY: &str = include_str!("../../queries/c-symbols.scm");
const CPP_QUERY: &str = include_str!("../../queries/cpp-symbols.scm");
const JAVA_QUERY: &str = include_str!("../../queries/java-symbols.scm");
const GO_QUERY: &str = include_str!("../../queries/go-symbols.scm");
const JAVASCRIPT_QUERY: &str = include_str!("../../queries/javascript-symbols.scm");
const TYPESCRIPT_QUERY: &str = include_str!("../../queries/typescript-symbols.scm");
const RUBY_QUERY: &str = include_str!("../../queries/ruby-symbols.scm");
const CSHARP_QUERY: &str = include_str!("../../queries/csharp-symbols.scm");
const KOTLIN_QUERY: &str = include_str!("../../queries/kotlin-symbols.scm");
const SCALA_QUERY: &str = include_str!("../../queries/scala-symbols.scm");
const SWIFT_QUERY: &str = include_str!("../../queries/swift-symbols.scm");
const PHP_QUERY: &str = include_str!("../../queries/php-symbols.scm");
const OBJC_QUERY: &str = include_str!("../../queries/objc-symbols.scm");

pub struct SymbolIndexer {
    parsers: HashMap<Language, Parser>,
    queries: HashMap<Language, Query>,
    reference_queries: HashMap<Language, Query>,
}

impl SymbolIndexer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut indexer = Self {
            parsers: HashMap::new(),
            queries: HashMap::new(),
            reference_queries: HashMap::new(),
        };

        // Initialize parsers and queries for each language
        indexer.init_language(Language::Rust)?;
        indexer.init_language(Language::Python)?;
        indexer.init_language(Language::C)?;
        indexer.init_language(Language::Cpp)?;
        indexer.init_language(Language::Java)?;
        indexer.init_language(Language::Go)?;
        indexer.init_language(Language::JavaScript)?;
        indexer.init_language(Language::TypeScript)?;
        indexer.init_language(Language::Ruby)?;
        indexer.init_language(Language::CSharp)?;
        indexer.init_language(Language::Kotlin)?;
        indexer.init_language(Language::Scala)?;
        indexer.init_language(Language::Swift)?;
        indexer.init_language(Language::PHP)?;
        indexer.init_language(Language::ObjectiveC)?;

        Ok(indexer)
    }

    fn init_language(&mut self, language: Language) -> Result<(), Box<dyn std::error::Error>> {
        let ts_language = language.tree_sitter_language();

        // Create parser
        let mut parser = Parser::new();
        parser.set_language(&ts_language)?;
        self.parsers.insert(language, parser);

        // Compile query
        let query_source = match language {
            Language::Rust => RUST_QUERY,
            Language::Python => PYTHON_QUERY,
            Language::C => C_QUERY,
            Language::Cpp => CPP_QUERY,
            Language::Java => JAVA_QUERY,
            Language::Go => GO_QUERY,
            Language::JavaScript => JAVASCRIPT_QUERY,
            Language::TypeScript => TYPESCRIPT_QUERY,
            Language::Ruby => RUBY_QUERY,
            Language::CSharp => CSHARP_QUERY,
            Language::Kotlin => KOTLIN_QUERY,
            Language::Scala => SCALA_QUERY,
            Language::Swift => SWIFT_QUERY,
            Language::PHP => PHP_QUERY,
            Language::ObjectiveC => OBJC_QUERY,
        };

        let query = Query::new(&ts_language, query_source)
            .map_err(|e| format!("Failed to compile query for {:?}: {}", language, e))?;
        self.queries.insert(language, query);

        // Compile reference query
        let ref_query_source = match language {
            Language::Rust => include_str!("../../queries/rust-references.scm"),
            Language::Python => include_str!("../../queries/python-references.scm"),
            Language::C => include_str!("../../queries/c-references.scm"),
            Language::Cpp => include_str!("../../queries/cpp-references.scm"),
            Language::Java => include_str!("../../queries/java-references.scm"),
            Language::Go => include_str!("../../queries/go-references.scm"),
            Language::JavaScript => include_str!("../../queries/javascript-references.scm"),
            Language::TypeScript => include_str!("../../queries/typescript-references.scm"),
            Language::Ruby => include_str!("../../queries/ruby-references.scm"),
            Language::CSharp => include_str!("../../queries/csharp-references.scm"),
            Language::Kotlin => include_str!("../../queries/kotlin-references.scm"),
            Language::Scala => include_str!("../../queries/scala-references.scm"),
            Language::Swift => include_str!("../../queries/swift-references.scm"),
            Language::PHP => include_str!("../../queries/php-references.scm"),
            Language::ObjectiveC => include_str!("../../queries/objc-references.scm"),
        };

        let ref_query = Query::new(&ts_language, ref_query_source).map_err(|e| {
            format!(
                "Failed to compile reference query for {:?}: {}",
                language, e
            )
        })?;
        self.reference_queries.insert(language, ref_query);

        Ok(())
    }

    pub fn extract_symbols(
        &mut self,
        source: &str,
        language: Language,
        file_path: &PathBuf,
    ) -> Result<Vec<Symbol>, Box<dyn std::error::Error>> {
        let parser = self
            .parsers
            .get_mut(&language)
            .ok_or("Parser not found for language")?;

        let query = self
            .queries
            .get(&language)
            .ok_or("Query not found for language")?;

        // Parse source code with error handling
        let tree = match parser.parse(source, None) {
            Some(tree) => tree,
            None => {
                // Return empty symbols instead of failing completely
                tracing::warn!("Failed to parse source code, continuing with empty symbols");
                return Ok(Vec::new());
            }
        };

        // Check for parse errors in the tree
        if tree.root_node().has_error() {
            tracing::warn!("Parse tree contains errors, extracting partial symbols");
        }

        let mut symbols = Vec::new();
        let mut cursor = tree_sitter::QueryCursor::new();

        // Execute tree-sitter query and extract symbols
        let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
        while let Some(match_) = matches.next() {
            if let Some(symbol) =
                self.create_symbol_from_match(&match_, source, file_path, language)
            {
                symbols.push(symbol);
            }
        }

        Ok(symbols)
    }

    pub fn extract_references(
        &mut self,
        source: &str,
        language: Language,
        file_path: &PathBuf,
    ) -> Result<Vec<Reference>, Box<dyn std::error::Error>> {
        let query = self
            .reference_queries
            .get(&language)
            .ok_or("Reference query not found")?;

        let parser = self.parsers.get_mut(&language).ok_or("Parser not found")?;

        let tree = parser.parse(source, None).ok_or("Failed to parse")?;

        let mut references = Vec::new();
        let mut cursor = tree_sitter::QueryCursor::new();

        let mut matches = cursor.matches(query, tree.root_node(), source.as_bytes());
        while let Some(match_) = matches.next() {
            for capture in match_.captures {
                let node = capture.node;
                let start_pos = node.start_position();
                let end_pos = node.end_position();

                let location = Location::new(
                    file_path.clone(),
                    start_pos.row as u32 + 1,
                    start_pos.column as u32,
                    end_pos.row as u32 + 1,
                    end_pos.column as u32,
                );

                references.push(Reference {
                    location,
                    reference_type: ReferenceType::Usage,
                    target_symbol: SymbolId::new(file_path, 0, 0), // Will be resolved later
                });
            }
        }

        Ok(references)
    }

    fn create_symbol_from_match(
        &self,
        match_: &tree_sitter::QueryMatch,
        source: &str,
        file_path: &PathBuf,
        language: Language,
    ) -> Option<Symbol> {
        let query = self.queries.get(&language)?;

        // Find name capture and definition capture
        let name_capture = match_.captures.iter().find(|capture| {
            let capture_name = &query.capture_names()[capture.index as usize];
            capture_name.ends_with(".name")
        })?;

        let definition_capture = match_.captures.iter().find(|capture| {
            let capture_name = &query.capture_names()[capture.index as usize];
            capture_name.ends_with(".definition")
        });

        let name_node = name_capture.node;
        let name = name_node.utf8_text(source.as_bytes()).ok()?.to_string();

        // Determine symbol type from capture name
        let capture_name = &query.capture_names()[name_capture.index as usize];
        let symbol_type = self.determine_symbol_type(capture_name);

        // Use definition node for location if available, otherwise use name node
        let location_node = definition_capture.map(|c| c.node).unwrap_or(name_node);
        let start_pos = location_node.start_position();
        let end_pos = location_node.end_position();

        let location = Location::new(
            file_path.clone(),
            start_pos.row as u32 + 1, // Tree-sitter uses 0-based rows
            start_pos.column as u32,
            end_pos.row as u32 + 1,
            end_pos.column as u32,
        );

        // Generate symbol ID
        let symbol_id = SymbolId::new(file_path, location.start_line, location.start_column);

        Some(Symbol {
            id: symbol_id,
            name,
            symbol_type,
            location,
            namespace: None,                // TODO: Extract namespace in future
            visibility: Visibility::Public, // TODO: Determine visibility
            source: None,
        })
    }

    fn determine_symbol_type(&self, capture_name: &str) -> SymbolType {
        if capture_name.starts_with("function") {
            SymbolType::Function
        } else if capture_name.starts_with("method") {
            SymbolType::Method
        } else if capture_name.starts_with("class") {
            SymbolType::Class
        } else if capture_name.starts_with("struct") {
            SymbolType::Struct
        } else if capture_name.starts_with("enum") {
            SymbolType::Enum
        } else if capture_name.starts_with("trait") || capture_name.starts_with("interface") {
            SymbolType::Interface
        } else if capture_name.starts_with("const") || capture_name.starts_with("static") {
            SymbolType::Constant
        } else if capture_name.starts_with("module") {
            SymbolType::Module
        } else if capture_name.starts_with("import") {
            SymbolType::Import
        } else if capture_name.starts_with("variable") {
            SymbolType::Variable
        } else {
            SymbolType::Variable // Default fallback
        }
    }

    pub fn get_parser(&mut self, language: Language) -> Option<&mut Parser> {
        self.parsers.get_mut(&language)
    }

    pub fn get_query(&self, language: Language) -> Option<&Query> {
        self.queries.get(&language)
    }

    pub fn supports_language(language: Language) -> bool {
        matches!(language, Language::Rust | Language::Python)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_language_detection() {
        assert_eq!(Language::from_extension("rs"), Some(Language::Rust));
        assert_eq!(Language::from_extension("py"), Some(Language::Python));
        assert_eq!(Language::from_extension("js"), Some(Language::JavaScript));
        assert_eq!(Language::from_extension("cs"), Some(Language::CSharp));
        assert_eq!(Language::from_extension("unknown"), None);

        assert_eq!(
            Language::from_path(Path::new("test.rs")),
            Some(Language::Rust)
        );
        assert_eq!(
            Language::from_path(Path::new("test.py")),
            Some(Language::Python)
        );
        assert_eq!(
            Language::from_path(Path::new("test.js")),
            Some(Language::JavaScript)
        );
        assert_eq!(
            Language::from_path(Path::new("test.cs")),
            Some(Language::CSharp)
        );
        assert_eq!(Language::from_path(Path::new("test.unknown")), None);
    }

    #[test]
    fn test_source_file_detection() {
        assert!(Language::is_source_file(Path::new("main.rs")));
        assert!(Language::is_source_file(Path::new("script.py")));
        assert!(!Language::is_source_file(Path::new("README.md")));
    }

    #[test]
    fn test_parser_initialization() {
        let indexer = SymbolIndexer::new().unwrap();
        assert!(indexer.parsers.contains_key(&Language::Rust));
        assert!(indexer.parsers.contains_key(&Language::Python));
    }

    #[test]
    fn test_query_compilation() {
        let indexer = SymbolIndexer::new().unwrap();
        assert!(indexer.get_query(Language::Rust).is_some());
        assert!(indexer.get_query(Language::Python).is_some());
    }

    #[test]
    fn test_rust_symbol_extraction() {
        let mut indexer = SymbolIndexer::new().unwrap();
        let rust_code = r#"
            fn test_function() {}
            struct TestStruct {}
            const TEST_CONST: i32 = 42;
        "#;

        let file_path = PathBuf::from("test.rs");
        let symbols = indexer
            .extract_symbols(rust_code, Language::Rust, &file_path)
            .unwrap();

        assert_eq!(symbols.len(), 3);
        assert!(symbols
            .iter()
            .any(|s| s.name == "test_function" && s.symbol_type == SymbolType::Function));
        assert!(symbols
            .iter()
            .any(|s| s.name == "TestStruct" && s.symbol_type == SymbolType::Struct));
        assert!(symbols
            .iter()
            .any(|s| s.name == "TEST_CONST" && s.symbol_type == SymbolType::Constant));
    }

    #[test]
    fn test_python_symbol_extraction() {
        let mut indexer = SymbolIndexer::new().unwrap();
        let python_code = r#"
def test_function():
    pass

class TestClass:
    def method(self):
        pass

test_var = 42
        "#;

        let file_path = PathBuf::from("test.py");
        let symbols = indexer
            .extract_symbols(python_code, Language::Python, &file_path)
            .unwrap();

        assert!(symbols.len() >= 2); // At least function and class
        assert!(symbols
            .iter()
            .any(|s| s.name == "test_function" && s.symbol_type == SymbolType::Function));
        assert!(symbols
            .iter()
            .any(|s| s.name == "TestClass" && s.symbol_type == SymbolType::Class));
    }

    #[test]
    fn test_language_support() {
        assert!(SymbolIndexer::supports_language(Language::Rust));
        assert!(SymbolIndexer::supports_language(Language::Python));
    }
}
