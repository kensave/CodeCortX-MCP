package main

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"log"
	"sync"
	"time"
)

// Constants
const (
	MaxConnections = 100
	DefaultTimeout = 30 * time.Second
	APIVersion     = "1.0.0"
)

// Error types
var (
	ErrNotConnected     = errors.New("not connected to database")
	ErrUserNotFound     = errors.New("user not found")
	ErrInvalidEmail     = errors.New("invalid email format")
	ErrValidationFailed = errors.New("validation failed")
)

// Interfaces
type DatabaseConnection interface {
	Connect(ctx context.Context) error
	ExecuteQuery(ctx context.Context, query string, args ...interface{}) (*QueryResult, error)
	Close() error
	BeginTransaction(ctx context.Context) (Transaction, error)
}

type Transaction interface {
	Commit() error
	Rollback() error
	ExecuteQuery(ctx context.Context, query string, args ...interface{}) (*QueryResult, error)
}

type Cache interface {
	Get(key string) (interface{}, bool)
	Set(key string, value interface{}, ttl time.Duration)
	Delete(key string)
	Clear()
}

type Logger interface {
	Info(msg string, fields ...interface{})
	Error(msg string, fields ...interface{})
	Debug(msg string, fields ...interface{})
	Warn(msg string, fields ...interface{})
}

// Database implementations
type PostgresConnection struct {
	host       string
	port       int
	database   string
	username   string
	password   string
	connected  bool
	mu         sync.RWMutex
}

func NewPostgresConnection(host string, port int, database, username, password string) *PostgresConnection {
	return &PostgresConnection{
		host:     host,
		port:     port,
		database: database,
		username: username,
		password: password,
	}
}

func (p *PostgresConnection) Connect(ctx context.Context) error {
	p.mu.Lock()
	defer p.mu.Unlock()
	
	// Simulate connection logic
	select {
	case <-time.After(100 * time.Millisecond):
		p.connected = true
		return nil
	case <-ctx.Done():
		return ctx.Err()
	}
}

func (p *PostgresConnection) ExecuteQuery(ctx context.Context, query string, args ...interface{}) (*QueryResult, error) {
	p.mu.RLock()
	defer p.mu.RUnlock()
	
	if !p.connected {
		return nil, ErrNotConnected
	}
	
	// Simulate query execution
	select {
	case <-time.After(50 * time.Millisecond):
		return &QueryResult{Rows: []map[string]interface{}{}}, nil
	case <-ctx.Done():
		return nil, ctx.Err()
	}
}

func (p *PostgresConnection) Close() error {
	p.mu.Lock()
	defer p.mu.Unlock()
	
	p.connected = false
	return nil
}

func (p *PostgresConnection) BeginTransaction(ctx context.Context) (Transaction, error) {
	return &PostgresTransaction{conn: p}, nil
}

type PostgresTransaction struct {
	conn *PostgresConnection
}

func (t *PostgresTransaction) Commit() error {
	return nil
}

func (t *PostgresTransaction) Rollback() error {
	return nil
}

func (t *PostgresTransaction) ExecuteQuery(ctx context.Context, query string, args ...interface{}) (*QueryResult, error) {
	return t.conn.ExecuteQuery(ctx, query, args...)
}

type MySQLConnection struct {
	connectionString string
	connected        bool
}

func NewMySQLConnection(connectionString string) *MySQLConnection {
	return &MySQLConnection{
		connectionString: connectionString,
	}
}

func (m *MySQLConnection) Connect(ctx context.Context) error {
	m.connected = true
	return nil
}

func (m *MySQLConnection) ExecuteQuery(ctx context.Context, query string, args ...interface{}) (*QueryResult, error) {
	if !m.connected {
		return nil, ErrNotConnected
	}
	return &QueryResult{Rows: []map[string]interface{}{}}, nil
}

func (m *MySQLConnection) Close() error {
	m.connected = false
	return nil
}

func (m *MySQLConnection) BeginTransaction(ctx context.Context) (Transaction, error) {
	return &MySQLTransaction{conn: m}, nil
}

type MySQLTransaction struct {
	conn *MySQLConnection
}

func (t *MySQLTransaction) Commit() error {
	return nil
}

func (t *MySQLTransaction) Rollback() error {
	return nil
}

func (t *MySQLTransaction) ExecuteQuery(ctx context.Context, query string, args ...interface{}) (*QueryResult, error) {
	return t.conn.ExecuteQuery(ctx, query, args...)
}

// Cache implementation
type MemoryCache struct {
	data map[string]cacheItem
	mu   sync.RWMutex
}

type cacheItem struct {
	value  interface{}
	expiry time.Time
}

func NewMemoryCache() *MemoryCache {
	cache := &MemoryCache{
		data: make(map[string]cacheItem),
	}
	
	// Start cleanup goroutine
	go cache.cleanup()
	
	return cache
}

func (c *MemoryCache) Get(key string) (interface{}, bool) {
	c.mu.RLock()
	defer c.mu.RUnlock()
	
	item, exists := c.data[key]
	if !exists {
		return nil, false
	}
	
	if time.Now().After(item.expiry) {
		delete(c.data, key)
		return nil, false
	}
	
	return item.value, true
}

func (c *MemoryCache) Set(key string, value interface{}, ttl time.Duration) {
	c.mu.Lock()
	defer c.mu.Unlock()
	
	c.data[key] = cacheItem{
		value:  value,
		expiry: time.Now().Add(ttl),
	}
}

func (c *MemoryCache) Delete(key string) {
	c.mu.Lock()
	defer c.mu.Unlock()
	
	delete(c.data, key)
}

func (c *MemoryCache) Clear() {
	c.mu.Lock()
	defer c.mu.Unlock()
	
	c.data = make(map[string]cacheItem)
}

func (c *MemoryCache) cleanup() {
	ticker := time.NewTicker(1 * time.Minute)
	defer ticker.Stop()
	
	for range ticker.C {
		c.mu.Lock()
		now := time.Now()
		for key, item := range c.data {
			if now.After(item.expiry) {
				delete(c.data, key)
			}
		}
		c.mu.Unlock()
	}
}

// Data models
type User struct {
	ID        int64                  `json:"id"`
	Username  string                 `json:"username"`
	Email     string                 `json:"email"`
	CreatedAt time.Time              `json:"created_at"`
	UpdatedAt time.Time              `json:"updated_at"`
	IsActive  bool                   `json:"is_active"`
	Metadata  map[string]interface{} `json:"metadata"`
}

func NewUser(username, email string) *User {
	now := time.Now()
	return &User{
		ID:        time.Now().UnixNano(), // Simple ID generation
		Username:  username,
		Email:     email,
		CreatedAt: now,
		UpdatedAt: now,
		IsActive:  true,
		Metadata:  make(map[string]interface{}),
	}
}

func (u *User) ValidateEmail() bool {
	// Simple email validation
	return len(u.Email) > 0 && 
		   len(u.Email) < 255 && 
		   contains(u.Email, "@") && 
		   contains(u.Email, ".")
}

func (u *User) UpdateTimestamp() {
	u.UpdatedAt = time.Now()
}

func (u *User) ToJSON() ([]byte, error) {
	return json.Marshal(u)
}

func UserFromJSON(data []byte) (*User, error) {
	var user User
	err := json.Unmarshal(data, &user)
	return &user, err
}

type Product struct {
	ID          int64     `json:"id"`
	Name        string    `json:"name"`
	Price       float64   `json:"price"`
	Category    string    `json:"category"`
	Description string    `json:"description"`
	Tags        []string  `json:"tags"`
	CreatedAt   time.Time `json:"created_at"`
}

func NewProduct(name string, price float64, category string) *Product {
	return &Product{
		ID:        time.Now().UnixNano(),
		Name:      name,
		Price:     price,
		Category:  category,
		Tags:      make([]string, 0),
		CreatedAt: time.Now(),
	}
}

func (p *Product) AddTag(tag string) {
	for _, existingTag := range p.Tags {
		if existingTag == tag {
			return // Tag already exists
		}
	}
	p.Tags = append(p.Tags, tag)
}

// Service layer
type UserService struct {
	db     DatabaseConnection
	cache  Cache
	logger Logger
}

func NewUserService(db DatabaseConnection, cache Cache, logger Logger) *UserService {
	return &UserService{
		db:     db,
		cache:  cache,
		logger: logger,
	}
}

func (s *UserService) CreateUser(ctx context.Context, username, email string) (*User, error) {
	user := NewUser(username, email)
	
	if !user.ValidateEmail() {
		s.logger.Error("Invalid email format", "email", email)
		return nil, ErrInvalidEmail
	}
	
	tx, err := s.db.BeginTransaction(ctx)
	if err != nil {
		s.logger.Error("Failed to begin transaction", "error", err)
		return nil, err
	}
	
	defer func() {
		if err != nil {
			tx.Rollback()
		}
	}()
	
	query := "INSERT INTO users (username, email, created_at, updated_at) VALUES ($1, $2, $3, $4)"
	_, err = tx.ExecuteQuery(ctx, query, user.Username, user.Email, user.CreatedAt, user.UpdatedAt)
	if err != nil {
		s.logger.Error("Failed to insert user", "error", err, "username", username)
		return nil, err
	}
	
	err = tx.Commit()
	if err != nil {
		s.logger.Error("Failed to commit transaction", "error", err)
		return nil, err
	}
	
	// Cache the user
	cacheKey := fmt.Sprintf("user:%d", user.ID)
	s.cache.Set(cacheKey, user, 1*time.Hour)
	
	s.logger.Info("User created successfully", "user_id", user.ID, "username", username)
	return user, nil
}

func (s *UserService) GetUser(ctx context.Context, id int64) (*User, error) {
	// Check cache first
	cacheKey := fmt.Sprintf("user:%d", id)
	if cached, found := s.cache.Get(cacheKey); found {
		if user, ok := cached.(*User); ok {
			s.logger.Debug("User found in cache", "user_id", id)
			return user, nil
		}
	}
	
	// Query database
	query := "SELECT id, username, email, created_at, updated_at, is_active FROM users WHERE id = $1"
	result, err := s.db.ExecuteQuery(ctx, query, id)
	if err != nil {
		s.logger.Error("Failed to query user", "error", err, "user_id", id)
		return nil, err
	}
	
	if len(result.Rows) == 0 {
		return nil, ErrUserNotFound
	}
	
	row := result.Rows[0]
	user := &User{
		ID:        row["id"].(int64),
		Username:  row["username"].(string),
		Email:     row["email"].(string),
		CreatedAt: row["created_at"].(time.Time),
		UpdatedAt: row["updated_at"].(time.Time),
		IsActive:  row["is_active"].(bool),
		Metadata:  make(map[string]interface{}),
	}
	
	// Cache the result
	s.cache.Set(cacheKey, user, 1*time.Hour)
	
	s.logger.Debug("User retrieved from database", "user_id", id)
	return user, nil
}

func (s *UserService) UpdateUser(ctx context.Context, id int64, updates map[string]interface{}) error {
	user, err := s.GetUser(ctx, id)
	if err != nil {
		return err
	}
	
	// Apply updates
	if username, ok := updates["username"]; ok {
		user.Username = username.(string)
	}
	if email, ok := updates["email"]; ok {
		user.Email = email.(string)
	}
	if isActive, ok := updates["is_active"]; ok {
		user.IsActive = isActive.(bool)
	}
	
	user.UpdateTimestamp()
	
	// Update cache
	cacheKey := fmt.Sprintf("user:%d", id)
	s.cache.Set(cacheKey, user, 1*time.Hour)
	
	s.logger.Info("User updated successfully", "user_id", id)
	return nil
}

// Configuration
type AppConfig struct {
	DebugMode       bool          `json:"debug_mode"`
	LogLevel        string        `json:"log_level"`
	DatabaseURL     string        `json:"database_url"`
	CacheSize       int           `json:"cache_size"`
	MaxConnections  int           `json:"max_connections"`
	RequestTimeout  time.Duration `json:"request_timeout"`
}

func NewAppConfig() *AppConfig {
	return &AppConfig{
		DebugMode:      false,
		LogLevel:       "INFO",
		DatabaseURL:    "postgresql://localhost:5432/myapp",
		CacheSize:      1000,
		MaxConnections: MaxConnections,
		RequestTimeout: DefaultTimeout,
	}
}

func (c *AppConfig) LoadFromEnv() {
	// In a real implementation, this would load from environment variables
	c.DebugMode = true
	c.LogLevel = "DEBUG"
}

// Logger implementation
type ConsoleLogger struct {
	level string
}

func NewConsoleLogger(level string) *ConsoleLogger {
	return &ConsoleLogger{level: level}
}

func (l *ConsoleLogger) Info(msg string, fields ...interface{}) {
	l.log("INFO", msg, fields...)
}

func (l *ConsoleLogger) Error(msg string, fields ...interface{}) {
	l.log("ERROR", msg, fields...)
}

func (l *ConsoleLogger) Debug(msg string, fields ...interface{}) {
	if l.level == "DEBUG" {
		l.log("DEBUG", msg, fields...)
	}
}

func (l *ConsoleLogger) Warn(msg string, fields ...interface{}) {
	l.log("WARN", msg, fields...)
}

func (l *ConsoleLogger) log(level, msg string, fields ...interface{}) {
	timestamp := time.Now().Format(time.RFC3339)
	fmt.Printf("[%s] %s: %s", level, timestamp, msg)
	
	for i := 0; i < len(fields); i += 2 {
		if i+1 < len(fields) {
			fmt.Printf(" %v=%v", fields[i], fields[i+1])
		}
	}
	fmt.Println()
}

// Query result
type QueryResult struct {
	Rows     []map[string]interface{}
	RowCount int
}

// Utility functions
func InitializeDatabase(config *AppConfig) (DatabaseConnection, error) {
	if contains(config.DatabaseURL, "postgresql") {
		return NewPostgresConnection("localhost", 5432, "myapp", "user", "password"), nil
	} else if contains(config.DatabaseURL, "mysql") {
		return NewMySQLConnection(config.DatabaseURL), nil
	}
	
	return nil, errors.New("unsupported database type")
}

func contains(s, substr string) bool {
	return len(s) >= len(substr) && 
		   (s == substr || 
		    (len(s) > len(substr) && 
		     (s[:len(substr)] == substr || 
		      s[len(s)-len(substr):] == substr || 
		      containsSubstring(s, substr))))
}

func containsSubstring(s, substr string) bool {
	for i := 0; i <= len(s)-len(substr); i++ {
		if s[i:i+len(substr)] == substr {
			return true
		}
	}
	return false
}

// Main application
func main() {
	ctx := context.Background()
	
	config := NewAppConfig()
	config.LoadFromEnv()
	
	logger := NewConsoleLogger(config.LogLevel)
	
	db, err := InitializeDatabase(config)
	if err != nil {
		log.Fatal("Failed to initialize database:", err)
	}
	
	err = db.Connect(ctx)
	if err != nil {
		log.Fatal("Failed to connect to database:", err)
	}
	defer db.Close()
	
	cache := NewMemoryCache()
	userService := NewUserService(db, cache, logger)
	
	// Create sample user
	user, err := userService.CreateUser(ctx, "john_doe", "john@example.com")
	if err != nil {
		logger.Error("Failed to create user", "error", err)
		return
	}
	
	logger.Info("Created user successfully", "user", user.Username)
	
	// Get user
	retrievedUser, err := userService.GetUser(ctx, user.ID)
	if err != nil {
		logger.Error("Failed to get user", "error", err)
		return
	}
	
	logger.Info("Retrieved user successfully", "user", retrievedUser.Username)
	
	// Update user
	updates := map[string]interface{}{
		"is_active": false,
	}
	
	err = userService.UpdateUser(ctx, user.ID, updates)
	if err != nil {
		logger.Error("Failed to update user", "error", err)
		return
	}
	
	logger.Info("Application completed successfully")
}
