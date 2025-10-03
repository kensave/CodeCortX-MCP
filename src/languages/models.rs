use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct SymbolId(pub u64);

impl SymbolId {
    pub fn new(file_path: &PathBuf, start_line: u32, start_column: u32) -> Self {
        let mut hasher = DefaultHasher::new();
        file_path.hash(&mut hasher);
        start_line.hash(&mut hasher);
        start_column.hash(&mut hasher);
        Self(hasher.finish())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct Symbol {
    pub id: SymbolId,
    pub name: String,
    pub symbol_type: SymbolType,
    pub location: Location,
    pub namespace: Option<String>,
    pub visibility: Visibility,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Encode, Decode)]
pub enum SymbolType {
    Module,
    Class,
    Interface,
    Method,
    Function,
    Constant,
    Variable,
    Enum,
    Struct,
    Import,
}

impl SymbolType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SymbolType::Module => "module",
            SymbolType::Class => "class",
            SymbolType::Interface => "interface",
            SymbolType::Method => "method",
            SymbolType::Function => "function",
            SymbolType::Constant => "constant",
            SymbolType::Variable => "variable",
            SymbolType::Enum => "enum",
            SymbolType::Struct => "struct",
            SymbolType::Import => "import",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Encode, Decode)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct Location {
    pub file: PathBuf,
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

impl Location {
    pub fn new(
        file: PathBuf,
        start_line: u32,
        start_column: u32,
        end_line: u32,
        end_column: u32,
    ) -> Self {
        Self {
            file,
            start_line,
            start_column,
            end_line,
            end_column,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct Reference {
    pub location: Location,
    pub reference_type: ReferenceType,
    pub target_symbol: SymbolId,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Encode, Decode)]
pub enum ReferenceType {
    Definition,
    Usage,
    Import,
    Call,
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct FileInfo {
    pub last_modified: SystemTime,
    pub content_hash: [u8; 32],
    pub symbol_count: u32,
    pub parse_status: ParseStatus,
    pub file_size: u64,
}

impl FileInfo {
    pub fn new(content_hash: [u8; 32], file_size: u64) -> Self {
        Self {
            last_modified: SystemTime::now(),
            content_hash,
            symbol_count: 0,
            parse_status: ParseStatus::NotParsed,
            file_size,
        }
    }

    pub fn from_file_content(content: &str) -> Self {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let hash: [u8; 32] = hasher.finalize().into();

        Self::new(hash, content.len() as u64)
    }

    pub async fn from_file_path(path: &std::path::Path) -> Result<Self, std::io::Error> {
        let content = tokio::fs::read_to_string(path).await?;
        let metadata = tokio::fs::metadata(path).await?;

        let mut file_info = Self::from_file_content(&content);
        file_info.last_modified = metadata.modified().unwrap_or(SystemTime::now());
        file_info.file_size = metadata.len();

        Ok(file_info)
    }

    pub fn has_changed(&self, other: &FileInfo) -> bool {
        self.content_hash != other.content_hash
            || self.last_modified != other.last_modified
            || self.file_size != other.file_size
    }

    pub fn update_parse_status(&mut self, status: ParseStatus, symbol_count: u32) {
        self.parse_status = status;
        self.symbol_count = symbol_count;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub enum ParseStatus {
    Success,
    PartialSuccess(Vec<String>),
    Failed(String),
    NotParsed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Rust,
    Python,
    C,
    Cpp,
    Java,
    Go,
    JavaScript,
    TypeScript,
    Ruby,
    CSharp,
    Kotlin,
    Scala,
    Swift,
    PHP,
    ObjectiveC,
}

impl Language {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "rs" => Some(Language::Rust),
            "py" => Some(Language::Python),
            "c" | "h" => Some(Language::C),
            "cpp" | "cc" | "cxx" | "hpp" | "hxx" => Some(Language::Cpp),
            "java" => Some(Language::Java),
            "go" => Some(Language::Go),
            "js" | "jsx" => Some(Language::JavaScript),
            "ts" | "tsx" => Some(Language::TypeScript),
            "rb" => Some(Language::Ruby),
            "cs" => Some(Language::CSharp),
            "kt" | "kts" => Some(Language::Kotlin),
            "scala" | "sc" => Some(Language::Scala),
            "swift" => Some(Language::Swift),
            "php" => Some(Language::PHP),
            "m" | "mm" => Some(Language::ObjectiveC),
            _ => None,
        }
    }

    pub fn from_path(path: &std::path::Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| Self::from_extension(ext))
    }

    pub fn tree_sitter_language(&self) -> tree_sitter::Language {
        match self {
            Language::Rust => tree_sitter_rust::LANGUAGE.into(),
            Language::Python => tree_sitter_python::LANGUAGE.into(),
            Language::C => tree_sitter_c::LANGUAGE.into(),
            Language::Cpp => tree_sitter_cpp::LANGUAGE.into(),
            Language::Java => tree_sitter_java::LANGUAGE.into(),
            Language::Go => tree_sitter_go::LANGUAGE.into(),
            Language::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            Language::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Language::Ruby => tree_sitter_ruby::LANGUAGE.into(),
            Language::CSharp => tree_sitter_c_sharp::LANGUAGE.into(),
            Language::Kotlin => tree_sitter_kotlin_ng::LANGUAGE.into(),
            Language::Scala => tree_sitter_scala::LANGUAGE.into(),
            Language::Swift => tree_sitter_swift::LANGUAGE.into(),
            Language::PHP => tree_sitter_php::LANGUAGE_PHP.into(),
            Language::ObjectiveC => tree_sitter_objc::LANGUAGE.into(),
        }
    }

    pub fn file_extensions(&self) -> &'static [&'static str] {
        match self {
            Language::Rust => &["rs"],
            Language::Python => &["py"],
            Language::C => &["c", "h"],
            Language::Cpp => &["cpp", "cc", "cxx", "hpp", "hxx"],
            Language::Java => &["java"],
            Language::Go => &["go"],
            Language::JavaScript => &["js", "jsx"],
            Language::TypeScript => &["ts", "tsx"],
            Language::Ruby => &["rb"],
            Language::CSharp => &["cs"],
            Language::Kotlin => &["kt", "kts"],
            Language::Scala => &["scala", "sc"],
            Language::Swift => &["swift"],
            Language::PHP => &["php"],
            Language::ObjectiveC => &["m", "mm"],
        }
    }

    pub fn is_source_file(path: &std::path::Path) -> bool {
        Self::from_path(path).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_id_generation() {
        let path = PathBuf::from("test.rs");
        let id1 = SymbolId::new(&path, 1, 0);
        let id2 = SymbolId::new(&path, 1, 0);
        let id3 = SymbolId::new(&path, 2, 0);

        assert_eq!(id1, id2); // Same location should generate same ID
        assert_ne!(id1, id3); // Different location should generate different ID
    }

    #[test]
    fn test_language_detection() {
        assert_eq!(Language::from_extension("rs"), Some(Language::Rust));
        assert_eq!(Language::from_extension("py"), Some(Language::Python));
        assert_eq!(Language::from_extension("js"), Some(Language::JavaScript));
        assert_eq!(Language::from_extension("cs"), Some(Language::CSharp));
        assert_eq!(Language::from_extension("kt"), Some(Language::Kotlin));
        assert_eq!(Language::from_extension("scala"), Some(Language::Scala));
        assert_eq!(Language::from_extension("swift"), Some(Language::Swift));
        assert_eq!(Language::from_extension("php"), Some(Language::PHP));
        assert_eq!(Language::from_extension("m"), Some(Language::ObjectiveC));
        assert_eq!(Language::from_extension("unknown"), None);
    }

    #[test]
    fn test_symbol_type_string_conversion() {
        assert_eq!(SymbolType::Function.as_str(), "function");
        assert_eq!(SymbolType::Class.as_str(), "class");
        assert_eq!(SymbolType::Module.as_str(), "module");
    }

    #[test]
    fn test_serialization() {
        let path = PathBuf::from("test.rs");
        let location = Location::new(path.clone(), 1, 0, 1, 10);
        let symbol_id = SymbolId::new(&path, 1, 0);

        let symbol = Symbol {
            id: symbol_id,
            name: "test_function".to_string(),
            symbol_type: SymbolType::Function,
            location,
            namespace: None,
            visibility: Visibility::Public,
            source: None,
        };

        // Test serialization/deserialization
        let serialized = serde_json::to_string(&symbol).unwrap();
        let deserialized: Symbol = serde_json::from_str(&serialized).unwrap();

        assert_eq!(symbol.name, deserialized.name);
        assert_eq!(symbol.symbol_type, deserialized.symbol_type);
    }
}
