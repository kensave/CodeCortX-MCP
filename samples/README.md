# Code Samples for Testing

This directory contains complex, real-world code samples for testing the CodeCortext MCP Server across all supported languages. These samples are designed to test comprehensive symbol extraction and ensure consistent behavior across languages.

## Purpose

These samples are used by millions of developers through the CodeCortext MCP Server, so they must be:
- **Comprehensive**: Cover all major language constructs
- **Realistic**: Represent real-world code patterns
- **Consistent**: Test the same functionality across languages
- **Complex**: Include multiple classes, inheritance, generics, etc.

## Language Coverage

### Currently Implemented
- **Rust** (`rust/complex_example.rs`): Traits, generics, modules, async functions
- **Python** (`python/complex_example.py`): Classes, decorators, async/await, type hints
- **PHP** (`php/complex_example.php`): Interfaces, traits, namespaces, strict types
- **Objective-C** (`objc/complex_example.m`): Protocols, categories, memory management

### Planned
- **Java**: Generics, annotations, packages
- **TypeScript**: Interfaces, generics, modules, decorators
- **Go**: Interfaces, goroutines, packages
- **C++**: Templates, namespaces, classes
- **JavaScript**: Classes, modules, async/await
- **C#**: Generics, LINQ, namespaces
- **Kotlin**: Data classes, coroutines, extensions
- **Scala**: Case classes, traits, pattern matching
- **Swift**: Protocols, extensions, generics
- **Ruby**: Modules, mixins, metaprogramming

## Sample Structure

Each language sample includes:

### Core Constructs
- **Classes/Structs**: Multiple classes with inheritance
- **Interfaces/Traits**: Abstract contracts and implementations
- **Functions/Methods**: Static and instance methods
- **Variables/Constants**: Module-level and class-level
- **Modules/Namespaces**: Organizational structures

### Advanced Features
- **Generics/Templates**: Type parameters and constraints
- **Async/Concurrency**: Async functions, promises, futures
- **Error Handling**: Exception types and error propagation
- **Decorators/Annotations**: Metadata and aspect-oriented programming
- **Memory Management**: RAII, smart pointers, garbage collection

### Real-World Patterns
- **Database Connections**: Abstract interfaces with multiple implementations
- **Service Layer**: Business logic with dependency injection
- **Data Models**: User, Product, and other domain objects
- **Configuration**: Application settings and environment handling
- **Logging**: Structured logging with different levels

## Testing Integration

These samples are used by:

1. **Unit Tests**: Individual language parsing tests
2. **Integration Tests**: Cross-language consistency tests
3. **Performance Tests**: Large-scale indexing benchmarks
4. **Regression Tests**: Ensuring no functionality breaks

## Usage

### Running Tests
```bash
# Test all languages comprehensively
cargo test --test comprehensive_language_test

# Test specific language features
cargo test --test comprehensive_language_test test_language_specific_features

# Performance testing with samples
cargo test --test performance_validation
```

### Adding New Languages

1. Create directory: `samples/{language}/`
2. Add complex example: `samples/{language}/complex_example.{ext}`
3. Include in `comprehensive_language_test.rs`
4. Update this README

### Sample Requirements

Each language sample must include:
- [ ] At least 3 classes/structs
- [ ] At least 2 interfaces/traits
- [ ] At least 5 functions/methods
- [ ] At least 3 constants/variables
- [ ] At least 1 module/namespace
- [ ] Generic/template usage
- [ ] Inheritance or composition
- [ ] Error handling patterns
- [ ] Real-world naming conventions

## Quality Assurance

All samples are validated for:
- **Syntax Correctness**: Must compile/parse without errors
- **Symbol Coverage**: Must generate expected symbol types
- **Consistency**: Same patterns across languages
- **Complexity**: Sufficient depth for real-world testing
- **Performance**: Reasonable parsing time and memory usage

## Contributing

When adding or modifying samples:

1. Ensure they represent real-world code patterns
2. Test with the comprehensive test suite
3. Verify symbol extraction works correctly
4. Update documentation and test expectations
5. Consider performance impact on indexing

These samples are critical infrastructure for ensuring CodeCortext works reliably for millions of developers worldwide.
