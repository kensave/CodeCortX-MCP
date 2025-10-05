<?php
/**
 * Complex PHP example with multiple constructs
 */

declare(strict_types=1);

namespace App\Database;

use DateTime;
use Exception;
use InvalidArgumentException;
use PDO;
use PDOException;

// Constants
const MAX_CONNECTIONS = 100;
const DEFAULT_TIMEOUT = 30;
const API_VERSION = '1.0.0';

// Interfaces
interface DatabaseConnectionInterface
{
    public function connect(): bool;
    public function executeQuery(string $query, array $params = []): array;
    public function close(): void;
    public function beginTransaction(): bool;
    public function commit(): bool;
    public function rollback(): bool;
}

interface CacheInterface
{
    public function get(string $key): mixed;
    public function set(string $key, mixed $value, int $ttl = 3600): bool;
    public function delete(string $key): bool;
    public function clear(): bool;
}

interface LoggerInterface
{
    public function info(string $message, array $context = []): void;
    public function error(string $message, array $context = []): void;
    public function debug(string $message, array $context = []): void;
}

// Traits
trait TimestampTrait
{
    protected DateTime $createdAt;
    protected DateTime $updatedAt;
    
    public function getCreatedAt(): DateTime
    {
        return $this->createdAt;
    }
    
    public function getUpdatedAt(): DateTime
    {
        return $this->updatedAt;
    }
    
    public function updateTimestamp(): void
    {
        $this->updatedAt = new DateTime();
    }
    
    protected function initializeTimestamps(): void
    {
        $this->createdAt = new DateTime();
        $this->updatedAt = new DateTime();
    }
}

trait ValidationTrait
{
    protected array $errors = [];
    
    public function getErrors(): array
    {
        return $this->errors;
    }
    
    public function hasErrors(): bool
    {
        return !empty($this->errors);
    }
    
    protected function addError(string $field, string $message): void
    {
        $this->errors[$field][] = $message;
    }
    
    protected function clearErrors(): void
    {
        $this->errors = [];
    }
    
    protected function validateEmail(string $email): bool
    {
        if (!filter_var($email, FILTER_VALIDATE_EMAIL)) {
            $this->addError('email', 'Invalid email format');
            return false;
        }
        return true;
    }
    
    protected function validateRequired(string $field, mixed $value): bool
    {
        if (empty($value)) {
            $this->addError($field, "Field {$field} is required");
            return false;
        }
        return true;
    }
}

// Abstract classes
abstract class BaseModel
{
    use TimestampTrait, ValidationTrait;
    
    protected int $id;
    
    public function __construct()
    {
        $this->initializeTimestamps();
    }
    
    public function getId(): int
    {
        return $this->id;
    }
    
    public function setId(int $id): void
    {
        $this->id = $id;
    }
    
    abstract public function toArray(): array;
    abstract public function validate(): bool;
}

// Database implementations
class PostgresConnection implements DatabaseConnectionInterface
{
    private PDO $connection;
    private string $host;
    private int $port;
    private string $database;
    private string $username;
    private string $password;
    private bool $isConnected = false;
    
    public function __construct(
        string $host,
        int $port,
        string $database,
        string $username,
        string $password
    ) {
        $this->host = $host;
        $this->port = $port;
        $this->database = $database;
        $this->username = $username;
        $this->password = $password;
    }
    
    public function connect(): bool
    {
        try {
            $dsn = "pgsql:host={$this->host};port={$this->port};dbname={$this->database}";
            $this->connection = new PDO($dsn, $this->username, $this->password, [
                PDO::ATTR_ERRMODE => PDO::ERRMODE_EXCEPTION,
                PDO::ATTR_DEFAULT_FETCH_MODE => PDO::FETCH_ASSOC,
            ]);
            $this->isConnected = true;
            return true;
        } catch (PDOException $e) {
            error_log("PostgreSQL connection failed: " . $e->getMessage());
            return false;
        }
    }
    
    public function executeQuery(string $query, array $params = []): array
    {
        if (!$this->isConnected) {
            throw new Exception("Not connected to database");
        }
        
        try {
            $stmt = $this->connection->prepare($query);
            $stmt->execute($params);
            return $stmt->fetchAll();
        } catch (PDOException $e) {
            throw new Exception("Query execution failed: " . $e->getMessage());
        }
    }
    
    public function close(): void
    {
        $this->connection = null;
        $this->isConnected = false;
    }
    
    public function beginTransaction(): bool
    {
        return $this->connection->beginTransaction();
    }
    
    public function commit(): bool
    {
        return $this->connection->commit();
    }
    
    public function rollback(): bool
    {
        return $this->connection->rollBack();
    }
}

class MySQLConnection implements DatabaseConnectionInterface
{
    private PDO $connection;
    private string $connectionString;
    
    public function __construct(string $connectionString)
    {
        $this->connectionString = $connectionString;
    }
    
    public function connect(): bool
    {
        try {
            $this->connection = new PDO($this->connectionString);
            return true;
        } catch (PDOException $e) {
            return false;
        }
    }
    
    public function executeQuery(string $query, array $params = []): array
    {
        $stmt = $this->connection->prepare($query);
        $stmt->execute($params);
        return $stmt->fetchAll();
    }
    
    public function close(): void
    {
        $this->connection = null;
    }
    
    public function beginTransaction(): bool
    {
        return $this->connection->beginTransaction();
    }
    
    public function commit(): bool
    {
        return $this->connection->commit();
    }
    
    public function rollback(): bool
    {
        return $this->connection->rollBack();
    }
}

// Cache implementation
class MemoryCache implements CacheInterface
{
    private array $data = [];
    private array $expiry = [];
    
    public function get(string $key): mixed
    {
        if (!isset($this->data[$key])) {
            return null;
        }
        
        if (isset($this->expiry[$key]) && $this->expiry[$key] < time()) {
            unset($this->data[$key], $this->expiry[$key]);
            return null;
        }
        
        return $this->data[$key];
    }
    
    public function set(string $key, mixed $value, int $ttl = 3600): bool
    {
        $this->data[$key] = $value;
        $this->expiry[$key] = time() + $ttl;
        return true;
    }
    
    public function delete(string $key): bool
    {
        unset($this->data[$key], $this->expiry[$key]);
        return true;
    }
    
    public function clear(): bool
    {
        $this->data = [];
        $this->expiry = [];
        return true;
    }
}

// Model classes
class User extends BaseModel
{
    private string $username;
    private string $email;
    private bool $isActive = true;
    private array $metadata = [];
    
    public function __construct(string $username, string $email)
    {
        parent::__construct();
        $this->username = $username;
        $this->email = $email;
    }
    
    public function getUsername(): string
    {
        return $this->username;
    }
    
    public function setUsername(string $username): void
    {
        $this->username = $username;
        $this->updateTimestamp();
    }
    
    public function getEmail(): string
    {
        return $this->email;
    }
    
    public function setEmail(string $email): void
    {
        $this->email = $email;
        $this->updateTimestamp();
    }
    
    public function isActive(): bool
    {
        return $this->isActive;
    }
    
    public function setActive(bool $isActive): void
    {
        $this->isActive = $isActive;
        $this->updateTimestamp();
    }
    
    public function getMetadata(): array
    {
        return $this->metadata;
    }
    
    public function setMetadata(array $metadata): void
    {
        $this->metadata = $metadata;
        $this->updateTimestamp();
    }
    
    public function validate(): bool
    {
        $this->clearErrors();
        
        $isValid = true;
        $isValid &= $this->validateRequired('username', $this->username);
        $isValid &= $this->validateRequired('email', $this->email);
        $isValid &= $this->validateEmail($this->email);
        
        return $isValid;
    }
    
    public function toArray(): array
    {
        return [
            'id' => $this->id,
            'username' => $this->username,
            'email' => $this->email,
            'is_active' => $this->isActive,
            'metadata' => $this->metadata,
            'created_at' => $this->createdAt->format('Y-m-d H:i:s'),
            'updated_at' => $this->updatedAt->format('Y-m-d H:i:s'),
        ];
    }
    
    public static function fromArray(array $data): self
    {
        $user = new self($data['username'], $data['email']);
        $user->setId($data['id']);
        $user->setActive($data['is_active'] ?? true);
        $user->setMetadata($data['metadata'] ?? []);
        return $user;
    }
}

class Product extends BaseModel
{
    private string $name;
    private float $price;
    private string $category;
    private ?string $description = null;
    private array $tags = [];
    
    public function __construct(string $name, float $price, string $category)
    {
        parent::__construct();
        $this->name = $name;
        $this->price = $price;
        $this->category = $category;
    }
    
    public function getName(): string
    {
        return $this->name;
    }
    
    public function setName(string $name): void
    {
        $this->name = $name;
        $this->updateTimestamp();
    }
    
    public function getPrice(): float
    {
        return $this->price;
    }
    
    public function setPrice(float $price): void
    {
        $this->price = $price;
        $this->updateTimestamp();
    }
    
    public function getCategory(): string
    {
        return $this->category;
    }
    
    public function setCategory(string $category): void
    {
        $this->category = $category;
        $this->updateTimestamp();
    }
    
    public function validate(): bool
    {
        $this->clearErrors();
        
        $isValid = true;
        $isValid &= $this->validateRequired('name', $this->name);
        
        if ($this->price <= 0) {
            $this->addError('price', 'Price must be greater than 0');
            $isValid = false;
        }
        
        return $isValid;
    }
    
    public function toArray(): array
    {
        return [
            'id' => $this->id,
            'name' => $this->name,
            'price' => $this->price,
            'category' => $this->category,
            'description' => $this->description,
            'tags' => $this->tags,
            'created_at' => $this->createdAt->format('Y-m-d H:i:s'),
            'updated_at' => $this->updatedAt->format('Y-m-d H:i:s'),
        ];
    }
}

// Service classes
class UserService
{
    private DatabaseConnectionInterface $db;
    private CacheInterface $cache;
    private LoggerInterface $logger;
    
    public function __construct(
        DatabaseConnectionInterface $db,
        CacheInterface $cache,
        LoggerInterface $logger
    ) {
        $this->db = $db;
        $this->cache = $cache;
        $this->logger = $logger;
    }
    
    public function createUser(string $username, string $email): User
    {
        $user = new User($username, $email);
        
        if (!$user->validate()) {
            throw new InvalidArgumentException('User validation failed: ' . implode(', ', $user->getErrors()));
        }
        
        try {
            $this->db->beginTransaction();
            
            $query = "INSERT INTO users (username, email, created_at, updated_at) VALUES (?, ?, ?, ?)";
            $params = [
                $user->getUsername(),
                $user->getEmail(),
                $user->getCreatedAt()->format('Y-m-d H:i:s'),
                $user->getUpdatedAt()->format('Y-m-d H:i:s')
            ];
            
            $this->db->executeQuery($query, $params);
            $this->db->commit();
            
            // Cache the user
            $this->cache->set("user:{$user->getId()}", $user->toArray());
            
            $this->logger->info("User created successfully", ['username' => $username]);
            
            return $user;
        } catch (Exception $e) {
            $this->db->rollback();
            $this->logger->error("Failed to create user", ['error' => $e->getMessage()]);
            throw $e;
        }
    }
    
    public function getUserById(int $id): ?User
    {
        // Check cache first
        $cached = $this->cache->get("user:{$id}");
        if ($cached) {
            return User::fromArray($cached);
        }
        
        try {
            $query = "SELECT * FROM users WHERE id = ?";
            $result = $this->db->executeQuery($query, [$id]);
            
            if (empty($result)) {
                return null;
            }
            
            $user = User::fromArray($result[0]);
            
            // Cache the result
            $this->cache->set("user:{$id}", $user->toArray());
            
            return $user;
        } catch (Exception $e) {
            $this->logger->error("Failed to get user", ['id' => $id, 'error' => $e->getMessage()]);
            return null;
        }
    }
    
    public function updateUser(int $id, array $data): bool
    {
        $user = $this->getUserById($id);
        if (!$user) {
            return false;
        }
        
        // Update user properties
        foreach ($data as $key => $value) {
            switch ($key) {
                case 'username':
                    $user->setUsername($value);
                    break;
                case 'email':
                    $user->setEmail($value);
                    break;
                case 'is_active':
                    $user->setActive($value);
                    break;
                case 'metadata':
                    $user->setMetadata($value);
                    break;
            }
        }
        
        if (!$user->validate()) {
            return false;
        }
        
        try {
            $query = "UPDATE users SET username = ?, email = ?, is_active = ?, metadata = ?, updated_at = ? WHERE id = ?";
            $params = [
                $user->getUsername(),
                $user->getEmail(),
                $user->isActive(),
                json_encode($user->getMetadata()),
                $user->getUpdatedAt()->format('Y-m-d H:i:s'),
                $id
            ];
            
            $this->db->executeQuery($query, $params);
            
            // Update cache
            $this->cache->set("user:{$id}", $user->toArray());
            
            return true;
        } catch (Exception $e) {
            $this->logger->error("Failed to update user", ['id' => $id, 'error' => $e->getMessage()]);
            return false;
        }
    }
}

// Configuration class
class AppConfig
{
    private bool $debugMode = false;
    private string $logLevel = 'INFO';
    private string $databaseUrl = 'postgresql://localhost:5432/myapp';
    private int $cacheSize = 1000;
    private int $maxConnections = MAX_CONNECTIONS;
    
    public function getDebugMode(): bool
    {
        return $this->debugMode;
    }
    
    public function setDebugMode(bool $debugMode): void
    {
        $this->debugMode = $debugMode;
    }
    
    public function getLogLevel(): string
    {
        return $this->logLevel;
    }
    
    public function setLogLevel(string $logLevel): void
    {
        $this->logLevel = $logLevel;
    }
    
    public function getDatabaseUrl(): string
    {
        return $this->databaseUrl;
    }
    
    public function setDatabaseUrl(string $databaseUrl): void
    {
        $this->databaseUrl = $databaseUrl;
    }
    
    public static function fromFile(string $configFile): self
    {
        $data = json_decode(file_get_contents($configFile), true);
        $config = new self();
        
        foreach ($data as $key => $value) {
            $method = 'set' . ucfirst($key);
            if (method_exists($config, $method)) {
                $config->$method($value);
            }
        }
        
        return $config;
    }
}

// Exception classes
class DatabaseException extends Exception {}
class ValidationException extends Exception {}
class ServiceException extends Exception {}

// Utility functions
function initializeDatabase(AppConfig $config): DatabaseConnectionInterface
{
    $url = $config->getDatabaseUrl();
    
    if (str_starts_with($url, 'postgresql')) {
        $db = new PostgresConnection('localhost', 5432, 'myapp', 'user', 'password');
    } else {
        $db = new MySQLConnection($url);
    }
    
    if (!$db->connect()) {
        throw new DatabaseException("Failed to connect to database");
    }
    
    return $db;
}

function createLogger(string $level = 'INFO'): LoggerInterface
{
    // Would return actual logger implementation
    return new class implements LoggerInterface {
        public function info(string $message, array $context = []): void {
            error_log("INFO: $message " . json_encode($context));
        }
        
        public function error(string $message, array $context = []): void {
            error_log("ERROR: $message " . json_encode($context));
        }
        
        public function debug(string $message, array $context = []): void {
            error_log("DEBUG: $message " . json_encode($context));
        }
    };
}

// Main application
function main(): void
{
    try {
        $config = new AppConfig();
        $config->setDebugMode(true);
        
        $db = initializeDatabase($config);
        $cache = new MemoryCache();
        $logger = createLogger($config->getLogLevel());
        
        $userService = new UserService($db, $cache, $logger);
        
        // Create sample user
        $user = $userService->createUser('john_doe', 'john@example.com');
        echo "Created user: " . json_encode($user->toArray()) . "\n";
        
        // Get user
        $retrievedUser = $userService->getUserById($user->getId());
        echo "Retrieved user: " . json_encode($retrievedUser->toArray()) . "\n";
        
    } catch (Exception $e) {
        error_log("Application error: " . $e->getMessage());
    }
}

if (php_sapi_name() === 'cli') {
    main();
}
?>
