/**
 * Complex TypeScript example with multiple constructs
 */

import { EventEmitter } from 'events';

// Type definitions and interfaces
interface DatabaseConnection {
    connect(): Promise<boolean>;
    executeQuery(query: string, params?: any[]): Promise<QueryResult>;
    close(): Promise<void>;
    beginTransaction(): Promise<void>;
    commit(): Promise<void>;
    rollback(): Promise<void>;
}

interface CacheInterface<K, V> {
    get(key: K): V | undefined;
    set(key: K, value: V, ttl?: number): void;
    delete(key: K): boolean;
    clear(): void;
}

interface LoggerInterface {
    info(message: string, meta?: any): void;
    error(message: string, meta?: any): void;
    debug(message: string, meta?: any): void;
    warn(message: string, meta?: any): void;
}

// Type aliases and generics
type UserId = number;
type ProductId = number;
type Timestamp = Date;

type ServiceResult<T> = {
    success: boolean;
    data?: T;
    error?: string;
};

type DatabaseConfig = {
    host: string;
    port: number;
    database: string;
    username: string;
    password: string;
    ssl?: boolean;
};

// Enums
enum ProductCategory {
    Electronics = 'electronics',
    Clothing = 'clothing',
    Books = 'books',
    Home = 'home',
    Sports = 'sports'
}

enum LogLevel {
    DEBUG = 'debug',
    INFO = 'info',
    WARN = 'warn',
    ERROR = 'error'
}

// Constants
export const MAX_CONNECTIONS = 100;
export const DEFAULT_TIMEOUT = 30000;
export const API_VERSION = '1.0.0';

// Database implementations
class PostgresConnection implements DatabaseConnection {
    private config: DatabaseConfig;
    private isConnected: boolean = false;
    private transactionActive: boolean = false;

    constructor(config: DatabaseConfig) {
        this.config = config;
    }

    async connect(): Promise<boolean> {
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

    async executeQuery(query: string, params: any[] = []): Promise<QueryResult> {
        if (!this.isConnected) {
            throw new Error('Not connected to database');
        }

        // Simulate query execution
        await this.delay(50);
        return new QueryResult([]);
    }

    async close(): Promise<void> {
        this.isConnected = false;
        this.transactionActive = false;
    }

    async beginTransaction(): Promise<void> {
        this.transactionActive = true;
    }

    async commit(): Promise<void> {
        this.transactionActive = false;
    }

    async rollback(): Promise<void> {
        this.transactionActive = false;
    }

    private delay(ms: number): Promise<void> {
        return new Promise(resolve => setTimeout(resolve, ms));
    }
}

class MySQLConnection implements DatabaseConnection {
    private connectionString: string;

    constructor(connectionString: string) {
        this.connectionString = connectionString;
    }

    async connect(): Promise<boolean> {
        return true;
    }

    async executeQuery(query: string, params?: any[]): Promise<QueryResult> {
        return new QueryResult([]);
    }

    async close(): Promise<void> {}
    async beginTransaction(): Promise<void> {}
    async commit(): Promise<void> {}
    async rollback(): Promise<void> {}
}

// Generic cache implementation
class MemoryCache<K, V> implements CacheInterface<K, V> {
    private data: Map<K, { value: V; expiry?: number }> = new Map();
    private maxSize: number;

    constructor(maxSize: number = 1000) {
        this.maxSize = maxSize;
    }

    get(key: K): V | undefined {
        const item = this.data.get(key);
        if (!item) return undefined;

        if (item.expiry && item.expiry < Date.now()) {
            this.data.delete(key);
            return undefined;
        }

        return item.value;
    }

    set(key: K, value: V, ttl?: number): void {
        if (this.data.size >= this.maxSize && !this.data.has(key)) {
            // Remove oldest entry (simple LRU)
            const firstKey = this.data.keys().next().value;
            this.data.delete(firstKey);
        }

        const expiry = ttl ? Date.now() + ttl : undefined;
        this.data.set(key, { value, expiry });
    }

    delete(key: K): boolean {
        return this.data.delete(key);
    }

    clear(): void {
        this.data.clear();
    }
}

// Data models
class User {
    public readonly id: UserId;
    public username: string;
    public email: string;
    public readonly createdAt: Timestamp;
    public updatedAt: Timestamp;
    public isActive: boolean;
    public metadata: Record<string, any>;

    constructor(id: UserId, username: string, email: string) {
        this.id = id;
        this.username = username;
        this.email = email;
        this.createdAt = new Date();
        this.updatedAt = new Date();
        this.isActive = true;
        this.metadata = {};
    }

    validateEmail(): boolean {
        const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
        return emailRegex.test(this.email);
    }

    updateTimestamp(): void {
        this.updatedAt = new Date();
    }

    toJSON(): object {
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

    static fromJSON(data: any): User {
        const user = new User(data.id, data.username, data.email);
        user.isActive = data.isActive ?? true;
        user.metadata = data.metadata ?? {};
        return user;
    }
}

class Product {
    public readonly id: ProductId;
    public name: string;
    public price: number;
    public category: ProductCategory;
    public description?: string;
    public tags: string[];
    public readonly createdAt: Timestamp;

    constructor(id: ProductId, name: string, price: number, category: ProductCategory) {
        this.id = id;
        this.name = name;
        this.price = price;
        this.category = category;
        this.tags = [];
        this.createdAt = new Date();
    }

    addTag(tag: string): void {
        if (!this.tags.includes(tag)) {
            this.tags.push(tag);
        }
    }

    removeTag(tag: string): void {
        this.tags = this.tags.filter(t => t !== tag);
    }
}

// Decorators
function retry(attempts: number = 3) {
    return function (target: any, propertyName: string, descriptor: PropertyDescriptor) {
        const method = descriptor.value;

        descriptor.value = async function (...args: any[]) {
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

function logExecution(target: any, propertyName: string, descriptor: PropertyDescriptor) {
    const method = descriptor.value;

    descriptor.value = async function (...args: any[]) {
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

// Service classes
class UserService<T extends DatabaseConnection> extends EventEmitter {
    private database: T;
    private cache: CacheInterface<UserId, User>;
    private logger: LoggerInterface;

    constructor(database: T, cache: CacheInterface<UserId, User>, logger: LoggerInterface) {
        super();
        this.database = database;
        this.cache = cache;
        this.logger = logger;
    }

    @retry(3)
    @logExecution
    async createUser(username: string, email: string): Promise<ServiceResult<User>> {
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

            this.emit('userCreated', user);
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
                error: error instanceof Error ? error.message : 'Unknown error'
            };
        }
    }

    @logExecution
    async getUser(id: UserId): Promise<ServiceResult<User>> {
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
                error: error instanceof Error ? error.message : 'Unknown error'
            };
        }
    }

    async updateUser(id: UserId, updates: Partial<Pick<User, 'username' | 'email' | 'isActive'>>): Promise<ServiceResult<User>> {
        const userResult = await this.getUser(id);
        if (!userResult.success || !userResult.data) {
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

        this.emit('userUpdated', user);
        return { success: true, data: user };
    }
}

// Configuration management
class AppConfig {
    public debugMode: boolean = false;
    public logLevel: LogLevel = LogLevel.INFO;
    public databaseConfig: DatabaseConfig;
    public cacheSize: number = 1000;
    public maxConnections: number = MAX_CONNECTIONS;

    constructor(databaseConfig: DatabaseConfig) {
        this.databaseConfig = databaseConfig;
    }

    static fromEnvironment(): AppConfig {
        const config = new AppConfig({
            host: process.env.DB_HOST || 'localhost',
            port: parseInt(process.env.DB_PORT || '5432'),
            database: process.env.DB_NAME || 'myapp',
            username: process.env.DB_USER || 'user',
            password: process.env.DB_PASSWORD || 'password'
        });

        config.debugMode = process.env.DEBUG === 'true';
        config.logLevel = (process.env.LOG_LEVEL as LogLevel) || LogLevel.INFO;
        config.cacheSize = parseInt(process.env.CACHE_SIZE || '1000');

        return config;
    }
}

// Logger implementation
class ConsoleLogger implements LoggerInterface {
    private level: LogLevel;

    constructor(level: LogLevel = LogLevel.INFO) {
        this.level = level;
    }

    info(message: string, meta?: any): void {
        if (this.shouldLog(LogLevel.INFO)) {
            console.log(`[INFO] ${message}`, meta || '');
        }
    }

    error(message: string, meta?: any): void {
        if (this.shouldLog(LogLevel.ERROR)) {
            console.error(`[ERROR] ${message}`, meta || '');
        }
    }

    debug(message: string, meta?: any): void {
        if (this.shouldLog(LogLevel.DEBUG)) {
            console.log(`[DEBUG] ${message}`, meta || '');
        }
    }

    warn(message: string, meta?: any): void {
        if (this.shouldLog(LogLevel.WARN)) {
            console.warn(`[WARN] ${message}`, meta || '');
        }
    }

    private shouldLog(level: LogLevel): boolean {
        const levels = [LogLevel.DEBUG, LogLevel.INFO, LogLevel.WARN, LogLevel.ERROR];
        return levels.indexOf(level) >= levels.indexOf(this.level);
    }
}

// Query result class
class QueryResult {
    public readonly rows: any[];
    public readonly rowCount: number;

    constructor(rows: any[]) {
        this.rows = rows;
        this.rowCount = rows.length;
    }
}

// Exception classes
class DatabaseError extends Error {
    constructor(message: string, public readonly originalError?: Error) {
        super(message);
        this.name = 'DatabaseError';
    }
}

class ValidationError extends Error {
    constructor(message: string, public readonly field?: string) {
        super(message);
        this.name = 'ValidationError';
    }
}

class ServiceError extends Error {
    constructor(message: string, public readonly code?: string) {
        super(message);
        this.name = 'ServiceError';
    }
}

// Utility functions
export function initializeDatabase(config: AppConfig): DatabaseConnection {
    const dbConfig = config.databaseConfig;
    
    if (dbConfig.host.includes('postgres')) {
        return new PostgresConnection(dbConfig);
    } else {
        return new MySQLConnection(`mysql://${dbConfig.username}:${dbConfig.password}@${dbConfig.host}:${dbConfig.port}/${dbConfig.database}`);
    }
}

export function createLogger(level: LogLevel = LogLevel.INFO): LoggerInterface {
    return new ConsoleLogger(level);
}

// Main application
export async function main(): Promise<void> {
    try {
        const config = AppConfig.fromEnvironment();
        const logger = createLogger(config.logLevel);
        const database = initializeDatabase(config);
        const cache = new MemoryCache<UserId, User>(config.cacheSize);

        await database.connect();

        const userService = new UserService(database, cache, logger);

        // Set up event listeners
        userService.on('userCreated', (user: User) => {
            logger.info('User created event', { userId: user.id });
        });

        userService.on('userUpdated', (user: User) => {
            logger.info('User updated event', { userId: user.id });
        });

        // Create sample user
        const createResult = await userService.createUser('john_doe', 'john@example.com');
        if (createResult.success && createResult.data) {
            logger.info('Created user successfully', createResult.data.toJSON());

            // Get user
            const getResult = await userService.getUser(createResult.data.id);
            if (getResult.success && getResult.data) {
                logger.info('Retrieved user successfully', getResult.data.toJSON());
            }
        }

    } catch (error) {
        console.error('Application error:', error);
    }
}

// Export types and classes for external use
export {
    DatabaseConnection,
    CacheInterface,
    LoggerInterface,
    User,
    Product,
    ProductCategory,
    UserService,
    AppConfig,
    ConsoleLogger,
    MemoryCache,
    PostgresConnection,
    MySQLConnection
};

// Run if this is the main module
if (require.main === module) {
    main().catch(console.error);
}
