#include <iostream>
#include <string>
#include <vector>
#include <memory>
#include <unordered_map>
#include <chrono>
#include <future>
#include <exception>
#include <algorithm>

// Constants
namespace constants {
    constexpr int MAX_CONNECTIONS = 100;
    constexpr int DEFAULT_TIMEOUT = 30;
    const std::string API_VERSION = "1.0.0";
}

// Forward declarations
class DatabaseConnection;
class User;
class Product;
class QueryResult;

// Enums
enum class ConnectionStatus {
    CLOSED,
    OPEN,
    ERROR
};

enum class QueryStatus {
    SUCCESS,
    ERROR,
    TIMEOUT
};

enum class UserStatus {
    ACTIVE,
    INACTIVE,
    SUSPENDED
};

// Exception classes
class DatabaseException : public std::exception {
private:
    std::string message_;
    
public:
    explicit DatabaseException(const std::string& message) : message_(message) {}
    const char* what() const noexcept override { return message_.c_str(); }
};

class ValidationException : public std::exception {
private:
    std::string message_;
    std::string field_;
    
public:
    ValidationException(const std::string& message, const std::string& field = "")
        : message_(message), field_(field) {}
    const char* what() const noexcept override { return message_.c_str(); }
    const std::string& field() const { return field_; }
};

// Abstract base class
class DatabaseConnection {
public:
    virtual ~DatabaseConnection() = default;
    virtual bool connect() = 0;
    virtual std::unique_ptr<QueryResult> executeQuery(const std::string& query, 
                                                     const std::vector<std::string>& params = {}) = 0;
    virtual void close() = 0;
    virtual void beginTransaction() = 0;
    virtual void commit() = 0;
    virtual void rollback() = 0;
    virtual ConnectionStatus getStatus() const = 0;
};

// PostgreSQL implementation
class PostgresConnection : public DatabaseConnection {
private:
    std::string host_;
    int port_;
    std::string database_;
    std::string username_;
    std::string password_;
    ConnectionStatus status_;
    bool transaction_active_;
    
public:
    PostgresConnection(const std::string& host, int port, const std::string& database,
                      const std::string& username, const std::string& password)
        : host_(host), port_(port), database_(database), username_(username), 
          password_(password), status_(ConnectionStatus::CLOSED), transaction_active_(false) {}
    
    bool connect() override {
        try {
            std::cout << "Connecting to PostgreSQL at " << host_ << ":" << port_ 
                      << "/" << database_ << " as " << username_ << std::endl;
            
            // Simulate connection logic
            std::this_thread::sleep_for(std::chrono::milliseconds(100));
            status_ = ConnectionStatus::OPEN;
            return true;
        } catch (const std::exception& e) {
            std::cerr << "Failed to connect: " << e.what() << std::endl;
            status_ = ConnectionStatus::ERROR;
            return false;
        }
    }
    
    std::unique_ptr<QueryResult> executeQuery(const std::string& query, 
                                            const std::vector<std::string>& params = {}) override;
    
    void close() override {
        status_ = ConnectionStatus::CLOSED;
        transaction_active_ = false;
    }
    
    void beginTransaction() override {
        if (status_ != ConnectionStatus::OPEN) {
            throw DatabaseException("Not connected to database");
        }
        transaction_active_ = true;
    }
    
    void commit() override {
        transaction_active_ = false;
    }
    
    void rollback() override {
        transaction_active_ = false;
    }
    
    ConnectionStatus getStatus() const override {
        return status_;
    }
};

// MySQL implementation
class MySQLConnection : public DatabaseConnection {
private:
    std::string connection_string_;
    ConnectionStatus status_;
    
public:
    explicit MySQLConnection(const std::string& connection_string)
        : connection_string_(connection_string), status_(ConnectionStatus::CLOSED) {}
    
    bool connect() override {
        status_ = ConnectionStatus::OPEN;
        return true;
    }
    
    std::unique_ptr<QueryResult> executeQuery(const std::string& query, 
                                            const std::vector<std::string>& params = {}) override;
    
    void close() override { status_ = ConnectionStatus::CLOSED; }
    void beginTransaction() override {}
    void commit() override {}
    void rollback() override {}
    ConnectionStatus getStatus() const override { return status_; }
};

// Query result class
class QueryResult {
private:
    std::vector<std::unordered_map<std::string, std::string>> rows_;
    QueryStatus status_;
    std::string error_message_;
    
public:
    QueryResult() : status_(QueryStatus::SUCCESS) {}
    
    void addRow(const std::unordered_map<std::string, std::string>& row) {
        rows_.push_back(row);
    }
    
    const std::vector<std::unordered_map<std::string, std::string>>& getRows() const {
        return rows_;
    }
    
    size_t getRowCount() const { return rows_.size(); }
    QueryStatus getStatus() const { return status_; }
    
    void setError(const std::string& error) {
        status_ = QueryStatus::ERROR;
        error_message_ = error;
    }
};

// Template cache class
template<typename K, typename V>
class Cache {
private:
    struct CacheItem {
        V value;
        std::chrono::steady_clock::time_point expiry;
        
        CacheItem(const V& val, std::chrono::seconds ttl)
            : value(val), expiry(std::chrono::steady_clock::now() + ttl) {}
    };
    
    std::unordered_map<K, CacheItem> data_;
    size_t max_size_;
    
public:
    explicit Cache(size_t max_size = 1000) : max_size_(max_size) {}
    
    void set(const K& key, const V& value, std::chrono::seconds ttl = std::chrono::seconds(3600)) {
        if (data_.size() >= max_size_ && data_.find(key) == data_.end()) {
            // Simple eviction: remove first element
            data_.erase(data_.begin());
        }
        
        data_.emplace(key, CacheItem(value, ttl));
    }
    
    std::optional<V> get(const K& key) {
        auto it = data_.find(key);
        if (it == data_.end()) {
            return std::nullopt;
        }
        
        if (std::chrono::steady_clock::now() > it->second.expiry) {
            data_.erase(it);
            return std::nullopt;
        }
        
        return it->second.value;
    }
    
    void remove(const K& key) {
        data_.erase(key);
    }
    
    void clear() {
        data_.clear();
    }
};

// Data model classes
class User {
private:
    int id_;
    std::string username_;
    std::string email_;
    std::chrono::system_clock::time_point created_at_;
    std::chrono::system_clock::time_point updated_at_;
    UserStatus status_;
    std::unordered_map<std::string, std::string> metadata_;
    
public:
    User(const std::string& username, const std::string& email)
        : id_(static_cast<int>(std::chrono::system_clock::now().time_since_epoch().count())),
          username_(username), email_(email), 
          created_at_(std::chrono::system_clock::now()),
          updated_at_(std::chrono::system_clock::now()),
          status_(UserStatus::ACTIVE) {}
    
    // Getters
    int getId() const { return id_; }
    const std::string& getUsername() const { return username_; }
    const std::string& getEmail() const { return email_; }
    UserStatus getStatus() const { return status_; }
    
    // Setters
    void setUsername(const std::string& username) {
        username_ = username;
        updateTimestamp();
    }
    
    void setEmail(const std::string& email) {
        email_ = email;
        updateTimestamp();
    }
    
    void setStatus(UserStatus status) {
        status_ = status;
        updateTimestamp();
    }
    
    // Validation
    bool validateEmail() const {
        return email_.find('@') != std::string::npos && 
               email_.find('.') != std::string::npos;
    }
    
    void updateTimestamp() {
        updated_at_ = std::chrono::system_clock::now();
    }
    
    // Serialization
    std::unordered_map<std::string, std::string> toMap() const {
        return {
            {"id", std::to_string(id_)},
            {"username", username_},
            {"email", email_},
            {"status", std::to_string(static_cast<int>(status_))}
        };
    }
    
    static std::unique_ptr<User> fromMap(const std::unordered_map<std::string, std::string>& data) {
        auto user = std::make_unique<User>(data.at("username"), data.at("email"));
        user->id_ = std::stoi(data.at("id"));
        return user;
    }
};

class Product {
private:
    int id_;
    std::string name_;
    double price_;
    std::string category_;
    std::string description_;
    std::vector<std::string> tags_;
    std::chrono::system_clock::time_point created_at_;
    
public:
    Product(const std::string& name, double price, const std::string& category)
        : id_(static_cast<int>(std::chrono::system_clock::now().time_since_epoch().count())),
          name_(name), price_(price), category_(category),
          created_at_(std::chrono::system_clock::now()) {}
    
    // Getters
    int getId() const { return id_; }
    const std::string& getName() const { return name_; }
    double getPrice() const { return price_; }
    const std::string& getCategory() const { return category_; }
    const std::vector<std::string>& getTags() const { return tags_; }
    
    // Tag management
    void addTag(const std::string& tag) {
        if (std::find(tags_.begin(), tags_.end(), tag) == tags_.end()) {
            tags_.push_back(tag);
        }
    }
    
    void removeTag(const std::string& tag) {
        tags_.erase(std::remove(tags_.begin(), tags_.end(), tag), tags_.end());
    }
    
    // Price calculations
    double calculateDiscountedPrice(double discount_percent) const {
        return price_ * (1.0 - discount_percent / 100.0);
    }
};

// Service classes with templates
template<typename DatabaseType>
class UserService {
private:
    std::unique_ptr<DatabaseType> database_;
    std::shared_ptr<Cache<int, std::shared_ptr<User>>> cache_;
    
public:
    UserService(std::unique_ptr<DatabaseType> database, 
                std::shared_ptr<Cache<int, std::shared_ptr<User>>> cache)
        : database_(std::move(database)), cache_(cache) {}
    
    std::future<std::shared_ptr<User>> createUserAsync(const std::string& username, const std::string& email) {
        return std::async(std::launch::async, [this, username, email]() {
            auto user = std::make_shared<User>(username, email);
            
            if (!user->validateEmail()) {
                throw ValidationException("Invalid email format", "email");
            }
            
            try {
                database_->beginTransaction();
                
                std::string query = "INSERT INTO users (username, email) VALUES (?, ?)";
                std::vector<std::string> params = {username, email};
                auto result = database_->executeQuery(query, params);
                
                if (result->getStatus() != QueryStatus::SUCCESS) {
                    throw DatabaseException("Failed to insert user");
                }
                
                database_->commit();
                
                // Cache the user
                cache_->set(user->getId(), user);
                
                std::cout << "User created successfully: " << username << std::endl;
                return user;
                
            } catch (const std::exception& e) {
                database_->rollback();
                std::cerr << "Failed to create user: " << e.what() << std::endl;
                throw;
            }
        });
    }
    
    std::shared_ptr<User> getUser(int id) {
        // Check cache first
        auto cached = cache_->get(id);
        if (cached) {
            return *cached;
        }
        
        // Query database
        std::string query = "SELECT * FROM users WHERE id = ?";
        std::vector<std::string> params = {std::to_string(id)};
        
        try {
            auto result = database_->executeQuery(query, params);
            
            if (result->getRowCount() == 0) {
                return nullptr;
            }
            
            auto user = User::fromMap(result->getRows()[0]);
            auto shared_user = std::shared_ptr<User>(user.release());
            
            // Cache the result
            cache_->set(id, shared_user);
            
            return shared_user;
            
        } catch (const std::exception& e) {
            std::cerr << "Failed to get user: " << e.what() << std::endl;
            return nullptr;
        }
    }
    
    bool updateUser(int id, const std::unordered_map<std::string, std::string>& updates) {
        auto user = getUser(id);
        if (!user) {
            return false;
        }
        
        // Apply updates
        for (const auto& [key, value] : updates) {
            if (key == "username") {
                user->setUsername(value);
            } else if (key == "email") {
                user->setEmail(value);
            }
        }
        
        // Update cache
        cache_->set(id, user);
        
        return true;
    }
};

// Specialized template for different database types
template<>
class UserService<PostgresConnection> {
    // PostgreSQL-specific optimizations could go here
};

// Factory pattern for database connections
class DatabaseFactory {
public:
    static std::unique_ptr<DatabaseConnection> createConnection(const std::string& type, 
                                                               const std::unordered_map<std::string, std::string>& config) {
        if (type == "postgresql") {
            return std::make_unique<PostgresConnection>(
                config.at("host"),
                std::stoi(config.at("port")),
                config.at("database"),
                config.at("username"),
                config.at("password")
            );
        } else if (type == "mysql") {
            std::string connection_string = "mysql://" + config.at("username") + ":" + 
                                          config.at("password") + "@" + config.at("host") + 
                                          ":" + config.at("port") + "/" + config.at("database");
            return std::make_unique<MySQLConnection>(connection_string);
        }
        
        throw std::invalid_argument("Unsupported database type: " + type);
    }
};

// Configuration management
class AppConfig {
private:
    std::unordered_map<std::string, std::string> config_;
    
public:
    AppConfig() {
        // Default configuration
        config_["debug_mode"] = "false";
        config_["log_level"] = "INFO";
        config_["cache_size"] = "1000";
        config_["max_connections"] = std::to_string(constants::MAX_CONNECTIONS);
    }
    
    void set(const std::string& key, const std::string& value) {
        config_[key] = value;
    }
    
    std::string get(const std::string& key, const std::string& default_value = "") const {
        auto it = config_.find(key);
        return (it != config_.end()) ? it->second : default_value;
    }
    
    bool getBool(const std::string& key, bool default_value = false) const {
        auto value = get(key);
        return value == "true" || value == "1";
    }
    
    int getInt(const std::string& key, int default_value = 0) const {
        auto value = get(key);
        return value.empty() ? default_value : std::stoi(value);
    }
    
    static AppConfig fromEnvironment() {
        AppConfig config;
        
        // Load from environment variables
        if (const char* debug = std::getenv("DEBUG")) {
            config.set("debug_mode", debug);
        }
        if (const char* log_level = std::getenv("LOG_LEVEL")) {
            config.set("log_level", log_level);
        }
        
        return config;
    }
};

// Utility functions
namespace utils {
    template<typename T>
    void logInfo(const T& message) {
        std::cout << "[INFO] " << message << std::endl;
    }
    
    template<typename T>
    void logError(const T& message) {
        std::cerr << "[ERROR] " << message << std::endl;
    }
    
    std::string getCurrentTimestamp() {
        auto now = std::chrono::system_clock::now();
        auto time_t = std::chrono::system_clock::to_time_t(now);
        return std::ctime(&time_t);
    }
}

// Implementation of deferred methods
std::unique_ptr<QueryResult> PostgresConnection::executeQuery(const std::string& query, 
                                                            const std::vector<std::string>& params) {
    if (status_ != ConnectionStatus::OPEN) {
        throw DatabaseException("Not connected to database");
    }
    
    auto result = std::make_unique<QueryResult>();
    
    // Simulate query execution
    std::this_thread::sleep_for(std::chrono::milliseconds(50));
    
    return result;
}

std::unique_ptr<QueryResult> MySQLConnection::executeQuery(const std::string& query, 
                                                         const std::vector<std::string>& params) {
    if (status_ != ConnectionStatus::OPEN) {
        throw DatabaseException("Not connected to database");
    }
    
    return std::make_unique<QueryResult>();
}

// Main application
int main() {
    try {
        utils::logInfo("Starting application " + constants::API_VERSION);
        
        // Load configuration
        auto config = AppConfig::fromEnvironment();
        config.set("db_type", "postgresql");
        config.set("db_host", "localhost");
        config.set("db_port", "5432");
        config.set("db_name", "myapp");
        config.set("db_user", "user");
        config.set("db_password", "password");
        
        // Create database connection
        std::unordered_map<std::string, std::string> db_config = {
            {"host", config.get("db_host")},
            {"port", config.get("db_port")},
            {"database", config.get("db_name")},
            {"username", config.get("db_user")},
            {"password", config.get("db_password")}
        };
        
        auto database = DatabaseFactory::createConnection(config.get("db_type"), db_config);
        
        if (!database->connect()) {
            utils::logError("Failed to connect to database");
            return 1;
        }
        
        // Create cache
        auto cache = std::make_shared<Cache<int, std::shared_ptr<User>>>(config.getInt("cache_size", 1000));
        
        // Create user service
        auto user_service = std::make_unique<UserService<DatabaseConnection>>(std::move(database), cache);
        
        // Create sample user asynchronously
        auto user_future = user_service->createUserAsync("john_doe", "john@example.com");
        auto user = user_future.get();
        
        if (user) {
            utils::logInfo("Created user: " + user->getUsername() + " <" + user->getEmail() + ">");
            
            // Get user
            auto retrieved_user = user_service->getUser(user->getId());
            if (retrieved_user) {
                utils::logInfo("Retrieved user: " + retrieved_user->getUsername());
            }
            
            // Update user
            std::unordered_map<std::string, std::string> updates = {
                {"email", "john.doe@example.com"}
            };
            
            if (user_service->updateUser(user->getId(), updates)) {
                utils::logInfo("User updated successfully");
            }
        }
        
        // Create sample product
        auto product = std::make_unique<Product>("Laptop", 999.99, "Electronics");
        product->addTag("computer");
        product->addTag("portable");
        
        double discounted = product->calculateDiscountedPrice(10.0);
        std::cout << "Product: " << product->getName() 
                  << ", Original: $" << product->getPrice()
                  << ", Discounted: $" << discounted << std::endl;
        
        utils::logInfo("Application completed successfully");
        
    } catch (const std::exception& e) {
        utils::logError("Application error: " + std::string(e.what()));
        return 1;
    }
    
    return 0;
}
