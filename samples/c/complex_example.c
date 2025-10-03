#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>

// Constants
#define MAX_CONNECTIONS 100
#define DEFAULT_TIMEOUT 30
#define BUFFER_SIZE 1024
#define API_VERSION "1.0.0"

// Forward declarations
typedef struct Connection Connection;
typedef struct User User;
typedef struct Product Product;
typedef struct QueryResult QueryResult;

// Enums
typedef enum {
    CONNECTION_CLOSED,
    CONNECTION_OPEN,
    CONNECTION_ERROR
} ConnectionStatus;

typedef enum {
    QUERY_SUCCESS,
    QUERY_ERROR,
    QUERY_TIMEOUT
} QueryStatus;

typedef enum {
    USER_ACTIVE,
    USER_INACTIVE,
    USER_SUSPENDED
} UserStatus;

// Structs
struct Connection {
    char* host;
    int port;
    char* database;
    char* username;
    char* password;
    ConnectionStatus status;
    bool transaction_active;
};

struct User {
    int id;
    char* username;
    char* email;
    time_t created_at;
    time_t updated_at;
    UserStatus status;
    char* metadata;
};

struct Product {
    int id;
    char* name;
    double price;
    char* category;
    char* description;
    char** tags;
    int tag_count;
    time_t created_at;
};

struct QueryResult {
    char** rows;
    int row_count;
    int column_count;
    QueryStatus status;
    char* error_message;
};

// Function prototypes
Connection* create_connection(const char* host, int port, const char* database, 
                            const char* username, const char* password);
bool connect_to_database(Connection* conn);
void close_connection(Connection* conn);
void free_connection(Connection* conn);

QueryResult* execute_query(Connection* conn, const char* query, const char** params, int param_count);
void free_query_result(QueryResult* result);

User* create_user(const char* username, const char* email);
bool validate_user_email(const User* user);
void update_user_timestamp(User* user);
char* user_to_json(const User* user);
User* user_from_json(const char* json);
void free_user(User* user);

Product* create_product(const char* name, double price, const char* category);
void add_product_tag(Product* product, const char* tag);
void remove_product_tag(Product* product, const char* tag);
double calculate_discounted_price(const Product* product, double discount_percent);
void free_product(Product* product);

// Cache functions
typedef struct CacheItem {
    char* key;
    void* value;
    time_t expiry;
    struct CacheItem* next;
} CacheItem;

typedef struct Cache {
    CacheItem** buckets;
    int bucket_count;
    int max_size;
    int current_size;
} Cache;

Cache* create_cache(int max_size);
void cache_set(Cache* cache, const char* key, void* value, int ttl_seconds);
void* cache_get(Cache* cache, const char* key);
void cache_delete(Cache* cache, const char* key);
void free_cache(Cache* cache);

// Service functions
typedef struct UserService {
    Connection* database;
    Cache* cache;
} UserService;

UserService* create_user_service(Connection* database, Cache* cache);
User* user_service_create_user(UserService* service, const char* username, const char* email);
User* user_service_get_user(UserService* service, int id);
bool user_service_update_user(UserService* service, int id, const char* username, const char* email);
void free_user_service(UserService* service);

// Utility functions
char* string_duplicate(const char* str);
bool string_contains(const char* str, const char* substr);
void log_info(const char* message, const char* context);
void log_error(const char* message, const char* context);
int hash_string(const char* str, int bucket_count);

// Implementation

Connection* create_connection(const char* host, int port, const char* database, 
                            const char* username, const char* password) {
    Connection* conn = malloc(sizeof(Connection));
    if (!conn) return NULL;
    
    conn->host = string_duplicate(host);
    conn->port = port;
    conn->database = string_duplicate(database);
    conn->username = string_duplicate(username);
    conn->password = string_duplicate(password);
    conn->status = CONNECTION_CLOSED;
    conn->transaction_active = false;
    
    return conn;
}

bool connect_to_database(Connection* conn) {
    if (!conn) return false;
    
    printf("Connecting to %s:%d/%s as %s\n", conn->host, conn->port, conn->database, conn->username);
    
    // Simulate connection logic
    conn->status = CONNECTION_OPEN;
    return true;
}

void close_connection(Connection* conn) {
    if (conn) {
        conn->status = CONNECTION_CLOSED;
        conn->transaction_active = false;
    }
}

void free_connection(Connection* conn) {
    if (conn) {
        free(conn->host);
        free(conn->database);
        free(conn->username);
        free(conn->password);
        free(conn);
    }
}

QueryResult* execute_query(Connection* conn, const char* query, const char** params, int param_count) {
    if (!conn || conn->status != CONNECTION_OPEN) {
        return NULL;
    }
    
    QueryResult* result = malloc(sizeof(QueryResult));
    if (!result) return NULL;
    
    // Simulate query execution
    result->rows = NULL;
    result->row_count = 0;
    result->column_count = 0;
    result->status = QUERY_SUCCESS;
    result->error_message = NULL;
    
    return result;
}

void free_query_result(QueryResult* result) {
    if (result) {
        if (result->rows) {
            for (int i = 0; i < result->row_count; i++) {
                free(result->rows[i]);
            }
            free(result->rows);
        }
        free(result->error_message);
        free(result);
    }
}

User* create_user(const char* username, const char* email) {
    User* user = malloc(sizeof(User));
    if (!user) return NULL;
    
    user->id = (int)time(NULL); // Simple ID generation
    user->username = string_duplicate(username);
    user->email = string_duplicate(email);
    user->created_at = time(NULL);
    user->updated_at = time(NULL);
    user->status = USER_ACTIVE;
    user->metadata = string_duplicate("{}");
    
    return user;
}

bool validate_user_email(const User* user) {
    if (!user || !user->email) return false;
    return string_contains(user->email, "@") && string_contains(user->email, ".");
}

void update_user_timestamp(User* user) {
    if (user) {
        user->updated_at = time(NULL);
    }
}

void free_user(User* user) {
    if (user) {
        free(user->username);
        free(user->email);
        free(user->metadata);
        free(user);
    }
}

Product* create_product(const char* name, double price, const char* category) {
    Product* product = malloc(sizeof(Product));
    if (!product) return NULL;
    
    product->id = (int)time(NULL);
    product->name = string_duplicate(name);
    product->price = price;
    product->category = string_duplicate(category);
    product->description = NULL;
    product->tags = NULL;
    product->tag_count = 0;
    product->created_at = time(NULL);
    
    return product;
}

void add_product_tag(Product* product, const char* tag) {
    if (!product || !tag) return;
    
    // Reallocate tags array
    product->tags = realloc(product->tags, (product->tag_count + 1) * sizeof(char*));
    if (product->tags) {
        product->tags[product->tag_count] = string_duplicate(tag);
        product->tag_count++;
    }
}

double calculate_discounted_price(const Product* product, double discount_percent) {
    if (!product) return 0.0;
    return product->price * (1.0 - discount_percent / 100.0);
}

void free_product(Product* product) {
    if (product) {
        free(product->name);
        free(product->category);
        free(product->description);
        
        if (product->tags) {
            for (int i = 0; i < product->tag_count; i++) {
                free(product->tags[i]);
            }
            free(product->tags);
        }
        
        free(product);
    }
}

Cache* create_cache(int max_size) {
    Cache* cache = malloc(sizeof(Cache));
    if (!cache) return NULL;
    
    cache->bucket_count = max_size / 10; // Simple bucket calculation
    cache->buckets = calloc(cache->bucket_count, sizeof(CacheItem*));
    cache->max_size = max_size;
    cache->current_size = 0;
    
    return cache;
}

void cache_set(Cache* cache, const char* key, void* value, int ttl_seconds) {
    if (!cache || !key) return;
    
    int bucket_index = hash_string(key, cache->bucket_count);
    
    CacheItem* item = malloc(sizeof(CacheItem));
    if (!item) return;
    
    item->key = string_duplicate(key);
    item->value = value;
    item->expiry = time(NULL) + ttl_seconds;
    item->next = cache->buckets[bucket_index];
    
    cache->buckets[bucket_index] = item;
    cache->current_size++;
}

void* cache_get(Cache* cache, const char* key) {
    if (!cache || !key) return NULL;
    
    int bucket_index = hash_string(key, cache->bucket_count);
    CacheItem* item = cache->buckets[bucket_index];
    
    while (item) {
        if (strcmp(item->key, key) == 0) {
            if (item->expiry > time(NULL)) {
                return item->value;
            }
        }
        item = item->next;
    }
    
    return NULL;
}

UserService* create_user_service(Connection* database, Cache* cache) {
    UserService* service = malloc(sizeof(UserService));
    if (!service) return NULL;
    
    service->database = database;
    service->cache = cache;
    
    return service;
}

User* user_service_create_user(UserService* service, const char* username, const char* email) {
    if (!service || !username || !email) return NULL;
    
    User* user = create_user(username, email);
    if (!user) return NULL;
    
    if (!validate_user_email(user)) {
        log_error("Invalid email format", email);
        free_user(user);
        return NULL;
    }
    
    // Simulate database insert
    const char* query = "INSERT INTO users (username, email) VALUES (?, ?)";
    const char* params[] = {username, email};
    QueryResult* result = execute_query(service->database, query, params, 2);
    
    if (result && result->status == QUERY_SUCCESS) {
        // Cache the user
        char cache_key[64];
        snprintf(cache_key, sizeof(cache_key), "user:%d", user->id);
        cache_set(service->cache, cache_key, user, 3600); // 1 hour TTL
        
        log_info("User created successfully", username);
    }
    
    free_query_result(result);
    return user;
}

User* user_service_get_user(UserService* service, int id) {
    if (!service) return NULL;
    
    // Check cache first
    char cache_key[64];
    snprintf(cache_key, sizeof(cache_key), "user:%d", id);
    
    User* cached_user = (User*)cache_get(service->cache, cache_key);
    if (cached_user) {
        return cached_user;
    }
    
    // Query database
    const char* query = "SELECT * FROM users WHERE id = ?";
    char id_str[32];
    snprintf(id_str, sizeof(id_str), "%d", id);
    const char* params[] = {id_str};
    
    QueryResult* result = execute_query(service->database, query, params, 1);
    
    User* user = NULL;
    if (result && result->status == QUERY_SUCCESS && result->row_count > 0) {
        // Simulate user creation from database row
        user = create_user("username", "email@example.com");
        if (user) {
            user->id = id;
            cache_set(service->cache, cache_key, user, 3600);
        }
    }
    
    free_query_result(result);
    return user;
}

// Utility function implementations
char* string_duplicate(const char* str) {
    if (!str) return NULL;
    
    size_t len = strlen(str) + 1;
    char* dup = malloc(len);
    if (dup) {
        strcpy(dup, str);
    }
    return dup;
}

bool string_contains(const char* str, const char* substr) {
    if (!str || !substr) return false;
    return strstr(str, substr) != NULL;
}

void log_info(const char* message, const char* context) {
    printf("[INFO] %s: %s\n", message, context ? context : "");
}

void log_error(const char* message, const char* context) {
    fprintf(stderr, "[ERROR] %s: %s\n", message, context ? context : "");
}

int hash_string(const char* str, int bucket_count) {
    if (!str) return 0;
    
    unsigned int hash = 0;
    while (*str) {
        hash = hash * 31 + *str++;
    }
    return hash % bucket_count;
}

// Main function
int main() {
    log_info("Starting application", API_VERSION);
    
    // Create database connection
    Connection* conn = create_connection("localhost", 5432, "myapp", "user", "password");
    if (!conn) {
        log_error("Failed to create connection", NULL);
        return 1;
    }
    
    if (!connect_to_database(conn)) {
        log_error("Failed to connect to database", NULL);
        free_connection(conn);
        return 1;
    }
    
    // Create cache
    Cache* cache = create_cache(1000);
    if (!cache) {
        log_error("Failed to create cache", NULL);
        close_connection(conn);
        free_connection(conn);
        return 1;
    }
    
    // Create user service
    UserService* user_service = create_user_service(conn, cache);
    if (!user_service) {
        log_error("Failed to create user service", NULL);
        free_cache(cache);
        close_connection(conn);
        free_connection(conn);
        return 1;
    }
    
    // Create sample user
    User* user = user_service_create_user(user_service, "john_doe", "john@example.com");
    if (user) {
        printf("Created user: %s <%s>\n", user->username, user->email);
        
        // Get user
        User* retrieved_user = user_service_get_user(user_service, user->id);
        if (retrieved_user) {
            printf("Retrieved user: %s\n", retrieved_user->username);
        }
    }
    
    // Create sample product
    Product* product = create_product("Laptop", 999.99, "Electronics");
    if (product) {
        add_product_tag(product, "computer");
        add_product_tag(product, "portable");
        
        double discounted = calculate_discounted_price(product, 10.0);
        printf("Product: %s, Original: $%.2f, Discounted: $%.2f\n", 
               product->name, product->price, discounted);
        
        free_product(product);
    }
    
    // Cleanup
    free_user_service(user_service);
    free_cache(cache);
    close_connection(conn);
    free_connection(conn);
    
    log_info("Application completed", NULL);
    return 0;
}
