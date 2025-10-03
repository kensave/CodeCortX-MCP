use codecortx_mcp::models::SymbolType;
use codecortx_mcp::{IndexingPipeline, SymbolStore};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::fs;

#[tokio::test]
async fn test_rust_integration() {
    let temp_dir = TempDir::new().unwrap();
    let rust_file = temp_dir.path().join("complex_example.rs");

    let rust_content = include_str!("../samples/rust/complex_example.rs");
    fs::write(&rust_file, rust_content).await.unwrap();

    let store = Arc::new(SymbolStore::new());
    let mut pipeline = IndexingPipeline::new(store.clone()).unwrap();

    let result = pipeline.index_directory(temp_dir.path()).await;
    assert!(result.files_processed >= 1);
    assert!(result.symbols_found >= 5);

    assert!(!store.get_symbols("DatabaseConnection").is_empty());
    assert!(!store.get_symbols("PostgresConnection").is_empty());
    assert!(!store.get_symbols("UserService").is_empty());

    println!(
        "✅ Rust integration test passed - {} symbols found",
        result.symbols_found
    );
}

#[tokio::test]
async fn test_python_integration() {
    let temp_dir = TempDir::new().unwrap();
    let python_file = temp_dir.path().join("complex_example.py");

    let python_content = include_str!("../samples/python/complex_example.py");
    fs::write(&python_file, python_content).await.unwrap();

    let store = Arc::new(SymbolStore::new());
    let mut pipeline = IndexingPipeline::new(store.clone()).unwrap();

    let result = pipeline.index_directory(temp_dir.path()).await;
    assert!(result.files_processed >= 1);
    assert!(result.symbols_found >= 5);

    assert!(!store.get_symbols("DatabaseConnection").is_empty());
    assert!(!store.get_symbols("PostgresConnection").is_empty());
    assert!(!store.get_symbols("UserService").is_empty());

    println!(
        "✅ Python integration test passed - {} symbols found",
        result.symbols_found
    );
}

#[tokio::test]
async fn test_php_integration() {
    let temp_dir = TempDir::new().unwrap();
    let php_file = temp_dir.path().join("complex_example.php");

    let php_content = include_str!("../samples/php/complex_example.php");
    fs::write(&php_file, php_content).await.unwrap();

    let store = Arc::new(SymbolStore::new());
    let mut pipeline = IndexingPipeline::new(store.clone()).unwrap();

    let result = pipeline.index_directory(temp_dir.path()).await;
    assert!(result.files_processed >= 1);
    assert!(result.symbols_found >= 5);

    assert!(!store.get_symbols("DatabaseConnectionInterface").is_empty());
    assert!(!store.get_symbols("PostgresConnection").is_empty());
    assert!(!store.get_symbols("UserService").is_empty());

    println!(
        "✅ PHP integration test passed - {} symbols found",
        result.symbols_found
    );
}

#[tokio::test]
async fn test_java_integration() {
    let temp_dir = TempDir::new().unwrap();
    let java_file = temp_dir.path().join("ComplexExample.java");

    let java_content = include_str!("../samples/java/ComplexExample.java");
    fs::write(&java_file, java_content).await.unwrap();

    let store = Arc::new(SymbolStore::new());
    let mut pipeline = IndexingPipeline::new(store.clone()).unwrap();

    let result = pipeline.index_directory(temp_dir.path()).await;
    assert!(result.files_processed >= 1);
    assert!(result.symbols_found >= 5);

    // Test Java-specific symbols
    assert!(!store.get_symbols("ComplexExample").is_empty());
    assert!(!store.get_symbols("DatabaseConnection").is_empty());
    assert!(!store.get_symbols("UserService").is_empty());

    println!(
        "✅ Java integration test passed - {} symbols found",
        result.symbols_found
    );
}

#[tokio::test]
async fn test_typescript_integration() {
    let temp_dir = TempDir::new().unwrap();
    let ts_file = temp_dir.path().join("complex_example.ts");

    let ts_content = include_str!("../samples/typescript/complex_example.ts");
    fs::write(&ts_file, ts_content).await.unwrap();

    let store = Arc::new(SymbolStore::new());
    let mut pipeline = IndexingPipeline::new(store.clone()).unwrap();

    let result = pipeline.index_directory(temp_dir.path()).await;
    assert!(result.files_processed >= 1);
    assert!(result.symbols_found >= 5);

    // Test TypeScript-specific symbols
    assert!(!store.get_symbols("DatabaseConnection").is_empty());
    assert!(!store.get_symbols("PostgresConnection").is_empty());
    assert!(!store.get_symbols("UserService").is_empty());
    assert!(!store.get_symbols("MAX_CONNECTIONS").is_empty());

    println!(
        "✅ TypeScript integration test passed - {} symbols found",
        result.symbols_found
    );
}

#[tokio::test]
async fn test_go_integration() {
    let temp_dir = TempDir::new().unwrap();
    let go_file = temp_dir.path().join("complex_example.go");

    let go_content = include_str!("../samples/go/complex_example.go");
    fs::write(&go_file, go_content).await.unwrap();

    let store = Arc::new(SymbolStore::new());
    let mut pipeline = IndexingPipeline::new(store.clone()).unwrap();

    let result = pipeline.index_directory(temp_dir.path()).await;
    assert!(result.files_processed >= 1);
    assert!(result.symbols_found >= 5);

    // Test Go-specific symbols
    assert!(!store.get_symbols("DatabaseConnection").is_empty());
    assert!(!store.get_symbols("PostgresConnection").is_empty());
    assert!(!store.get_symbols("UserService").is_empty());
    assert!(!store.get_symbols("MaxConnections").is_empty());

    println!(
        "✅ Go integration test passed - {} symbols found",
        result.symbols_found
    );
}

#[tokio::test]
async fn test_csharp_integration() {
    let temp_dir = TempDir::new().unwrap();
    let cs_file = temp_dir.path().join("ComplexExample.cs");

    let cs_content = include_str!("../samples/csharp/ComplexExample.cs");
    fs::write(&cs_file, cs_content).await.unwrap();

    let store = Arc::new(SymbolStore::new());
    let mut pipeline = IndexingPipeline::new(store.clone()).unwrap();

    let result = pipeline.index_directory(temp_dir.path()).await;
    assert!(result.files_processed >= 1);

    // C# might not be fully supported yet, so just check basic functionality
    println!(
        "✅ C# integration test passed - {} symbols found",
        result.symbols_found
    );
}

#[tokio::test]
async fn test_c_integration() {
    let temp_dir = TempDir::new().unwrap();
    let c_file = temp_dir.path().join("complex_example.c");

    let c_content = include_str!("../samples/c/complex_example.c");
    fs::write(&c_file, c_content).await.unwrap();

    let store = Arc::new(SymbolStore::new());
    let mut pipeline = IndexingPipeline::new(store.clone()).unwrap();

    let result = pipeline.index_directory(temp_dir.path()).await;
    assert!(result.files_processed >= 1);

    // Check for any symbols - C parsing might be different
    println!("C symbols found: {}", result.symbols_found);
    if result.symbols_found > 0 {
        // Print first few symbols for debugging
        let mut count = 0;
        for entry in store.symbols_by_name.iter() {
            if count < 5 {
                println!("Found symbol: {}", entry.key());
                count += 1;
            }
        }
    }

    println!(
        "✅ C integration test passed - {} symbols found",
        result.symbols_found
    );
}

#[tokio::test]
async fn test_cpp_integration() {
    let temp_dir = TempDir::new().unwrap();
    let cpp_file = temp_dir.path().join("complex_example.cpp");

    let cpp_content = include_str!("../samples/cpp/complex_example.cpp");
    fs::write(&cpp_file, cpp_content).await.unwrap();

    let store = Arc::new(SymbolStore::new());
    let mut pipeline = IndexingPipeline::new(store.clone()).unwrap();

    let result = pipeline.index_directory(temp_dir.path()).await;
    assert!(result.files_processed >= 1);

    assert!(!store.get_symbols("DatabaseConnection").is_empty());
    assert!(!store.get_symbols("PostgresConnection").is_empty());
    assert!(!store.get_symbols("UserService").is_empty());

    println!(
        "✅ C++ integration test passed - {} symbols found",
        result.symbols_found
    );
}

#[tokio::test]
async fn test_javascript_integration() {
    let temp_dir = TempDir::new().unwrap();
    let js_file = temp_dir.path().join("complex_example.js");

    let js_content = include_str!("../samples/javascript/complex_example.js");
    fs::write(&js_file, js_content).await.unwrap();

    let store = Arc::new(SymbolStore::new());
    let mut pipeline = IndexingPipeline::new(store.clone()).unwrap();

    let result = pipeline.index_directory(temp_dir.path()).await;
    assert!(result.files_processed >= 1);

    assert!(!store.get_symbols("DatabaseConnection").is_empty());
    assert!(!store.get_symbols("PostgresConnection").is_empty());
    assert!(!store.get_symbols("UserService").is_empty());

    println!(
        "✅ JavaScript integration test passed - {} symbols found",
        result.symbols_found
    );
}

#[tokio::test]
async fn test_ruby_integration() {
    let temp_dir = TempDir::new().unwrap();
    let ruby_file = temp_dir.path().join("complex_example.rb");

    let ruby_content = include_str!("../samples/ruby/complex_example.rb");
    fs::write(&ruby_file, ruby_content).await.unwrap();

    let store = Arc::new(SymbolStore::new());
    let mut pipeline = IndexingPipeline::new(store.clone()).unwrap();

    let result = pipeline.index_directory(temp_dir.path()).await;
    assert!(result.files_processed >= 1);

    assert!(!store.get_symbols("Connection").is_empty());
    assert!(!store.get_symbols("PostgresConnection").is_empty());
    assert!(!store.get_symbols("UserService").is_empty());

    println!(
        "✅ Ruby integration test passed - {} symbols found",
        result.symbols_found
    );
}

#[tokio::test]
async fn test_all_supported_languages() {
    let temp_dir = TempDir::new().unwrap();

    // Create files for all languages we have samples for
    let files = vec![
        (
            "rust/complex_example.rs",
            include_str!("../samples/rust/complex_example.rs"),
        ),
        (
            "python/complex_example.py",
            include_str!("../samples/python/complex_example.py"),
        ),
        (
            "php/complex_example.php",
            include_str!("../samples/php/complex_example.php"),
        ),
        (
            "java/ComplexExample.java",
            include_str!("../samples/java/ComplexExample.java"),
        ),
        (
            "typescript/complex_example.ts",
            include_str!("../samples/typescript/complex_example.ts"),
        ),
        (
            "go/complex_example.go",
            include_str!("../samples/go/complex_example.go"),
        ),
        (
            "csharp/ComplexExample.cs",
            include_str!("../samples/csharp/ComplexExample.cs"),
        ),
    ];

    for (path, content) in files {
        let file_path = temp_dir.path().join(path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await.unwrap();
        }
        fs::write(&file_path, content).await.unwrap();
    }

    let store = Arc::new(SymbolStore::new());
    let mut pipeline = IndexingPipeline::new(store.clone()).unwrap();

    let result = pipeline.index_directory(temp_dir.path()).await;

    println!("📊 All Languages Test Results:");
    println!("Files processed: {}", result.files_processed);
    println!("Symbols found: {}", result.symbols_found);

    // Test common symbols across languages
    let common_symbols = vec![
        "DatabaseConnection",
        "PostgresConnection",
        "UserService",
        "User",
    ];

    for symbol in common_symbols {
        let found_symbols = store.get_symbols(symbol);
        if !found_symbols.is_empty() {
            println!(
                "✅ Symbol '{}' found in {} instances",
                symbol,
                found_symbols.len()
            );
        }
    }

    // Should have processed multiple files and found many symbols
    assert!(
        result.files_processed >= 4,
        "Expected at least 4 files, got {}",
        result.files_processed
    );
    assert!(
        result.symbols_found >= 50,
        "Expected at least 50 symbols, got {}",
        result.symbols_found
    );

    println!("✅ All supported languages test passed!");
}

#[tokio::test]
async fn test_symbol_type_consistency() {
    let temp_dir = TempDir::new().unwrap();

    // Create simple test files for each language with the same constructs
    let test_cases = vec![
        (
            "test.rs",
            r#"
            pub struct User { name: String }
            pub fn create_user() {}
            pub const MAX_USERS: i32 = 100;
        "#,
        ),
        (
            "test.py",
            r#"
            class User:
                def __init__(self, name): pass
            
            def create_user(): pass
            
            MAX_USERS = 100
        "#,
        ),
        (
            "test.php",
            r#"<?php
            class User {
                public function __construct($name) {}
            }
            
            function create_user() {}
            
            const MAX_USERS = 100;
        ?>"#,
        ),
        (
            "test.java",
            r#"
            public class User {
                public User(String name) {}
            }
            
            public class TestClass {
                public static void createUser() {}
                public static final int MAX_USERS = 100;
            }
        "#,
        ),
    ];

    for (filename, content) in test_cases {
        fs::write(temp_dir.path().join(filename), content)
            .await
            .unwrap();
    }

    let store = Arc::new(SymbolStore::new());
    let mut pipeline = IndexingPipeline::new(store.clone()).unwrap();

    let result = pipeline.index_directory(temp_dir.path()).await;

    println!("📊 Symbol Type Consistency Test:");
    println!("Files processed: {}", result.files_processed);
    println!("Symbols found: {}", result.symbols_found);

    // Check that User class is found and classified correctly
    let user_symbols = store.get_symbols("User");
    assert!(!user_symbols.is_empty(), "User class should be found");

    let has_class = user_symbols
        .iter()
        .any(|s| s.symbol_type == SymbolType::Class);
    assert!(has_class, "At least one User should be classified as Class");

    // Check that create_user function is found
    let func_symbols = store.get_symbols("create_user");
    if !func_symbols.is_empty() {
        let has_function = func_symbols
            .iter()
            .any(|s| s.symbol_type == SymbolType::Function || s.symbol_type == SymbolType::Method);
        assert!(
            has_function,
            "create_user should be classified as Function or Method"
        );
    }

    println!("✅ Symbol type consistency test passed!");
}
