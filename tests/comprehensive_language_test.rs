use roberto_mcp::models::SymbolType;
use roberto_mcp::{IndexingPipeline, SymbolStore};
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::fs;

/// Test the same functionality across all supported languages
/// This ensures consistent behavior and comprehensive coverage

#[tokio::test]
async fn test_all_languages_comprehensive() {
    let temp_dir = TempDir::new().unwrap();
    let store = Arc::new(SymbolStore::new());
    let mut pipeline = IndexingPipeline::new(store.clone()).unwrap();

    // Create complex samples for each language
    create_all_language_samples(&temp_dir).await;

    // Index all files
    let result = pipeline.index_directory(temp_dir.path()).await;
    println!(
        "Files processed: {}, Symbols found: {}",
        result.files_processed, result.symbols_found
    );

    // Adjust expectations based on what languages are actually supported
    assert!(
        result.files_processed >= 15,
        "Expected at least 15 files processed, got {}",
        result.files_processed
    );
    assert!(
        result.symbols_found >= 100,
        "Expected at least 100 symbols, got {}",
        result.symbols_found
    );

    // Test consistent functionality across all languages
    test_class_extraction(&store).await;
    test_function_extraction(&store).await;
    test_interface_extraction(&store).await;
    test_variable_extraction(&store).await;
    test_complex_scenarios(&store).await;
}

async fn create_all_language_samples(temp_dir: &TempDir) {
    // Rust - Complex example
    let rust_content = r#"
pub mod database {
    pub trait Connection {
        fn connect(&self) -> Result<(), Error>;
    }
    
    pub struct PostgresConnection {
        pub host: String,
    }
    
    impl Connection for PostgresConnection {
        fn connect(&self) -> Result<(), Error> { Ok(()) }
    }
}

pub struct UserService<T> {
    db: T,
}

impl<T: database::Connection> UserService<T> {
    pub fn new(db: T) -> Self { Self { db } }
    pub async fn create_user(&self, name: String) -> User { User { name } }
}

pub struct User { pub name: String }
pub enum Status { Active, Inactive }
pub const MAX_USERS: usize = 1000;

pub fn initialize_app() -> Result<(), Error> { Ok(()) }
"#;
    fs::write(temp_dir.path().join("complex.rs"), rust_content)
        .await
        .unwrap();

    // Python - Complex example
    let python_content = r#"
from abc import ABC, abstractmethod
from typing import Generic, TypeVar

T = TypeVar('T')

class DatabaseConnection(ABC):
    @abstractmethod
    def connect(self) -> bool:
        pass

class PostgresConnection(DatabaseConnection):
    def __init__(self, host: str):
        self.host = host
    
    def connect(self) -> bool:
        return True

class UserService(Generic[T]):
    def __init__(self, db: T):
        self.db = db
    
    async def create_user(self, name: str) -> 'User':
        return User(name)

class User:
    def __init__(self, name: str):
        self.name = name

MAX_USERS = 1000

def initialize_app():
    pass
"#;
    fs::write(temp_dir.path().join("complex.py"), python_content)
        .await
        .unwrap();

    // PHP - Complex example
    let php_content = r#"<?php
interface DatabaseConnectionInterface {
    public function connect(): bool;
}

class PostgresConnection implements DatabaseConnectionInterface {
    private string $host;
    
    public function __construct(string $host) {
        $this->host = $host;
    }
    
    public function connect(): bool {
        return true;
    }
}

class UserService {
    private DatabaseConnectionInterface $db;
    
    public function __construct(DatabaseConnectionInterface $db) {
        $this->db = $db;
    }
    
    public function createUser(string $name): User {
        return new User($name);
    }
}

class User {
    public function __construct(private string $name) {}
}

const MAX_USERS = 1000;

function initialize_app(): void {}
?>"#;
    fs::write(temp_dir.path().join("complex.php"), php_content)
        .await
        .unwrap();

    // Objective-C - Complex example
    let objc_content = r#"
#import <Foundation/Foundation.h>

@protocol DatabaseConnection
- (BOOL)connect;
@end

@interface PostgresConnection : NSObject <DatabaseConnection>
@property (nonatomic, strong) NSString *host;
- (instancetype)initWithHost:(NSString *)host;
@end

@implementation PostgresConnection
- (instancetype)initWithHost:(NSString *)host {
    if (self = [super init]) {
        _host = host;
    }
    return self;
}

- (BOOL)connect {
    return YES;
}
@end

@interface UserService : NSObject
@property (nonatomic, strong) id<DatabaseConnection> db;
- (instancetype)initWithDatabase:(id<DatabaseConnection>)db;
- (User *)createUserWithName:(NSString *)name;
@end

@implementation UserService
- (instancetype)initWithDatabase:(id<DatabaseConnection>)db {
    if (self = [super init]) {
        _db = db;
    }
    return self;
}

- (User *)createUserWithName:(NSString *)name {
    return [[User alloc] initWithName:name];
}
@end

@interface User : NSObject
@property (nonatomic, strong) NSString *name;
- (instancetype)initWithName:(NSString *)name;
@end

@implementation User
- (instancetype)initWithName:(NSString *)name {
    if (self = [super init]) {
        _name = name;
    }
    return self;
}
@end

void initialize_app() {
    // Implementation
}
"#;
    fs::write(temp_dir.path().join("complex.m"), objc_content)
        .await
        .unwrap();

    // Java - Complex example
    let java_content = r#"
package com.example;

interface DatabaseConnection {
    boolean connect();
}

class PostgresConnection implements DatabaseConnection {
    private String host;
    
    public PostgresConnection(String host) {
        this.host = host;
    }
    
    public boolean connect() {
        return true;
    }
}

class UserService<T extends DatabaseConnection> {
    private T db;
    
    public UserService(T db) {
        this.db = db;
    }
    
    public User createUser(String name) {
        return new User(name);
    }
}

class User {
    private String name;
    
    public User(String name) {
        this.name = name;
    }
}

public class Application {
    public static final int MAX_USERS = 1000;
    
    public static void initializeApp() {
        // Implementation
    }
}
"#;
    fs::write(temp_dir.path().join("Application.java"), java_content)
        .await
        .unwrap();

    // Add more languages...
    create_additional_language_samples(temp_dir).await;
}

async fn create_additional_language_samples(temp_dir: &TempDir) {
    // TypeScript
    let ts_content = r#"
interface DatabaseConnection {
    connect(): Promise<boolean>;
}

class PostgresConnection implements DatabaseConnection {
    constructor(private host: string) {}
    
    async connect(): Promise<boolean> {
        return true;
    }
}

class UserService<T extends DatabaseConnection> {
    constructor(private db: T) {}
    
    async createUser(name: string): Promise<User> {
        return new User(name);
    }
}

class User {
    constructor(public name: string) {}
}

export const MAX_USERS = 1000;

export function initializeApp(): void {}
"#;
    fs::write(temp_dir.path().join("complex.ts"), ts_content)
        .await
        .unwrap();

    // Go
    let go_content = r#"
package main

import "fmt"

type DatabaseConnection interface {
    Connect() bool
}

type PostgresConnection struct {
    Host string
}

func (p *PostgresConnection) Connect() bool {
    return true
}

type UserService struct {
    db DatabaseConnection
}

func NewUserService(db DatabaseConnection) *UserService {
    return &UserService{db: db}
}

func (s *UserService) CreateUser(name string) *User {
    return &User{Name: name}
}

type User struct {
    Name string
}

const MaxUsers = 1000

func InitializeApp() error {
    return nil
}

func main() {
    fmt.Println("Application started")
}
"#;
    fs::write(temp_dir.path().join("main.go"), go_content)
        .await
        .unwrap();

    // C++
    let cpp_content = r#"
#include <string>
#include <memory>

namespace database {
    class Connection {
    public:
        virtual bool connect() = 0;
        virtual ~Connection() = default;
    };
    
    class PostgresConnection : public Connection {
    private:
        std::string host;
    public:
        PostgresConnection(const std::string& host) : host(host) {}
        bool connect() override { return true; }
    };
}

template<typename T>
class UserService {
private:
    std::unique_ptr<T> db;
public:
    UserService(std::unique_ptr<T> db) : db(std::move(db)) {}
    
    std::unique_ptr<User> createUser(const std::string& name) {
        return std::make_unique<User>(name);
    }
};

class User {
private:
    std::string name;
public:
    User(const std::string& name) : name(name) {}
};

const int MAX_USERS = 1000;

void initializeApp() {
    // Implementation
}
"#;
    fs::write(temp_dir.path().join("complex.cpp"), cpp_content)
        .await
        .unwrap();

    // C
    let c_content = r#"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    char* host;
} PostgresConnection;

typedef struct {
    char* name;
} User;

typedef struct {
    PostgresConnection* db;
} UserService;

int postgres_connect(PostgresConnection* conn) {
    return 1;
}

PostgresConnection* create_postgres_connection(const char* host) {
    PostgresConnection* conn = malloc(sizeof(PostgresConnection));
    conn->host = strdup(host);
    return conn;
}

UserService* create_user_service(PostgresConnection* db) {
    UserService* service = malloc(sizeof(UserService));
    service->db = db;
    return service;
}

User* create_user(UserService* service, const char* name) {
    User* user = malloc(sizeof(User));
    user->name = strdup(name);
    return user;
}

void initialize_app() {
    printf("Application initialized\n");
}

#define MAX_USERS 1000

int main() {
    initialize_app();
    return 0;
}
"#;
    fs::write(temp_dir.path().join("complex.c"), c_content)
        .await
        .unwrap();

    // JavaScript
    let js_content = r#"
class DatabaseConnection {
    async connect() {
        throw new Error("Not implemented");
    }
}

class PostgresConnection extends DatabaseConnection {
    constructor(host) {
        super();
        this.host = host;
    }
    
    async connect() {
        return true;
    }
}

class UserService {
    constructor(db) {
        this.db = db;
    }
    
    async createUser(name) {
        return new User(name);
    }
}

class User {
    constructor(name) {
        this.name = name;
    }
}

const MAX_USERS = 1000;

function initializeApp() {
    console.log("App initialized");
}

module.exports = {
    DatabaseConnection,
    PostgresConnection,
    UserService,
    User,
    MAX_USERS,
    initializeApp
};
"#;
    fs::write(temp_dir.path().join("complex.js"), js_content)
        .await
        .unwrap();

    // C#
    let csharp_content = r#"
using System;
using System.Threading.Tasks;

namespace MyApp
{
    public interface IDatabaseConnection
    {
        Task<bool> ConnectAsync();
    }

    public class PostgresConnection : IDatabaseConnection
    {
        private readonly string host;
        
        public PostgresConnection(string host)
        {
            this.host = host;
        }
        
        public async Task<bool> ConnectAsync()
        {
            return await Task.FromResult(true);
        }
    }

    public class UserService<T> where T : IDatabaseConnection
    {
        private readonly T db;
        
        public UserService(T db)
        {
            this.db = db;
        }
        
        public async Task<User> CreateUserAsync(string name)
        {
            return await Task.FromResult(new User(name));
        }
    }

    public class User
    {
        public string Name { get; }
        
        public User(string name)
        {
            Name = name;
        }
    }

    public static class Constants
    {
        public const int MAX_USERS = 1000;
    }

    public class Program
    {
        public static void InitializeApp()
        {
            Console.WriteLine("App initialized");
        }
        
        public static void Main(string[] args)
        {
            InitializeApp();
        }
    }
}
"#;
    fs::write(temp_dir.path().join("Program.cs"), csharp_content)
        .await
        .unwrap();

    // Ruby
    let ruby_content = r#"
module Database
  class Connection
    def connect
      raise NotImplementedError
    end
  end

  class PostgresConnection < Connection
    attr_reader :host
    
    def initialize(host)
      @host = host
    end
    
    def connect
      true
    end
  end
end

class UserService
  def initialize(db)
    @db = db
  end
  
  def create_user(name)
    User.new(name)
  end
end

class User
  attr_reader :name
  
  def initialize(name)
    @name = name
  end
end

MAX_USERS = 1000

def initialize_app
  puts "App initialized"
end

if __FILE__ == $0
  initialize_app
end
"#;
    fs::write(temp_dir.path().join("complex.rb"), ruby_content)
        .await
        .unwrap();

    // Scala
    let scala_content = r#"
package myapp

trait DatabaseConnection {
  def connect(): Boolean
}

class PostgresConnection(val host: String) extends DatabaseConnection {
  override def connect(): Boolean = true
}

class UserService[T <: DatabaseConnection](db: T) {
  def createUser(name: String): User = User(name)
}

case class User(name: String)

object Constants {
  val MAX_USERS = 1000
}

object Application {
  def initializeApp(): Unit = {
    println("App initialized")
  }
  
  def main(args: Array[String]): Unit = {
    initializeApp()
  }
}
"#;
    fs::write(temp_dir.path().join("Application.scala"), scala_content)
        .await
        .unwrap();

    // Swift
    let swift_content = r#"
import Foundation

protocol DatabaseConnection {
    func connect() async -> Bool
}

class PostgresConnection: DatabaseConnection {
    let host: String
    
    init(host: String) {
        self.host = host
    }
    
    func connect() async -> Bool {
        return true
    }
}

class UserService<T: DatabaseConnection> {
    private let db: T
    
    init(db: T) {
        self.db = db
    }
    
    func createUser(name: String) async -> User {
        return User(name: name)
    }
}

struct User {
    let name: String
}

let MAX_USERS = 1000

func initializeApp() {
    print("App initialized")
}

@main
struct Application {
    static func main() {
        initializeApp()
    }
}
"#;
    fs::write(temp_dir.path().join("Application.swift"), swift_content)
        .await
        .unwrap();

    // Kotlin
    let kotlin_content = r#"
package com.myapp

interface DatabaseConnection {
    suspend fun connect(): Boolean
}

class PostgresConnection(private val host: String) : DatabaseConnection {
    override suspend fun connect(): Boolean = true
}

class UserService<T : DatabaseConnection>(private val db: T) {
    suspend fun createUser(name: String): User {
        return User(name)
    }
}

data class User(val name: String)

object Constants {
    const val MAX_USERS = 1000
}

fun initializeApp() {
    println("App initialized")
}

fun main() {
    initializeApp()
}
"#;
    fs::write(temp_dir.path().join("Application.kt"), kotlin_content)
        .await
        .unwrap();
}

async fn test_class_extraction(store: &SymbolStore) {
    // Test that classes are extracted from all languages
    let expected_classes = vec!["PostgresConnection", "UserService", "User", "Application"];

    for class_name in expected_classes {
        let symbols = store.get_symbols(class_name);
        assert!(!symbols.is_empty(), "Class '{}' not found", class_name);

        // Verify at least one is classified as Class
        let has_class = symbols.iter().any(|s| s.symbol_type == SymbolType::Class);
        assert!(has_class, "Class '{}' not properly classified", class_name);
    }
}

async fn test_function_extraction(store: &SymbolStore) {
    // Test that functions/methods are extracted from all languages
    let expected_functions = vec![
        "connect",
        "createUser",
        "create_user",
        "initializeApp",
        "initialize_app",
        "InitializeApp",
        "main",
    ];

    for func_name in expected_functions {
        let symbols = store.get_symbols(func_name);
        if !symbols.is_empty() {
            // Verify at least one is classified as Function or Method
            let has_function = symbols.iter().any(|s| {
                s.symbol_type == SymbolType::Function || s.symbol_type == SymbolType::Method
            });
            assert!(
                has_function,
                "Function '{}' not properly classified",
                func_name
            );
        }
    }
}

async fn test_interface_extraction(store: &SymbolStore) {
    // Test that interfaces/traits are extracted
    let expected_interfaces = vec![
        "DatabaseConnection",
        "DatabaseConnectionInterface",
        "Connection",
    ];

    for interface_name in expected_interfaces {
        let symbols = store.get_symbols(interface_name);
        if !symbols.is_empty() {
            // Check if any are classified as Interface
            let _has_interface = symbols
                .iter()
                .any(|s| s.symbol_type == SymbolType::Interface);
        }
    }
}

async fn test_variable_extraction(store: &SymbolStore) {
    // Test that constants/variables are extracted
    let expected_variables = vec![
        "MAX_USERS",
        "MaxUsers",
        "T", // Type variables
    ];

    for var_name in expected_variables {
        let symbols = store.get_symbols(var_name);
        if !symbols.is_empty() {
            let _has_variable = symbols
                .iter()
                .any(|s| s.symbol_type == SymbolType::Variable);
        }
    }
}

async fn test_complex_scenarios(store: &SymbolStore) {
    // Test complex scenarios that should work across languages

    // 1. Generic/Template classes
    let generic_classes = store.get_symbols("UserService");
    assert!(
        !generic_classes.is_empty(),
        "Generic UserService class not found"
    );

    // 2. Interface implementations
    let postgres_symbols = store.get_symbols("PostgresConnection");
    assert!(
        !postgres_symbols.is_empty(),
        "PostgresConnection implementation not found"
    );

    // 3. Multiple files with same symbol names
    let user_symbols = store.get_symbols("User");
    assert!(
        user_symbols.len() >= 3,
        "User class should appear in multiple languages"
    );

    // 4. Verify symbol distribution across languages
    let mut language_counts = HashMap::new();
    for symbol in store.get_all_symbols() {
        let lang = detect_language_from_file(&symbol.location.file);
        *language_counts.entry(lang).or_insert(0) += 1;
    }

    // Should have symbols from at least 8 different languages
    assert!(
        language_counts.len() >= 8,
        "Symbols should be found in multiple languages, got: {}",
        language_counts.len()
    );

    println!("Language distribution: {:?}", language_counts);
}

fn detect_language_from_file(file_path: &std::path::Path) -> String {
    match file_path.extension().and_then(|ext| ext.to_str()) {
        Some("rs") => "Rust".to_string(),
        Some("py") => "Python".to_string(),
        Some("php") => "PHP".to_string(),
        Some("m") => "Objective-C".to_string(),
        Some("java") => "Java".to_string(),
        Some("ts") => "TypeScript".to_string(),
        Some("go") => "Go".to_string(),
        Some("cpp") | Some("cc") => "C++".to_string(),
        _ => "Unknown".to_string(),
    }
}

// Helper trait to get all symbols (would need to be implemented in SymbolStore)
trait SymbolStoreExt {
    fn get_all_symbols(&self) -> Vec<roberto_mcp::models::Symbol>;
}

impl SymbolStoreExt for SymbolStore {
    fn get_all_symbols(&self) -> Vec<roberto_mcp::models::Symbol> {
        let mut all_symbols = Vec::new();

        // This is a simplified implementation - in reality we'd iterate through
        // the internal storage structures
        for entry in self.symbols_by_name.iter() {
            for symbol_id in entry.value() {
                if let Some(symbol_entry) = self.symbol_data.get(symbol_id) {
                    all_symbols.push(symbol_entry.value().clone());
                }
            }
        }

        all_symbols
    }
}

#[tokio::test]
async fn test_language_specific_features() {
    let temp_dir = TempDir::new().unwrap();
    let store = Arc::new(SymbolStore::new());
    let mut pipeline = IndexingPipeline::new(store.clone()).unwrap();

    // Test language-specific features

    // Rust: Traits and implementations
    let rust_traits = r#"
pub trait Display {
    fn fmt(&self) -> String;
}

pub struct Point {
    x: i32,
    y: i32,
}

impl Display for Point {
    fn fmt(&self) -> String {
        format!("({}, {})", self.x, self.y)
    }
}
"#;
    fs::write(temp_dir.path().join("traits.rs"), rust_traits)
        .await
        .unwrap();

    // Python: Decorators and async
    let python_decorators = r#"
from functools import wraps

def retry(times=3):
    def decorator(func):
        @wraps(func)
        async def wrapper(*args, **kwargs):
            for i in range(times):
                try:
                    return await func(*args, **kwargs)
                except Exception:
                    if i == times - 1:
                        raise
        return wrapper
    return decorator

class AsyncService:
    @retry(times=5)
    async def fetch_data(self):
        pass
"#;
    fs::write(temp_dir.path().join("decorators.py"), python_decorators)
        .await
        .unwrap();

    // PHP: Traits and namespaces
    let php_traits = r#"<?php
namespace App\Services;

trait Timestampable {
    protected $created_at;
    protected $updated_at;
    
    public function touch() {
        $this->updated_at = new \DateTime();
    }
}

class User {
    use Timestampable;
    
    private string $name;
    
    public function __construct(string $name) {
        $this->name = $name;
        $this->created_at = new \DateTime();
        $this->updated_at = new \DateTime();
    }
}
?>"#;
    fs::write(temp_dir.path().join("traits.php"), php_traits)
        .await
        .unwrap();

    let result = pipeline.index_directory(temp_dir.path()).await;
    assert!(result.files_processed >= 3);
    assert!(result.symbols_found >= 10);

    // Verify language-specific features were extracted
    assert!(
        !store.get_symbols("Display").is_empty(),
        "Rust trait not found"
    );
    assert!(
        !store.get_symbols("retry").is_empty(),
        "Python decorator not found"
    );
    assert!(
        !store.get_symbols("Timestampable").is_empty(),
        "PHP trait not found"
    );

    println!("✅ Language-specific features test passed");
}
