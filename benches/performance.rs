use roberto_mcp::models::{Language, Location, Symbol, SymbolId, SymbolType, Visibility};
use roberto_mcp::{SymbolIndexer, SymbolStore};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
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

fn bench_symbol_lookup(c: &mut Criterion) {
    let store = Arc::new(SymbolStore::new());

    // Insert 10,000 symbols
    for i in 0..10_000 {
        let symbol = create_test_symbol(&format!("function_{}", i), "test.rs", i + 1);
        store.insert_symbol_unchecked(symbol);
    }

    // Benchmark O(1) symbol lookup
    c.bench_function("symbol_lookup_o1", |b| {
        b.iter(|| {
            let symbols = store.get_symbols(black_box("function_5000"));
            black_box(symbols);
        })
    });

    // Target: <1ms lookups
    let mut group = c.benchmark_group("symbol_lookup_performance");
    group.significance_level(0.1).sample_size(1000);

    for size in [100, 1_000, 10_000, 50_000].iter() {
        group.bench_with_input(BenchmarkId::new("lookup_time", size), size, |b, &size| {
            let store = Arc::new(SymbolStore::new());

            // Insert symbols
            for i in 0..*size {
                let symbol = create_test_symbol(&format!("func_{}", i), "test.rs", i + 1);
                store.insert_symbol_unchecked(symbol);
            }

            b.iter(|| {
                let target = format!("func_{}", size / 2);
                let symbols = store.get_symbols(black_box(&target));
                black_box(symbols);
            });
        });
    }
    group.finish();
}

fn bench_symbol_insertion(c: &mut Criterion) {
    let mut group = c.benchmark_group("symbol_insertion");
    group.significance_level(0.1).sample_size(100);

    for size in [100, 1_000, 5_000].iter() {
        group.bench_with_input(BenchmarkId::new("batch_insert", size), size, |b, &size| {
            b.iter_with_setup(
                || {
                    let store = Arc::new(SymbolStore::new());
                    let mut symbols = Vec::new();
                    for i in 0..*size {
                        symbols.push(create_test_symbol(&format!("func_{}", i), "test.rs", i + 1));
                    }
                    (store, symbols)
                },
                |(store, symbols)| {
                    for symbol in symbols {
                        store.insert_symbol_unchecked(black_box(symbol));
                    }
                },
            );
        });
    }
    group.finish();
}

fn bench_prefix_search(c: &mut Criterion) {
    let store = Arc::new(SymbolStore::new());

    // Insert symbols with different prefixes
    for i in 0..1_000 {
        let symbol = create_test_symbol(&format!("test_function_{}", i), "test.rs", i + 1);
        store.insert_symbol_unchecked(symbol);
    }
    for i in 0..1_000 {
        let symbol = create_test_symbol(&format!("other_function_{}", i), "test.rs", i + 1001);
        store.insert_symbol_unchecked(symbol);
    }
    for i in 0..1_000 {
        let symbol = create_test_symbol(&format!("helper_function_{}", i), "test.rs", i + 2001);
        store.insert_symbol_unchecked(symbol);
    }

    c.bench_function("prefix_search", |b| {
        b.iter(|| {
            let results = store.find_symbols_by_prefix(black_box("test_"));
            black_box(results);
        })
    });
}

fn bench_file_indexing_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_indexing_simulation");
    group.significance_level(0.1).sample_size(50);

    // Simulate indexing different numbers of files
    for file_count in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("files", file_count),
            file_count,
            |b, &file_count| {
                b.iter_with_setup(
                    || {
                        let mut indexer = SymbolIndexer::new().unwrap();
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
                        "#;
                        (indexer, rust_code)
                    },
                    |(mut indexer, rust_code)| {
                        let start = Instant::now();

                        for i in 0..*file_count {
                            let file_path = PathBuf::from(format!("test_{}.rs", i));
                            let _symbols = indexer
                                .extract_symbols(
                                    black_box(rust_code),
                                    Language::Rust,
                                    black_box(&file_path),
                                )
                                .unwrap();
                        }

                        let duration = start.elapsed();

                        // Target: >100 files/sec
                        let files_per_second = (*file_count as f64) / duration.as_secs_f64();

                        // This is measured but not asserted in the benchmark
                        black_box(files_per_second);
                    },
                );
            },
        );
    }
    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_tracking");

    group.bench_function("memory_tracking_overhead", |b| {
        b.iter_with_setup(
            || {
                let store = Arc::new(SymbolStore::new());
                let symbol = create_test_symbol("test_function", "test.rs", 1);
                (store, symbol)
            },
            |(store, symbol)| {
                let initial_memory = store.get_memory_usage();
                store.insert_symbol_unchecked(black_box(symbol));
                let final_memory = store.get_memory_usage();
                black_box(final_memory - initial_memory);
            },
        );
    });

    group.finish();
}

fn bench_concurrent_access(c: &mut Criterion) {
    use std::sync::Arc;
    use std::thread;

    let store = Arc::new(SymbolStore::new());

    // Pre-populate with symbols
    for i in 0..1_000 {
        let symbol = create_test_symbol(&format!("concurrent_func_{}", i), "test.rs", i + 1);
        store.insert_symbol_unchecked(symbol);
    }

    c.bench_function("concurrent_reads", |b| {
        b.iter(|| {
            let store_clone = Arc::clone(&store);
            let handles: Vec<_> = (0..4)
                .map(|thread_id| {
                    let store = Arc::clone(&store_clone);
                    thread::spawn(move || {
                        for i in 0..100 {
                            let target =
                                format!("concurrent_func_{}", (thread_id * 100 + i) % 1000);
                            let _symbols = store.get_symbols(&target);
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
        })
    });
}

criterion_group!(
    benches,
    bench_symbol_lookup,
    bench_symbol_insertion,
    bench_prefix_search,
    bench_file_indexing_simulation,
    bench_memory_usage,
    bench_concurrent_access
);

criterion_main!(benches);
