use codecortx_mcp::models::SymbolType;
use codecortx_mcp::{IndexingPipeline, SymbolStore};
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::fs;

#[tokio::test]
async fn test_get_file_outline() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rs");

    // Create test file with various symbol types
    let content = r#"
pub struct User {
    pub id: u32,
    name: String,
}

impl User {
    pub fn new(id: u32, name: String) -> Self {
        Self { id, name }
    }
    
    fn get_name(&self) -> &str {
        &self.name
    }
}

pub const MAX_USERS: u32 = 1000;

pub enum Status {
    Active,
    Inactive,
}

pub fn create_user() -> User {
    User::new(1, "test".to_string())
}
"#;

    fs::write(&test_file, content).await.unwrap();

    // Index the file
    let store = Arc::new(SymbolStore::new());
    let mut pipeline = IndexingPipeline::new(store.clone()).unwrap();
    let result = pipeline.index_directory(temp_dir.path()).await;
    assert!(result.files_processed > 0);

    // Get file outline
    let symbols = store.get_symbols_by_file(&test_file);

    // Verify we have the expected symbols
    let symbol_names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
    assert!(symbol_names.contains(&"User"));
    assert!(symbol_names.contains(&"new"));
    assert!(symbol_names.contains(&"get_name"));
    assert!(symbol_names.contains(&"MAX_USERS"));
    assert!(symbol_names.contains(&"Status"));
    assert!(symbol_names.contains(&"create_user"));

    // Verify symbol types
    let structs: Vec<_> = symbols
        .iter()
        .filter(|s| matches!(s.symbol_type, SymbolType::Struct))
        .collect();
    let functions: Vec<_> = symbols
        .iter()
        .filter(|s| matches!(s.symbol_type, SymbolType::Function | SymbolType::Method))
        .collect();
    let constants: Vec<_> = symbols
        .iter()
        .filter(|s| matches!(s.symbol_type, SymbolType::Constant))
        .collect();
    let enums: Vec<_> = symbols
        .iter()
        .filter(|s| matches!(s.symbol_type, SymbolType::Enum))
        .collect();

    assert_eq!(structs.len(), 1);
    assert!(functions.len() >= 3); // new, get_name, create_user
    assert_eq!(constants.len(), 1);
    assert_eq!(enums.len(), 1);
}

#[tokio::test]
async fn test_get_directory_outline() {
    let temp_dir = TempDir::new().unwrap();

    // Create directory structure
    let models_dir = temp_dir.path().join("models");
    let services_dir = temp_dir.path().join("services");
    fs::create_dir_all(&models_dir).await.unwrap();
    fs::create_dir_all(&services_dir).await.unwrap();

    // Create model files
    let user_model = models_dir.join("user.rs");
    fs::write(
        &user_model,
        r#"
pub struct User {
    pub id: u32,
}

pub enum UserRole {
    Admin,
    User,
}
"#,
    )
    .await
    .unwrap();

    let product_model = models_dir.join("product.rs");
    fs::write(
        &product_model,
        r#"
pub struct Product {
    pub id: u32,
}
"#,
    )
    .await
    .unwrap();

    // Create service files
    let user_service = services_dir.join("user_service.rs");
    fs::write(
        &user_service,
        r#"
pub struct UserService {
    db: Database,
}

impl UserService {
    pub fn new() -> Self {
        Self { db: Database::new() }
    }
    
    pub fn create_user(&self) -> User {
        User { id: 1 }
    }
}
"#,
    )
    .await
    .unwrap();

    // Index all files
    let store = Arc::new(SymbolStore::new());
    let mut pipeline = IndexingPipeline::new(store.clone()).unwrap();
    let result = pipeline.index_directory(temp_dir.path()).await;
    assert!(result.files_processed > 0);

    // Test directory outline - classes only (default)
    let mut dir_symbols = std::collections::BTreeMap::new();

    for entry in store.symbol_data.iter() {
        let symbol = entry.value();
        if let Some(parent) = symbol.location.file.parent() {
            if parent.starts_with(temp_dir.path()) {
                // Only include classes by default
                if matches!(
                    symbol.symbol_type,
                    SymbolType::Class | SymbolType::Struct | SymbolType::Enum
                ) {
                    let rel_path = parent.strip_prefix(temp_dir.path()).unwrap_or(parent);
                    dir_symbols
                        .entry(rel_path.to_path_buf())
                        .or_insert_with(Vec::new)
                        .push(symbol.name.clone());
                }
            }
        }
    }

    // Verify structure
    assert!(dir_symbols.contains_key(&PathBuf::from("models")));
    assert!(dir_symbols.contains_key(&PathBuf::from("services")));

    let models_symbols = &dir_symbols[&PathBuf::from("models")];
    assert!(models_symbols.contains(&"User".to_string()));
    assert!(models_symbols.contains(&"UserRole".to_string()));
    assert!(models_symbols.contains(&"Product".to_string()));

    let services_symbols = &dir_symbols[&PathBuf::from("services")];
    assert!(services_symbols.contains(&"UserService".to_string()));

    // Test with includes - should also have methods
    let mut dir_symbols_with_methods = std::collections::BTreeMap::new();
    let includes = vec!["methods".to_string()];

    for entry in store.symbol_data.iter() {
        let symbol = entry.value();
        if let Some(parent) = symbol.location.file.parent() {
            if parent.starts_with(temp_dir.path()) {
                let should_include = match symbol.symbol_type {
                    SymbolType::Class | SymbolType::Struct | SymbolType::Enum => true,
                    SymbolType::Method | SymbolType::Function => {
                        includes.contains(&"methods".to_string())
                    }
                    _ => false,
                };

                if should_include {
                    let rel_path = parent.strip_prefix(temp_dir.path()).unwrap_or(parent);
                    dir_symbols_with_methods
                        .entry(rel_path.to_path_buf())
                        .or_insert_with(Vec::new)
                        .push(symbol.name.clone());
                }
            }
        }
    }

    let services_with_methods = &dir_symbols_with_methods[&PathBuf::from("services")];
    assert!(services_with_methods.contains(&"UserService".to_string()));
    assert!(services_with_methods.contains(&"new".to_string()));
    assert!(services_with_methods.contains(&"create_user".to_string()));
}

#[tokio::test]
async fn test_outline_empty_file() {
    let temp_dir = TempDir::new().unwrap();
    let empty_file = temp_dir.path().join("empty.rs");
    fs::write(&empty_file, "").await.unwrap();

    let store = Arc::new(SymbolStore::new());
    let mut pipeline = IndexingPipeline::new(store.clone()).unwrap();
    let result = pipeline.index_directory(temp_dir.path()).await;
    assert_eq!(result.files_processed, 1);

    let symbols = store.get_symbols_by_file(&empty_file);
    assert!(symbols.is_empty());
}

#[tokio::test]
async fn test_outline_nonexistent_file() {
    let store = Arc::new(SymbolStore::new());
    let nonexistent = PathBuf::from("/nonexistent/file.rs");

    let symbols = store.get_symbols_by_file(&nonexistent);
    assert!(symbols.is_empty());
}
