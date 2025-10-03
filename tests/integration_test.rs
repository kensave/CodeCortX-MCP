use codecortx_mcp::models::SymbolType;
use codecortx_mcp::{IndexingPipeline, SymbolStore};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::fs;

#[tokio::test]
async fn test_end_to_end_rust_indexing() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rs");

    let rust_code = r#"
pub fn hello_world() {
    println!("Hello, world!");
}

pub struct TestStruct {
    pub field: i32,
}

impl TestStruct {
    pub fn new(field: i32) -> Self {
        Self { field }
    }
}

pub const TEST_CONST: i32 = 42;
"#;

    fs::write(&test_file, rust_code).await.unwrap();

    let store = Arc::new(SymbolStore::new());
    let mut pipeline = IndexingPipeline::new(store.clone()).unwrap();

    // Index the file
    let result = pipeline.index_directory(temp_dir.path()).await;
    assert!(result.files_processed > 0);
    assert!(result.symbols_found > 0);

    // Verify symbols were extracted
    let symbols = store.get_symbols("hello_world");
    assert!(!symbols.is_empty());
    assert_eq!(symbols[0].symbol_type, SymbolType::Function);

    let struct_symbols = store.get_symbols("TestStruct");
    assert!(!struct_symbols.is_empty());
    assert_eq!(struct_symbols[0].symbol_type, SymbolType::Struct);

    let const_symbols = store.get_symbols("TEST_CONST");
    assert!(!const_symbols.is_empty());
    assert_eq!(const_symbols[0].symbol_type, SymbolType::Constant);
}

#[tokio::test]
async fn test_end_to_end_python_indexing() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.py");

    let python_code = r#"
def hello_world():
    print("Hello, world!")

class TestClass:
    def __init__(self, value):
        self.value = value
    
    def get_value(self):
        return self.value

TEST_CONSTANT = 42
"#;

    fs::write(&test_file, python_code).await.unwrap();

    let store = Arc::new(SymbolStore::new());
    let mut pipeline = IndexingPipeline::new(store.clone()).unwrap();

    let result = pipeline.index_directory(temp_dir.path()).await;
    assert!(result.files_processed > 0);

    let func_symbols = store.get_symbols("hello_world");
    assert!(!func_symbols.is_empty());
    assert_eq!(func_symbols[0].symbol_type, SymbolType::Function);

    let class_symbols = store.get_symbols("TestClass");
    assert!(!class_symbols.is_empty());
    assert_eq!(class_symbols[0].symbol_type, SymbolType::Class);

    let const_symbols = store.get_symbols("TEST_CONSTANT");
    assert!(!const_symbols.is_empty());
    assert_eq!(const_symbols[0].symbol_type, SymbolType::Variable);
}

#[tokio::test]
async fn test_php_symbol_extraction() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.php");

    let php_code = r#"<?php
class Calculator {
    public function add($a, $b) {
        return $a + $b;
    }
}

interface UserInterface {
    public function getName();
}

function global_function() {
    return "test";
}
?>"#;

    fs::write(&test_file, php_code).await.unwrap();

    let store = Arc::new(SymbolStore::new());
    let mut pipeline = IndexingPipeline::new(store.clone()).unwrap();

    let result = pipeline.index_directory(temp_dir.path()).await;
    assert!(result.files_processed > 0);

    let class_symbols = store.get_symbols("Calculator");
    assert!(!class_symbols.is_empty());
    assert_eq!(class_symbols[0].symbol_type, SymbolType::Class);

    let interface_symbols = store.get_symbols("UserInterface");
    assert!(!interface_symbols.is_empty());
    assert_eq!(interface_symbols[0].symbol_type, SymbolType::Interface);

    let func_symbols = store.get_symbols("global_function");
    assert!(!func_symbols.is_empty());
    assert_eq!(func_symbols[0].symbol_type, SymbolType::Function);
}

#[tokio::test]
async fn test_objective_c_symbol_extraction() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.m");

    let objc_code = r#"#import <Foundation/Foundation.h>

@interface Calculator : NSObject
- (int)add:(int)a to:(int)b;
@end

@implementation Calculator
- (int)add:(int)a to:(int)b {
    return a + b;
}
@end

void global_function() {
    NSLog(@"Hello");
}
"#;

    fs::write(&test_file, objc_code).await.unwrap();

    let store = Arc::new(SymbolStore::new());
    let mut pipeline = IndexingPipeline::new(store.clone()).unwrap();

    let result = pipeline.index_directory(temp_dir.path()).await;
    assert!(result.files_processed > 0);

    let class_symbols = store.get_symbols("Calculator");
    assert!(!class_symbols.is_empty());
    assert_eq!(class_symbols[0].symbol_type, SymbolType::Class);

    let func_symbols = store.get_symbols("global_function");
    assert!(!func_symbols.is_empty());
    assert_eq!(func_symbols[0].symbol_type, SymbolType::Function);
}

#[tokio::test]
async fn test_multi_language_repository() {
    let temp_dir = TempDir::new().unwrap();

    // Create Rust file
    let rust_file = temp_dir.path().join("main.rs");
    fs::write(&rust_file, "fn rust_function() {}")
        .await
        .unwrap();

    // Create Python file
    let python_file = temp_dir.path().join("script.py");
    fs::write(&python_file, "def python_function(): pass")
        .await
        .unwrap();

    // Create PHP file
    let php_file = temp_dir.path().join("index.php");
    fs::write(&php_file, "<?php function php_function() {} ?>")
        .await
        .unwrap();

    let store = Arc::new(SymbolStore::new());
    let mut pipeline = IndexingPipeline::new(store.clone()).unwrap();

    let result = pipeline.index_directory(temp_dir.path()).await;
    assert!(result.files_processed >= 3);

    // Verify all languages were indexed
    assert!(!store.get_symbols("rust_function").is_empty());
    assert!(!store.get_symbols("python_function").is_empty());
    assert!(!store.get_symbols("php_function").is_empty());
}
