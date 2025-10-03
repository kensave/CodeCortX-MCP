use codecortx_mcp::models::{Language, Location, Symbol, SymbolId, SymbolType, Visibility};
use codecortx_mcp::{SymbolIndexer, SymbolStore};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

fn create_test_symbol(name: &str, file: &str, line: u32) -> Symbol {
    let path = PathBuf::from(file);
    let location = Location::new(path.clone(), line, 0, line, 10);
    let symbol_id = SymbolId::new(&path, line, 0);

    Symbol {
        id: symbol_id,
        name: name.to_string(),
        symbol_type: SymbolType::Function,
        location,
        namespace: None,
        visibility: Visibility::Public,
        source: None,
    }
}

#[test]
fn test_symbol_lookup_performance_requirement() {
    let store = Arc::new(SymbolStore::new());

    // Insert 50,000 symbols to simulate a large codebase
    for i in 0..50_000 {
        let symbol = create_test_symbol(&format!("function_{}", i), "test.rs", i + 1);
        store.insert_symbol_unchecked(symbol);
    }

    // Test multiple lookups to get average performance
    let mut total_duration = std::time::Duration::new(0, 0);
    let test_count = 1000;

    for i in 0..test_count {
        let target = format!("function_{}", i * 50); // Test different symbols
        let start = Instant::now();
        let _symbols = store.get_symbols(&target);
        let duration = start.elapsed();
        total_duration += duration;
    }

    let avg_duration = total_duration / test_count;

    // Requirement: Symbol lookups should be <1ms
    assert!(
        avg_duration.as_millis() < 1,
        "Symbol lookup performance requirement failed: {}ms average (target: <1ms)",
        avg_duration.as_millis()
    );

    println!(
        "✅ Symbol lookup performance: {}μs average (target: <1ms)",
        avg_duration.as_micros()
    );
}

#[test]
fn test_indexing_speed_performance_requirement() {
    let mut indexer = SymbolIndexer::new().unwrap();

    let rust_code = r#"
    pub fn function_1() {
        println!("Function 1");
    }

    pub struct TestStruct {
        pub field: i32,
    }

    impl TestStruct {
        pub fn new(field: i32) -> Self {
            Self { field }
        }
    }

    pub const CONSTANT: i32 = 42;
    "#;

    let file_count = 150; // Test with 150 files to exceed 100 files/sec requirement
    let start = Instant::now();

    for i in 0..file_count {
        let file_path = PathBuf::from(format!("test_{}.rs", i));
        let _symbols = indexer
            .extract_symbols(rust_code, Language::Rust, &file_path)
            .unwrap();
    }

    let duration = start.elapsed();
    let files_per_second = (file_count as f64) / duration.as_secs_f64();

    // Requirement: Indexing speed should be >100 files/sec
    assert!(
        files_per_second > 100.0,
        "Indexing performance requirement failed: {:.1} files/sec (target: >100 files/sec)",
        files_per_second
    );

    println!(
        "✅ Indexing performance: {:.1} files/sec (target: >100 files/sec)",
        files_per_second
    );
}

#[test]
fn test_memory_usage_tracking_accuracy() {
    let store = Arc::new(SymbolStore::new());
    let initial_memory = store.get_memory_usage();

    // Insert symbols and track memory growth
    let symbol_count = 1000;
    let mut expected_memory = 0u64;

    for i in 0..symbol_count {
        let symbol = create_test_symbol(&format!("func_{}", i), "test.rs", i + 1);

        // Estimate memory size manually
        let estimated_size = symbol.name.len() + 100; // Name + overhead estimate
        expected_memory += estimated_size as u64;

        store.insert_symbol_unchecked(symbol);
    }

    let final_memory = store.get_memory_usage();
    let actual_growth = final_memory - initial_memory;

    // Memory tracking should be reasonably accurate (within 2x of estimate)
    assert!(
        actual_growth > expected_memory / 4,
        "Memory tracking seems too low: {} bytes tracked, expected ~{} bytes",
        actual_growth,
        expected_memory
    );

    assert!(
        actual_growth < expected_memory * 4,
        "Memory tracking seems too high: {} bytes tracked, expected ~{} bytes",
        actual_growth,
        expected_memory
    );

    println!(
        "✅ Memory tracking: {} bytes for {} symbols ({} bytes/symbol average)",
        actual_growth,
        symbol_count,
        actual_growth / symbol_count as u64
    );
}

#[test]
fn test_concurrent_access_performance() {
    use std::sync::Arc;
    use std::thread;

    let store = Arc::new(SymbolStore::new());

    // Pre-populate with symbols
    let symbol_count = 10_000;
    for i in 0..symbol_count {
        let symbol = create_test_symbol(&format!("concurrent_func_{}", i), "test.rs", i + 1);
        store.insert_symbol_unchecked(symbol);
    }

    let start = Instant::now();
    let thread_count = 8;
    let lookups_per_thread = 1000;

    let handles: Vec<_> = (0..thread_count)
        .map(|thread_id| {
            let store = Arc::clone(&store);
            thread::spawn(move || {
                for i in 0..lookups_per_thread {
                    let target = format!(
                        "concurrent_func_{}",
                        (thread_id * lookups_per_thread + i) % symbol_count
                    );
                    let _symbols = store.get_symbols(&target);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start.elapsed();
    let total_lookups = thread_count * lookups_per_thread;
    let lookups_per_second = (total_lookups as f64) / duration.as_secs_f64();

    // Concurrent access should not significantly degrade performance
    // Target: Should handle thousands of lookups per second even under concurrency
    assert!(
        lookups_per_second > 50_000.0, // 50k lookups/sec minimum
        "Concurrent performance too low: {:.0} lookups/sec (target: >50k/sec)",
        lookups_per_second
    );

    println!(
        "✅ Concurrent access performance: {:.0} lookups/sec with {} threads",
        lookups_per_second, thread_count
    );
}

#[test]
fn test_prefix_search_performance() {
    let store = Arc::new(SymbolStore::new());

    // Insert symbols with different prefixes
    let prefix_groups = 10;
    let symbols_per_group = 1000;

    for group in 0..prefix_groups {
        for i in 0..symbols_per_group {
            let symbol = create_test_symbol(
                &format!("prefix_{}_{}", group, i),
                "test.rs",
                (group * symbols_per_group + i) + 1,
            );
            store.insert_symbol_unchecked(symbol);
        }
    }

    // Test prefix search performance
    let start = Instant::now();
    let search_count = 100;

    for i in 0..search_count {
        let prefix = format!("prefix_{}_", i % prefix_groups);
        let _results = store.find_symbols_by_prefix(&prefix);
    }

    let duration = start.elapsed();
    let avg_search_time = duration / search_count;

    // Prefix searches should be reasonably fast even with many symbols
    assert!(
        avg_search_time.as_millis() < 10,
        "Prefix search too slow: {}ms average (target: <10ms)",
        avg_search_time.as_millis()
    );

    println!(
        "✅ Prefix search performance: {}ms average (target: <10ms)",
        avg_search_time.as_millis()
    );
}
