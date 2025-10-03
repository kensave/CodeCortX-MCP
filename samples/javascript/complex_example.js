/**
 * Complex JavaScript example with multiple constructs
 */

// Constants
const MAX_CONNECTIONS = 100;
const DEFAULT_TIMEOUT = 30000;
const API_VERSION = '1.0.0';

// Database interfaces and implementations
class DatabaseConnection {
    async connect() {
        throw new Error('Not implemented');
    }
    
    async executeQuery(query, params = []) {
        throw new Error('Not implemented');
    }
    
    async close() {
        throw new Error('Not implemented');
    }
    
    async beginTransaction() {
        throw new Error('Not implemented');
    }
    
    async commit() {
        throw new Error('Not implemented');
    }
    
    async rollback() {
        throw new Error('Not implemented');
    }
}

class PostgresConnection extends DatabaseConnection {
    constructor(config) {
        super();
        this.host = config.host;
        this.port = config.port;
        this.database = config.database;
        this.username = config.username;
        this.password = config.password;
        this.isConnected = false;
        this.transactionActive = false;
    }
    
    async connect() {
        try {
            // Simulate connection logic
            await this.delay(100);
            this.isConnected = true;
            return true;
        } catch (error) {
            console.error('Failed to connect to PostgreSQL:', error);
            return false;
        }
    }
    
    async executeQuery(query, params = []) {
        if (!this.isConnected) {
            throw new Error('Not connected to database');
        }
        
        // Simulate query execution
        await this.delay(50);
        return new QueryResult([]);
    }
    
    async close() {
        this.isConnected = false;
        this.transactionActive = false;
    }
    
    async beginTransaction() {
        this.transactionActive = true;
    }
    
    async commit() {
        this.transactionActive = false;
    }
    
    async rollback() {
        this.transactionActive = false;
    }
    
    delay(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }
}

class MySQLConnection extends DatabaseConnection {
    constructor(connectionString) {
        super();
        this.connectionString = connectionString;
        this.isConnected = false;
    }
    
    async connect() {
        this.isConnected = true;
        return true;
    }
    
    async executeQuery(query, params = []) {
        if (!this.isConnected) {
            throw new Error('Not connected to database');
        }
        return new QueryResult([]);
    }
    
    async close() {
        this.isConnected = false;
    }
    
    async beginTransaction() {}
    async commit() {}
    async rollback() {}
}

// Cache implementation
class MemoryCache {
    constructor(maxSize = 1000) {
        this.data = new Map();
        this.maxSize = maxSize;
    }
    
    get(key) {
        const item = this.data.get(key);
        if (!item) return undefined;
        
        if (item.expiry && item.expiry < Date.now()) {
            this.data.delete(key);
            return undefined;
        }
        
        return item.value;
    }
    
    set(key, value, ttl) {
        if (this.data.size >= this.maxSize && !this.data.has(key)) {
            // Remove oldest entry (simple LRU)
            const firstKey = this.data.keys().next().value;
            this.data.delete(firstKey);
        }
        
        const expiry = ttl ? Date.now() + ttl : undefined;
        this.data.set(key, { value, expiry });
    }
    
    delete(key) {
        return this.data.delete(key);
    }
    
    clear() {
        this.data.clear();
    }
}

// Data models
class User {
    constructor(id, username, email) {
        this.id = id;
        this.username = username;
        this.email = email;
        this.createdAt = new Date();
        this.updatedAt = new Date();
        this.isActive = true;
        this.metadata = {};
    }
    
    validateEmail() {
        const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
        return emailRegex.test(this.email);
    }
    
    updateTimestamp() {
        this.updatedAt = new Date();
    }
    
    toJSON() {
        return {
            id: this.id,
            username: this.username,
            email: this.email,
            createdAt: this.createdAt.toISOString(),
            updatedAt: this.updatedAt.toISOString(),
            isActive: this.isActive,
            metadata: this.metadata
        };
    }
    
    static fromJSON(data) {
        const user = new User(data.id, data.username, data.email);
        user.isActive = data.isActive ?? true;
        user.metadata = data.metadata ?? {};
        return user;
    }
}

class Product {
    constructor(id, name, price, category) {
        this.id = id;
        this.name = name;
        this.price = price;
        this.category = category;
        this.tags = [];
        this.createdAt = new Date();
    }
    
    addTag(tag) {
        if (!this.tags.includes(tag)) {
            this.tags.push(tag);
        }
    }
    
    removeTag(tag) {
        this.tags = this.tags.filter(t => t !== tag);
    }
    
    calculateDiscountedPrice(discountPercent) {
        return this.price * (1 - discountPercent / 100);
    }
}

// Service classes with decorators (using functions)
function retry(attempts = 3) {
    return function(target, propertyName, descriptor) {
        const method = descriptor.value;
        
        descriptor.value = async function(...args) {
            for (let i = 0; i < attempts; i++) {
                try {
                    return await method.apply(this, args);
                } catch (error) {
                    if (i === attempts - 1) throw error;
                    await new Promise(resolve => setTimeout(resolve, 100 * Math.pow(2, i)));
                }
            }
        };
    };
}

function logExecution(target, propertyName, descriptor) {
    const method = descriptor.value;
    
    descriptor.value = async function(...args) {
        console.log(`Executing ${propertyName}`);
        try {
            const result = await method.apply(this, args);
            console.log(`Successfully executed ${propertyName}`);
            return result;
        } catch (error) {
            console.error(`Error in ${propertyName}:`, error);
            throw error;
        }
    };
}

class UserService {
    constructor(database, cache, logger) {
        this.database = database;
        this.cache = cache;
        this.logger = logger;
    }
    
    async createUser(username, email) {
        try {
            const user = new User(Date.now(), username, email);
            
            if (!user.validateEmail()) {
                return {
                    success: false,
                    error: 'Invalid email format'
                };
            }
            
            await this.database.beginTransaction();
            
            const query = 'INSERT INTO users (username, email, created_at) VALUES ($1, $2, $3)';
            await this.database.executeQuery(query, [username, email, user.createdAt]);
            
            await this.database.commit();
            
            // Cache the user
            this.cache.set(user.id, user);
            
            this.logger.info('User created successfully', { userId: user.id, username });
            
            return {
                success: true,
                data: user
            };
        } catch (error) {
            await this.database.rollback();
            this.logger.error('Failed to create user', { error, username, email });
            
            return {
                success: false,
                error: error.message
            };
        }
    }
    
    async getUser(id) {
        try {
            // Check cache first
            const cached = this.cache.get(id);
            if (cached) {
                return { success: true, data: cached };
            }
            
            // Query database
            const query = 'SELECT * FROM users WHERE id = $1';
            const result = await this.database.executeQuery(query, [id]);
            
            if (result.rows.length === 0) {
                return {
                    success: false,
                    error: 'User not found'
                };
            }
            
            const user = User.fromJSON(result.rows[0]);
            this.cache.set(id, user);
            
            return { success: true, data: user };
        } catch (error) {
            this.logger.error('Failed to get user', { error, userId: id });
            return {
                success: false,
                error: error.message
            };
        }
    }
    
    async updateUser(id, updates) {
        const userResult = await this.getUser(id);
        if (!userResult.success) {
            return userResult;
        }
        
        const user = userResult.data;
        
        // Apply updates
        if (updates.username !== undefined) user.username = updates.username;
        if (updates.email !== undefined) user.email = updates.email;
        if (updates.isActive !== undefined) user.isActive = updates.isActive;
        
        user.updateTimestamp();
        
        // Update cache
        this.cache.set(id, user);
        
        return { success: true, data: user };
    }
}

class ProductService {
    constructor(database, cache) {
        this.database = database;
        this.cache = cache;
    }
    
    async createProduct(name, price, category) {
        const product = new Product(Date.now(), name, price, category);
        
        // Save to database
        const query = 'INSERT INTO products (name, price, category) VALUES ($1, $2, $3)';
        await this.database.executeQuery(query, [name, price, category]);
        
        this.cache.set(product.id, product);
        return product;
    }
    
    async searchProducts(searchQuery, categoryFilter = null) {
        let query = 'SELECT * FROM products WHERE name ILIKE $1';
        const params = [`%${searchQuery}%`];
        
        if (categoryFilter) {
            query += ' AND category = $2';
            params.push(categoryFilter);
        }
        
        const result = await this.database.executeQuery(query, params);
        return result.rows.map(row => Product.fromJSON ? Product.fromJSON(row) : row);
    }
}

// Configuration management
class AppConfig {
    constructor() {
        this.debugMode = false;
        this.logLevel = 'INFO';
        this.databaseConfig = {
            host: 'localhost',
            port: 5432,
            database: 'myapp',
            username: 'user',
            password: 'password'
        };
        this.cacheSize = 1000;
        this.maxConnections = MAX_CONNECTIONS;
    }
    
    static fromEnvironment() {
        const config = new AppConfig();
        
        config.debugMode = process.env.DEBUG === 'true';
        config.logLevel = process.env.LOG_LEVEL || 'INFO';
        config.cacheSize = parseInt(process.env.CACHE_SIZE || '1000');
        
        if (process.env.DB_HOST) config.databaseConfig.host = process.env.DB_HOST;
        if (process.env.DB_PORT) config.databaseConfig.port = parseInt(process.env.DB_PORT);
        if (process.env.DB_NAME) config.databaseConfig.database = process.env.DB_NAME;
        if (process.env.DB_USER) config.databaseConfig.username = process.env.DB_USER;
        if (process.env.DB_PASSWORD) config.databaseConfig.password = process.env.DB_PASSWORD;
        
        return config;
    }
}

// Logger implementation
class ConsoleLogger {
    constructor(level = 'INFO') {
        this.level = level;
    }
    
    info(message, meta = {}) {
        if (this.shouldLog('INFO')) {
            console.log(`[INFO] ${message}`, meta);
        }
    }
    
    error(message, meta = {}) {
        if (this.shouldLog('ERROR')) {
            console.error(`[ERROR] ${message}`, meta);
        }
    }
    
    debug(message, meta = {}) {
        if (this.shouldLog('DEBUG')) {
            console.log(`[DEBUG] ${message}`, meta);
        }
    }
    
    warn(message, meta = {}) {
        if (this.shouldLog('WARN')) {
            console.warn(`[WARN] ${message}`, meta);
        }
    }
    
    shouldLog(level) {
        const levels = ['DEBUG', 'INFO', 'WARN', 'ERROR'];
        return levels.indexOf(level) >= levels.indexOf(this.level);
    }
}

// Query result class
class QueryResult {
    constructor(rows = []) {
        this.rows = rows;
        this.rowCount = rows.length;
    }
}

// Exception classes
class DatabaseError extends Error {
    constructor(message, originalError = null) {
        super(message);
        this.name = 'DatabaseError';
        this.originalError = originalError;
    }
}

class ValidationError extends Error {
    constructor(message, field = null) {
        super(message);
        this.name = 'ValidationError';
        this.field = field;
    }
}

class ServiceError extends Error {
    constructor(message, code = null) {
        super(message);
        this.name = 'ServiceError';
        this.code = code;
    }
}

// Utility functions
function initializeDatabase(config) {
    const dbConfig = config.databaseConfig;
    
    if (dbConfig.host.includes('postgres')) {
        return new PostgresConnection(dbConfig);
    } else {
        return new MySQLConnection(`mysql://${dbConfig.username}:${dbConfig.password}@${dbConfig.host}:${dbConfig.port}/${dbConfig.database}`);
    }
}

function createLogger(level = 'INFO') {
    return new ConsoleLogger(level);
}

// Main application
async function main() {
    try {
        const config = AppConfig.fromEnvironment();
        const logger = createLogger(config.logLevel);
        const database = initializeDatabase(config);
        const cache = new MemoryCache(config.cacheSize);
        
        await database.connect();
        
        const userService = new UserService(database, cache, logger);
        const productService = new ProductService(database, cache);
        
        // Create sample user
        const createResult = await userService.createUser('john_doe', 'john@example.com');
        if (createResult.success) {
            logger.info('Created user successfully', createResult.data.toJSON());
            
            // Get user
            const getResult = await userService.getUser(createResult.data.id);
            if (getResult.success) {
                logger.info('Retrieved user successfully', getResult.data.toJSON());
            }
        }
        
        // Create sample product
        const product = await productService.createProduct('Laptop', 999.99, 'Electronics');
        logger.info('Created product successfully', { productId: product.id });
        
    } catch (error) {
        console.error('Application error:', error);
    }
}

// Export for module usage
if (typeof module !== 'undefined' && module.exports) {
    module.exports = {
        DatabaseConnection,
        PostgresConnection,
        MySQLConnection,
        MemoryCache,
        User,
        Product,
        UserService,
        ProductService,
        AppConfig,
        ConsoleLogger,
        QueryResult,
        DatabaseError,
        ValidationError,
        ServiceError,
        initializeDatabase,
        createLogger,
        main,
        MAX_CONNECTIONS,
        DEFAULT_TIMEOUT,
        API_VERSION
    };
}

// Run if this is the main module
if (typeof require !== 'undefined' && require.main === module) {
    main().catch(console.error);
}
