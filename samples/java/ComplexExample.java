package com.example.myapp;

import java.util.*;
import java.util.concurrent.CompletableFuture;
import java.time.LocalDateTime;

// Database interfaces and implementations
interface DatabaseConnection {
    CompletableFuture<Boolean> connect();
    CompletableFuture<QueryResult> executeQuery(String query, Object... params);
    void close();
}

class PostgresConnection implements DatabaseConnection {
    private final String host;
    private final int port;
    private final String database;
    private boolean isConnected = false;
    
    public PostgresConnection(String host, int port, String database) {
        this.host = host;
        this.port = port;
        this.database = database;
    }
    
    @Override
    public CompletableFuture<Boolean> connect() {
        return CompletableFuture.supplyAsync(() -> {
            this.isConnected = true;
            return true;
        });
    }
    
    @Override
    public CompletableFuture<QueryResult> executeQuery(String query, Object... params) {
        return CompletableFuture.supplyAsync(() -> new QueryResult());
    }
    
    @Override
    public void close() {
        this.isConnected = false;
    }
}

class MySQLConnection implements DatabaseConnection {
    private final String connectionString;
    
    public MySQLConnection(String connectionString) {
        this.connectionString = connectionString;
    }
    
    @Override
    public CompletableFuture<Boolean> connect() {
        return CompletableFuture.completedFuture(true);
    }
    
    @Override
    public CompletableFuture<QueryResult> executeQuery(String query, Object... params) {
        return CompletableFuture.completedFuture(new QueryResult());
    }
    
    @Override
    public void close() {}
}

// Data models
class User {
    private Long id;
    private String username;
    private String email;
    private LocalDateTime createdAt;
    private boolean isActive;
    private Map<String, Object> metadata;
    
    public User(String username, String email) {
        this.username = username;
        this.email = email;
        this.createdAt = LocalDateTime.now();
        this.isActive = true;
        this.metadata = new HashMap<>();
    }
    
    // Getters and setters
    public Long getId() { return id; }
    public void setId(Long id) { this.id = id; }
    
    public String getUsername() { return username; }
    public void setUsername(String username) { this.username = username; }
    
    public String getEmail() { return email; }
    public void setEmail(String email) { this.email = email; }
    
    public boolean validateEmail() {
        return email != null && email.contains("@") && email.contains(".");
    }
    
    public Map<String, Object> toMap() {
        Map<String, Object> map = new HashMap<>();
        map.put("id", id);
        map.put("username", username);
        map.put("email", email);
        map.put("createdAt", createdAt);
        map.put("isActive", isActive);
        map.put("metadata", metadata);
        return map;
    }
}

class Product {
    private Long id;
    private String name;
    private Double price;
    private ProductCategory category;
    private String description;
    private List<String> tags;
    
    public Product(String name, Double price, ProductCategory category) {
        this.name = name;
        this.price = price;
        this.category = category;
        this.tags = new ArrayList<>();
    }
    
    public Long getId() { return id; }
    public String getName() { return name; }
    public Double getPrice() { return price; }
    public ProductCategory getCategory() { return category; }
}

enum ProductCategory {
    ELECTRONICS("Electronics"),
    CLOTHING("Clothing"),
    BOOKS("Books"),
    HOME("Home & Garden"),
    SPORTS("Sports");
    
    private final String displayName;
    
    ProductCategory(String displayName) {
        this.displayName = displayName;
    }
    
    public String getDisplayName() {
        return displayName;
    }
}

// Generic cache implementation
class Cache<K, V> {
    private final Map<K, V> data;
    private final int maxSize;
    
    public Cache(int maxSize) {
        this.maxSize = maxSize;
        this.data = new LinkedHashMap<K, V>(16, 0.75f, true) {
            @Override
            protected boolean removeEldestEntry(Map.Entry<K, V> eldest) {
                return size() > Cache.this.maxSize;
            }
        };
    }
    
    public synchronized V get(K key) {
        return data.get(key);
    }
    
    public synchronized void put(K key, V value) {
        data.put(key, value);
    }
    
    public synchronized void clear() {
        data.clear();
    }
}

// Service classes
class UserService<T extends DatabaseConnection> {
    private final T database;
    private final Cache<Long, User> cache;
    
    public UserService(T database) {
        this.database = database;
        this.cache = new Cache<>(1000);
    }
    
    public CompletableFuture<User> createUser(String username, String email) {
        return CompletableFuture.supplyAsync(() -> {
            User user = new User(username, email);
            
            if (!user.validateEmail()) {
                throw new IllegalArgumentException("Invalid email format");
            }
            
            // Simulate database save
            user.setId(System.currentTimeMillis());
            cache.put(user.getId(), user);
            
            return user;
        });
    }
    
    public CompletableFuture<Optional<User>> getUser(Long id) {
        return CompletableFuture.supplyAsync(() -> {
            User cached = cache.get(id);
            if (cached != null) {
                return Optional.of(cached);
            }
            
            // Simulate database query
            return Optional.empty();
        });
    }
    
    public CompletableFuture<Boolean> updateUser(Long id, Map<String, Object> updates) {
        return getUser(id).thenApply(userOpt -> {
            if (userOpt.isPresent()) {
                User user = userOpt.get();
                
                if (updates.containsKey("username")) {
                    user.setUsername((String) updates.get("username"));
                }
                if (updates.containsKey("email")) {
                    user.setEmail((String) updates.get("email"));
                }
                
                cache.put(id, user);
                return true;
            }
            return false;
        });
    }
}

// Configuration
class AppConfig {
    private boolean debugMode = false;
    private String logLevel = "INFO";
    private String databaseUrl = "postgresql://localhost:5432/myapp";
    private int cacheSize = 1000;
    
    public boolean isDebugMode() { return debugMode; }
    public void setDebugMode(boolean debugMode) { this.debugMode = debugMode; }
    
    public String getLogLevel() { return logLevel; }
    public void setLogLevel(String logLevel) { this.logLevel = logLevel; }
    
    public String getDatabaseUrl() { return databaseUrl; }
    public void setDatabaseUrl(String databaseUrl) { this.databaseUrl = databaseUrl; }
    
    public static AppConfig fromProperties(Properties props) {
        AppConfig config = new AppConfig();
        config.setDebugMode(Boolean.parseBoolean(props.getProperty("debug.mode", "false")));
        config.setLogLevel(props.getProperty("log.level", "INFO"));
        config.setDatabaseUrl(props.getProperty("database.url", config.getDatabaseUrl()));
        return config;
    }
}

// Exception classes
class DatabaseException extends Exception {
    public DatabaseException(String message) {
        super(message);
    }
    
    public DatabaseException(String message, Throwable cause) {
        super(message, cause);
    }
}

class ValidationException extends Exception {
    public ValidationException(String message) {
        super(message);
    }
}

class ServiceException extends Exception {
    public ServiceException(String message, Throwable cause) {
        super(message, cause);
    }
}

// Query result
class QueryResult {
    private final List<Map<String, Object>> rows;
    
    public QueryResult() {
        this.rows = new ArrayList<>();
    }
    
    public List<Map<String, Object>> getRows() {
        return rows;
    }
}

// Constants
class Constants {
    public static final int MAX_CONNECTIONS = 100;
    public static final int DEFAULT_TIMEOUT = 30;
    public static final String API_VERSION = "1.0.0";
}

// Main application
public class ComplexExample {
    
    public static DatabaseConnection initializeDatabase(AppConfig config) throws DatabaseException {
        String url = config.getDatabaseUrl();
        
        if (url.startsWith("postgresql")) {
            return new PostgresConnection("localhost", 5432, "myapp");
        } else if (url.startsWith("mysql")) {
            return new MySQLConnection(url);
        } else {
            throw new DatabaseException("Unsupported database type");
        }
    }
    
    public static void main(String[] args) {
        try {
            AppConfig config = new AppConfig();
            config.setDebugMode(true);
            
            DatabaseConnection db = initializeDatabase(config);
            UserService<DatabaseConnection> userService = new UserService<>(db);
            
            // Create sample user
            CompletableFuture<User> userFuture = userService.createUser("john_doe", "john@example.com");
            User user = userFuture.get();
            
            System.out.println("Created user: " + user.getUsername());
            
            // Get user
            CompletableFuture<Optional<User>> retrievedUserFuture = userService.getUser(user.getId());
            Optional<User> retrievedUser = retrievedUserFuture.get();
            
            if (retrievedUser.isPresent()) {
                System.out.println("Retrieved user: " + retrievedUser.get().getUsername());
            }
            
        } catch (Exception e) {
            System.err.println("Application error: " + e.getMessage());
            e.printStackTrace();
        }
    }
}
