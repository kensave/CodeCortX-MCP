// Complex Rust example with comprehensive constructs
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use serde::{Serialize, Deserialize};
use tokio::sync::{mpsc, oneshot};

// Constants
pub const MAX_CONNECTIONS: usize = 100;
pub const DEFAULT_TIMEOUT: u64 = 30;
pub const API_VERSION: &str = "1.0.0";
pub const CACHE_SIZE: usize = 1000;

// Type aliases
pub type UserId = u64;
pub type ProductId = u64;
pub type ConnectionId = u64;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

// Error types
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Connection failed: {message}")]
    ConnectionFailed { message: String },
    #[error("Query failed: {query}")]
    QueryFailed { query: String },
    #[error("Transaction failed")]
    TransactionFailed,
    #[error("Timeout after {seconds} seconds")]
    Timeout { seconds: u64 },
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid email format: {email}")]
    InvalidEmail { email: String },
    #[error("Field '{field}' is required")]
    RequiredField { field: String },
    #[error("Value '{value}' is out of range")]
    OutOfRange { value: String },
}

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
    #[error("Not found: {resource}")]
    NotFound { resource: String },
    #[error("Permission denied")]
    PermissionDenied,
}

// Traits
pub trait DatabaseConnection: Send + Sync {
    async fn connect(&self) -> Result<()>;
    async fn execute_query(&self, query: &str, params: &[&str]) -> Result<QueryResult>;
    async fn close(&self) -> Result<()>;
    async fn begin_transaction(&self) -> Result<Box<dyn Transaction>>;
    fn is_connected(&self) -> bool;
}

pub trait Transaction: Send + Sync {
    async fn execute(&self, query: &str, params: &[&str]) -> Result<QueryResult>;
    async fn commit(self: Box<Self>) -> Result<()>;
    async fn rollback(self: Box<Self>) -> Result<()>;
}

pub trait Cache<K, V>: Send + Sync {
    async fn get(&self, key: &K) -> Option<V>;
    async fn set(&self, key: K, value: V, ttl: Option<Duration>);
    async fn remove(&self, key: &K) -> bool;
    async fn clear(&self);
    fn len(&self) -> usize;
}

pub trait Validator<T> {
    type Error;
    fn validate(&self, item: &T) -> std::result::Result<(), Self::Error>;
}

pub trait Repository<T, ID> {
    async fn find_by_id(&self, id: ID) -> Result<Option<T>>;
    async fn save(&self, item: &T) -> Result<ID>;
    async fn update(&self, id: ID, item: &T) -> Result<()>;
    async fn delete(&self, id: ID) -> Result<bool>;
    async fn find_all(&self) -> Result<Vec<T>>;
}

// Enums
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
    Deleted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductCategory {
    Electronics,
    Clothing,
    Books,
    Home,
    Sports,
    Automotive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

// Structs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub max_connections: usize,
    pub timeout: Duration,
    pub ssl_enabled: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            database: "myapp".to_string(),
            username: "user".to_string(),
            password: "password".to_string(),
            max_connections: MAX_CONNECTIONS,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT),
            ssl_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub status: UserStatus,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub last_login: Option<SystemTime>,
    pub metadata: HashMap<String, String>,
}

impl User {
    pub fn new(username: String, email: String) -> Self {
        let now = SystemTime::now();
        Self {
            id: 0, // Will be set by database
            username,
            email,
            first_name: None,
            last_name: None,
            status: UserStatus::Active,
            created_at: now,
            updated_at: now,
            last_login: None,
            metadata: HashMap::new(),
        }
    }
    
    pub fn full_name(&self) -> String {
        match (&self.first_name, &self.last_name) {
            (Some(first), Some(last)) => format!("{} {}", first, last),
            (Some(first), None) => first.clone(),
            (None, Some(last)) => last.clone(),
            (None, None) => self.username.clone(),
        }
    }
    
    pub fn is_active(&self) -> bool {
        matches!(self.status, UserStatus::Active)
    }
    
    pub fn update_login_time(&mut self) {
        self.last_login = Some(SystemTime::now());
        self.updated_at = SystemTime::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: ProductId,
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
    pub category: ProductCategory,
    pub tags: Vec<String>,
    pub in_stock: bool,
    pub stock_quantity: u32,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

impl Product {
    pub fn new(name: String, price: f64, category: ProductCategory) -> Self {
        let now = SystemTime::now();
        Self {
            id: 0,
            name,
            description: None,
            price,
            category,
            tags: Vec::new(),
            in_stock: true,
            stock_quantity: 0,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = SystemTime::now();
        }
    }
    
    pub fn calculate_discounted_price(&self, discount_percent: f64) -> f64 {
        self.price * (1.0 - discount_percent / 100.0)
    }
    
    pub fn is_available(&self) -> bool {
        self.in_stock && self.stock_quantity > 0
    }
}

#[derive(Debug)]
pub struct QueryResult {
    pub rows: Vec<HashMap<String, String>>,
    pub affected_rows: u64,
    pub execution_time: Duration,
}

impl QueryResult {
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            affected_rows: 0,
            execution_time: Duration::from_millis(0),
        }
    }
    
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }
}

// Database implementations
pub struct PostgresConnection {
    config: DatabaseConfig,
    status: Arc<RwLock<ConnectionStatus>>,
    connection_id: ConnectionId,
}

impl PostgresConnection {
    pub fn new(config: DatabaseConfig) -> Self {
        Self {
            config,
            status: Arc::new(RwLock::new(ConnectionStatus::Disconnected)),
            connection_id: rand::random(),
        }
    }
    
    async fn simulate_network_delay(&self) {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[async_trait::async_trait]
impl DatabaseConnection for PostgresConnection {
    async fn connect(&self) -> Result<()> {
        {
            let mut status = self.status.write().unwrap();
            *status = ConnectionStatus::Connecting;
        }
        
        self.simulate_network_delay().await;
        
        // Simulate connection logic
        if self.config.host == "invalid" {
            let mut status = self.status.write().unwrap();
            *status = ConnectionStatus::Error;
            return Err(Box::new(DatabaseError::ConnectionFailed {
                message: "Invalid host".to_string(),
            }));
        }
        
        let mut status = self.status.write().unwrap();
        *status = ConnectionStatus::Connected;
        
        println!("Connected to PostgreSQL at {}:{}", self.config.host, self.config.port);
        Ok(())
    }
    
    async fn execute_query(&self, query: &str, params: &[&str]) -> Result<QueryResult> {
        if !self.is_connected() {
            return Err(Box::new(DatabaseError::QueryFailed {
                query: query.to_string(),
            }));
        }
        
        self.simulate_network_delay().await;
        
        let mut result = QueryResult::new();
        result.execution_time = Duration::from_millis(25);
        
        // Simulate different query types
        if query.starts_with("SELECT") {
            // Simulate SELECT results
            let mut row = HashMap::new();
            row.insert("id".to_string(), "1".to_string());
            row.insert("name".to_string(), "test".to_string());
            result.rows.push(row);
        } else if query.starts_with("INSERT") || query.starts_with("UPDATE") || query.starts_with("DELETE") {
            result.affected_rows = 1;
        }
        
        Ok(result)
    }
    
    async fn close(&self) -> Result<()> {
        let mut status = self.status.write().unwrap();
        *status = ConnectionStatus::Disconnected;
        Ok(())
    }
    
    async fn begin_transaction(&self) -> Result<Box<dyn Transaction>> {
        if !self.is_connected() {
            return Err(Box::new(DatabaseError::TransactionFailed));
        }
        
        Ok(Box::new(PostgresTransaction::new(self.connection_id)))
    }
    
    fn is_connected(&self) -> bool {
        matches!(*self.status.read().unwrap(), ConnectionStatus::Connected)
    }
}

pub struct PostgresTransaction {
    connection_id: ConnectionId,
    is_active: bool,
}

impl PostgresTransaction {
    fn new(connection_id: ConnectionId) -> Self {
        Self {
            connection_id,
            is_active: true,
        }
    }
}

#[async_trait::async_trait]
impl Transaction for PostgresTransaction {
    async fn execute(&self, query: &str, _params: &[&str]) -> Result<QueryResult> {
        if !self.is_active {
            return Err(Box::new(DatabaseError::TransactionFailed));
        }
        
        let mut result = QueryResult::new();
        result.affected_rows = 1;
        Ok(result)
    }
    
    async fn commit(mut self: Box<Self>) -> Result<()> {
        self.is_active = false;
        Ok(())
    }
    
    async fn rollback(mut self: Box<Self>) -> Result<()> {
        self.is_active = false;
        Ok(())
    }
}

// Cache implementation
pub struct MemoryCache<K, V> {
    data: Arc<Mutex<HashMap<K, CacheEntry<V>>>>,
    max_size: usize,
}

struct CacheEntry<V> {
    value: V,
    expires_at: Option<SystemTime>,
    access_count: u64,
}

impl<K: Clone + Eq + std::hash::Hash, V: Clone> MemoryCache<K, V> {
    pub fn new(max_size: usize) -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
            max_size,
        }
    }
    
    fn cleanup_expired(&self, data: &mut HashMap<K, CacheEntry<V>>) {
        let now = SystemTime::now();
        data.retain(|_, entry| {
            entry.expires_at.map_or(true, |expires| expires > now)
        });
    }
    
    fn evict_lru(&self, data: &mut HashMap<K, CacheEntry<V>>) {
        if data.len() >= self.max_size {
            // Find entry with lowest access count
            if let Some(lru_key) = data.iter()
                .min_by_key(|(_, entry)| entry.access_count)
                .map(|(k, _)| k.clone()) {
                data.remove(&lru_key);
            }
        }
    }
}

#[async_trait::async_trait]
impl<K: Clone + Eq + std::hash::Hash + Send + Sync, V: Clone + Send + Sync> Cache<K, V> for MemoryCache<K, V> {
    async fn get(&self, key: &K) -> Option<V> {
        let mut data = self.data.lock().unwrap();
        self.cleanup_expired(&mut data);
        
        if let Some(entry) = data.get_mut(key) {
            entry.access_count += 1;
            Some(entry.value.clone())
        } else {
            None
        }
    }
    
    async fn set(&self, key: K, value: V, ttl: Option<Duration>) {
        let mut data = self.data.lock().unwrap();
        self.cleanup_expired(&mut data);
        self.evict_lru(&mut data);
        
        let expires_at = ttl.map(|duration| SystemTime::now() + duration);
        let entry = CacheEntry {
            value,
            expires_at,
            access_count: 1,
        };
        
        data.insert(key, entry);
    }
    
    async fn remove(&self, key: &K) -> bool {
        let mut data = self.data.lock().unwrap();
        data.remove(key).is_some()
    }
    
    async fn clear(&self) {
        let mut data = self.data.lock().unwrap();
        data.clear();
    }
    
    fn len(&self) -> usize {
        let data = self.data.lock().unwrap();
        data.len()
    }
}

// Validators
pub struct UserValidator;

impl Validator<User> for UserValidator {
    type Error = ValidationError;
    
    fn validate(&self, user: &User) -> std::result::Result<(), Self::Error> {
        if user.username.is_empty() {
            return Err(ValidationError::RequiredField {
                field: "username".to_string(),
            });
        }
        
        if !user.email.contains('@') || !user.email.contains('.') {
            return Err(ValidationError::InvalidEmail {
                email: user.email.clone(),
            });
        }
        
        Ok(())
    }
}

pub struct ProductValidator;

impl Validator<Product> for ProductValidator {
    type Error = ValidationError;
    
    fn validate(&self, product: &Product) -> std::result::Result<(), Self::Error> {
        if product.name.is_empty() {
            return Err(ValidationError::RequiredField {
                field: "name".to_string(),
            });
        }
        
        if product.price < 0.0 {
            return Err(ValidationError::OutOfRange {
                value: product.price.to_string(),
            });
        }
        
        Ok(())
    }
}

// Repository implementations
pub struct UserRepository<D: DatabaseConnection> {
    database: Arc<D>,
    cache: Arc<dyn Cache<UserId, User>>,
    validator: UserValidator,
}

impl<D: DatabaseConnection> UserRepository<D> {
    pub fn new(database: Arc<D>, cache: Arc<dyn Cache<UserId, User>>) -> Self {
        Self {
            database,
            cache,
            validator: UserValidator,
        }
    }
}

#[async_trait::async_trait]
impl<D: DatabaseConnection> Repository<User, UserId> for UserRepository<D> {
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>> {
        // Check cache first
        if let Some(user) = self.cache.get(&id).await {
            return Ok(Some(user));
        }
        
        // Query database
        let query = "SELECT * FROM users WHERE id = $1";
        let params = [&id.to_string()];
        let result = self.database.execute_query(query, &params).await?;
        
        if result.rows.is_empty() {
            return Ok(None);
        }
        
        // Simulate user creation from database row
        let mut user = User::new("username".to_string(), "email@example.com".to_string());
        user.id = id;
        
        // Cache the result
        self.cache.set(id, user.clone(), Some(Duration::from_secs(3600))).await;
        
        Ok(Some(user))
    }
    
    async fn save(&self, user: &User) -> Result<UserId> {
        self.validator.validate(user)?;
        
        let query = "INSERT INTO users (username, email) VALUES ($1, $2) RETURNING id";
        let params = [&user.username, &user.email];
        let result = self.database.execute_query(query, &params).await?;
        
        let id = 1; // Simulate ID from database
        self.cache.set(id, user.clone(), Some(Duration::from_secs(3600))).await;
        
        Ok(id)
    }
    
    async fn update(&self, id: UserId, user: &User) -> Result<()> {
        self.validator.validate(user)?;
        
        let query = "UPDATE users SET username = $1, email = $2 WHERE id = $3";
        let params = [&user.username, &user.email, &id.to_string()];
        self.database.execute_query(query, &params).await?;
        
        self.cache.set(id, user.clone(), Some(Duration::from_secs(3600))).await;
        
        Ok(())
    }
    
    async fn delete(&self, id: UserId) -> Result<bool> {
        let query = "DELETE FROM users WHERE id = $1";
        let params = [&id.to_string()];
        let result = self.database.execute_query(query, &params).await?;
        
        self.cache.remove(&id).await;
        
        Ok(result.affected_rows > 0)
    }
    
    async fn find_all(&self) -> Result<Vec<User>> {
        let query = "SELECT * FROM users ORDER BY created_at DESC";
        let result = self.database.execute_query(query, &[]).await?;
        
        // Simulate user creation from database rows
        let users: Vec<User> = result.rows.iter().enumerate()
            .map(|(i, _)| {
                let mut user = User::new(format!("user{}", i), format!("user{}@example.com", i));
                user.id = i as u64;
                user
            })
            .collect();
        
        Ok(users)
    }
}

// Service layer
pub struct UserService<D: DatabaseConnection> {
    repository: Arc<UserRepository<D>>,
    event_sender: mpsc::UnboundedSender<UserEvent>,
}

#[derive(Debug, Clone)]
pub enum UserEvent {
    Created { user_id: UserId, username: String },
    Updated { user_id: UserId },
    Deleted { user_id: UserId },
    LoginRecorded { user_id: UserId },
}

impl<D: DatabaseConnection> UserService<D> {
    pub fn new(repository: Arc<UserRepository<D>>) -> (Self, mpsc::UnboundedReceiver<UserEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        
        let service = Self {
            repository,
            event_sender: tx,
        };
        
        (service, rx)
    }
    
    pub async fn create_user(&self, username: String, email: String) -> Result<User> {
        let mut user = User::new(username.clone(), email);
        let user_id = self.repository.save(&user).await?;
        user.id = user_id;
        
        // Send event
        let _ = self.event_sender.send(UserEvent::Created {
            user_id,
            username: username.clone(),
        });
        
        Ok(user)
    }
    
    pub async fn get_user(&self, id: UserId) -> Result<Option<User>> {
        self.repository.find_by_id(id).await
    }
    
    pub async fn update_user(&self, id: UserId, updates: UserUpdates) -> Result<Option<User>> {
        if let Some(mut user) = self.repository.find_by_id(id).await? {
            if let Some(username) = updates.username {
                user.username = username;
            }
            if let Some(email) = updates.email {
                user.email = email;
            }
            if let Some(first_name) = updates.first_name {
                user.first_name = Some(first_name);
            }
            if let Some(last_name) = updates.last_name {
                user.last_name = Some(last_name);
            }
            
            user.updated_at = SystemTime::now();
            
            self.repository.update(id, &user).await?;
            
            // Send event
            let _ = self.event_sender.send(UserEvent::Updated { user_id: id });
            
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }
    
    pub async fn record_login(&self, id: UserId) -> Result<bool> {
        if let Some(mut user) = self.repository.find_by_id(id).await? {
            user.update_login_time();
            self.repository.update(id, &user).await?;
            
            // Send event
            let _ = self.event_sender.send(UserEvent::LoginRecorded { user_id: id });
            
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Debug, Default)]
pub struct UserUpdates {
    pub username: Option<String>,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

// Configuration management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub cache_size: usize,
    pub log_level: LogLevel,
    pub debug_mode: bool,
    pub max_request_size: usize,
    pub request_timeout: Duration,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            cache_size: CACHE_SIZE,
            log_level: LogLevel::Info,
            debug_mode: false,
            max_request_size: 1024 * 1024, // 1MB
            request_timeout: Duration::from_secs(30),
        }
    }
}

impl AppConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        if let Ok(host) = std::env::var("DB_HOST") {
            config.database.host = host;
        }
        if let Ok(port) = std::env::var("DB_PORT") {
            if let Ok(port) = port.parse() {
                config.database.port = port;
            }
        }
        if let Ok(debug) = std::env::var("DEBUG") {
            config.debug_mode = debug == "true" || debug == "1";
        }
        
        config
    }
}

// Application modules
pub mod database {
    pub use super::{DatabaseConnection, Transaction, PostgresConnection, QueryResult};
}

pub mod cache {
    pub use super::{Cache, MemoryCache};
}

pub mod models {
    pub use super::{User, Product, UserStatus, ProductCategory};
}

pub mod services {
    pub use super::{UserService, UserEvent, UserUpdates};
}

pub mod repositories {
    pub use super::{Repository, UserRepository};
}

pub mod validators {
    pub use super::{Validator, UserValidator, ProductValidator};
}

pub mod config {
    pub use super::{AppConfig, DatabaseConfig};
}

pub mod errors {
    pub use super::{DatabaseError, ValidationError, ServiceError};
}

// Main application function
pub async fn run_application() -> Result<()> {
    println!("Starting CodeCortext Application {}", API_VERSION);
    
    // Load configuration
    let config = AppConfig::from_env();
    
    // Initialize database
    let database = Arc::new(PostgresConnection::new(config.database.clone()));
    database.connect().await?;
    
    // Initialize cache
    let cache: Arc<dyn Cache<UserId, User>> = Arc::new(MemoryCache::new(config.cache_size));
    
    // Initialize repository
    let user_repository = Arc::new(UserRepository::new(database.clone(), cache.clone()));
    
    // Initialize service
    let (user_service, mut event_receiver) = UserService::new(user_repository.clone());
    
    // Start event processing task
    let event_task = tokio::spawn(async move {
        while let Some(event) = event_receiver.recv().await {
            match event {
                UserEvent::Created { user_id, username } => {
                    println!("User created: {} (ID: {})", username, user_id);
                }
                UserEvent::Updated { user_id } => {
                    println!("User updated: ID {}", user_id);
                }
                UserEvent::Deleted { user_id } => {
                    println!("User deleted: ID {}", user_id);
                }
                UserEvent::LoginRecorded { user_id } => {
                    println!("Login recorded for user: ID {}", user_id);
                }
            }
        }
    });
    
    // Create sample user
    let user = user_service.create_user("john_doe".to_string(), "john@example.com".to_string()).await?;
    println!("Created user: {} <{}>", user.username, user.email);
    
    // Get user
    if let Some(retrieved_user) = user_service.get_user(user.id).await? {
        println!("Retrieved user: {}", retrieved_user.full_name());
    }
    
    // Update user
    let updates = UserUpdates {
        first_name: Some("John".to_string()),
        last_name: Some("Doe".to_string()),
        ..Default::default()
    };
    
    if let Some(updated_user) = user_service.update_user(user.id, updates).await? {
        println!("Updated user: {}", updated_user.full_name());
    }
    
    // Record login
    user_service.record_login(user.id).await?;
    
    // Create sample product
    let mut product = Product::new("Laptop".to_string(), 999.99, ProductCategory::Electronics);
    product.add_tag("computer".to_string());
    product.add_tag("portable".to_string());
    
    let discounted_price = product.calculate_discounted_price(10.0);
    println!("Product: {}, Original: ${:.2}, Discounted: ${:.2}", 
             product.name, product.price, discounted_price);
    
    // Cleanup
    database.close().await?;
    event_task.abort();
    
    println!("Application completed successfully");
    Ok(())
}

// Tests module
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_user_creation() {
        let user = User::new("test_user".to_string(), "test@example.com".to_string());
        assert_eq!(user.username, "test_user");
        assert_eq!(user.email, "test@example.com");
        assert!(user.is_active());
    }
    
    #[tokio::test]
    async fn test_cache_operations() {
        let cache = MemoryCache::new(10);
        
        cache.set("key1".to_string(), "value1".to_string(), None).await;
        let value = cache.get(&"key1".to_string()).await;
        assert_eq!(value, Some("value1".to_string()));
        
        let removed = cache.remove(&"key1".to_string()).await;
        assert!(removed);
        
        let value = cache.get(&"key1".to_string()).await;
        assert_eq!(value, None);
    }
}
