# Complex Ruby example with multiple constructs

# Constants
MAX_CONNECTIONS = 100
DEFAULT_TIMEOUT = 30
API_VERSION = '1.0.0'

# Database module
module Database
  # Abstract connection class
  class Connection
    def connect
      raise NotImplementedError, 'Subclass must implement connect'
    end
    
    def execute_query(query, params = [])
      raise NotImplementedError, 'Subclass must implement execute_query'
    end
    
    def close
      raise NotImplementedError, 'Subclass must implement close'
    end
    
    def begin_transaction
      raise NotImplementedError, 'Subclass must implement begin_transaction'
    end
    
    def commit
      raise NotImplementedError, 'Subclass must implement commit'
    end
    
    def rollback
      raise NotImplementedError, 'Subclass must implement rollback'
    end
  end
  
  # PostgreSQL implementation
  class PostgresConnection < Connection
    attr_reader :host, :port, :database, :username
    attr_accessor :connected, :transaction_active
    
    def initialize(host, port, database, username, password)
      @host = host
      @port = port
      @database = database
      @username = username
      @password = password
      @connected = false
      @transaction_active = false
    end
    
    def connect
      sleep(0.1) # Simulate connection time
      @connected = true
      puts "Connected to PostgreSQL at #{@host}:#{@port}"
      true
    rescue => e
      puts "Failed to connect: #{e.message}"
      false
    end
    
    def execute_query(query, params = [])
      raise 'Not connected' unless @connected
      
      # Simulate query execution
      sleep(0.05)
      QueryResult.new([])
    end
    
    def close
      @connected = false
      @transaction_active = false
    end
    
    def begin_transaction
      @transaction_active = true
    end
    
    def commit
      @transaction_active = false
    end
    
    def rollback
      @transaction_active = false
    end
  end
  
  # MySQL implementation
  class MySQLConnection < Connection
    attr_reader :connection_string
    
    def initialize(connection_string)
      @connection_string = connection_string
      @connected = false
    end
    
    def connect
      @connected = true
    end
    
    def execute_query(query, params = [])
      raise 'Not connected' unless @connected
      QueryResult.new([])
    end
    
    def close
      @connected = false
    end
    
    def begin_transaction; end
    def commit; end
    def rollback; end
  end
end

# Cache implementation
class MemoryCache
  def initialize(max_size = 1000)
    @data = {}
    @max_size = max_size
    @access_order = []
  end
  
  def get(key)
    return nil unless @data.key?(key)
    
    item = @data[key]
    return nil if item[:expiry] && item[:expiry] < Time.now
    
    # Update LRU order
    @access_order.delete(key)
    @access_order << key
    
    item[:value]
  end
  
  def set(key, value, ttl = nil)
    # Remove oldest if at capacity
    if @data.size >= @max_size && !@data.key?(key)
      oldest_key = @access_order.shift
      @data.delete(oldest_key)
    end
    
    expiry = ttl ? Time.now + ttl : nil
    @data[key] = { value: value, expiry: expiry }
    
    @access_order.delete(key)
    @access_order << key
  end
  
  def delete(key)
    @access_order.delete(key)
    @data.delete(key)
  end
  
  def clear
    @data.clear
    @access_order.clear
  end
end

# Data models
class User
  attr_accessor :id, :username, :email, :created_at, :updated_at, :is_active, :metadata
  
  def initialize(username, email)
    @id = Time.now.to_i
    @username = username
    @email = email
    @created_at = Time.now
    @updated_at = Time.now
    @is_active = true
    @metadata = {}
  end
  
  def validate_email
    @email.include?('@') && @email.include?('.')
  end
  
  def update_timestamp
    @updated_at = Time.now
  end
  
  def to_hash
    {
      id: @id,
      username: @username,
      email: @email,
      created_at: @created_at,
      updated_at: @updated_at,
      is_active: @is_active,
      metadata: @metadata
    }
  end
  
  def self.from_hash(data)
    user = new(data[:username], data[:email])
    user.id = data[:id]
    user.is_active = data[:is_active]
    user.metadata = data[:metadata] || {}
    user
  end
end

class Product
  attr_accessor :id, :name, :price, :category, :description, :tags, :created_at
  
  def initialize(name, price, category)
    @id = Time.now.to_i
    @name = name
    @price = price
    @category = category
    @description = nil
    @tags = []
    @created_at = Time.now
  end
  
  def add_tag(tag)
    @tags << tag unless @tags.include?(tag)
  end
  
  def remove_tag(tag)
    @tags.delete(tag)
  end
  
  def calculate_discounted_price(discount_percent)
    @price * (1 - discount_percent / 100.0)
  end
end

# Service classes with mixins
module Loggable
  def log_info(message, context = {})
    puts "[INFO] #{message} #{context}"
  end
  
  def log_error(message, context = {})
    puts "[ERROR] #{message} #{context}"
  end
  
  def log_debug(message, context = {})
    puts "[DEBUG] #{message} #{context}" if debug_mode?
  end
  
  private
  
  def debug_mode?
    ENV['DEBUG'] == 'true'
  end
end

module Retryable
  def with_retry(max_attempts = 3)
    attempts = 0
    begin
      attempts += 1
      yield
    rescue => e
      if attempts < max_attempts
        sleep(0.1 * (2 ** (attempts - 1)))
        retry
      else
        raise e
      end
    end
  end
end

class UserService
  include Loggable
  include Retryable
  
  def initialize(database, cache, logger = nil)
    @database = database
    @cache = cache
    @logger = logger
  end
  
  def create_user(username, email)
    with_retry do
      user = User.new(username, email)
      
      unless user.validate_email
        return { success: false, error: 'Invalid email format' }
      end
      
      @database.begin_transaction
      
      query = 'INSERT INTO users (username, email, created_at) VALUES (?, ?, ?)'
      @database.execute_query(query, [username, email, user.created_at])
      
      @database.commit
      
      # Cache the user
      @cache.set(user.id, user)
      
      log_info('User created successfully', { user_id: user.id, username: username })
      
      { success: true, data: user }
    end
  rescue => e
    @database.rollback
    log_error('Failed to create user', { error: e.message, username: username, email: email })
    
    { success: false, error: e.message }
  end
  
  def get_user(id)
    # Check cache first
    cached = @cache.get(id)
    return { success: true, data: cached } if cached
    
    # Query database
    query = 'SELECT * FROM users WHERE id = ?'
    result = @database.execute_query(query, [id])
    
    if result.rows.empty?
      return { success: false, error: 'User not found' }
    end
    
    user = User.from_hash(result.rows.first)
    @cache.set(id, user)
    
    { success: true, data: user }
  rescue => e
    log_error('Failed to get user', { error: e.message, user_id: id })
    { success: false, error: e.message }
  end
  
  def update_user(id, updates)
    user_result = get_user(id)
    return user_result unless user_result[:success]
    
    user = user_result[:data]
    
    # Apply updates
    updates.each do |key, value|
      case key.to_sym
      when :username
        user.username = value
      when :email
        user.email = value
      when :is_active
        user.is_active = value
      when :metadata
        user.metadata = value
      end
    end
    
    user.update_timestamp
    
    # Update cache
    @cache.set(id, user)
    
    { success: true, data: user }
  end
end

class ProductService
  include Loggable
  
  def initialize(database, cache)
    @database = database
    @cache = cache
  end
  
  def create_product(name, price, category)
    product = Product.new(name, price, category)
    
    query = 'INSERT INTO products (name, price, category) VALUES (?, ?, ?)'
    @database.execute_query(query, [name, price, category])
    
    @cache.set(product.id, product)
    log_info('Product created', { product_id: product.id, name: name })
    
    product
  end
  
  def search_products(search_query, category_filter = nil)
    query = 'SELECT * FROM products WHERE name LIKE ?'
    params = ["%#{search_query}%"]
    
    if category_filter
      query += ' AND category = ?'
      params << category_filter
    end
    
    result = @database.execute_query(query, params)
    result.rows.map { |row| Product.from_hash(row) }
  end
end

# Configuration management
class AppConfig
  attr_accessor :debug_mode, :log_level, :database_config, :cache_size, :max_connections
  
  def initialize
    @debug_mode = false
    @log_level = 'INFO'
    @database_config = {
      host: 'localhost',
      port: 5432,
      database: 'myapp',
      username: 'user',
      password: 'password'
    }
    @cache_size = 1000
    @max_connections = MAX_CONNECTIONS
  end
  
  def self.from_environment
    config = new
    
    config.debug_mode = ENV['DEBUG'] == 'true'
    config.log_level = ENV['LOG_LEVEL'] || 'INFO'
    config.cache_size = (ENV['CACHE_SIZE'] || '1000').to_i
    
    config.database_config[:host] = ENV['DB_HOST'] if ENV['DB_HOST']
    config.database_config[:port] = ENV['DB_PORT'].to_i if ENV['DB_PORT']
    config.database_config[:database] = ENV['DB_NAME'] if ENV['DB_NAME']
    config.database_config[:username] = ENV['DB_USER'] if ENV['DB_USER']
    config.database_config[:password] = ENV['DB_PASSWORD'] if ENV['DB_PASSWORD']
    
    config
  end
end

# Logger implementation
class ConsoleLogger
  def initialize(level = 'INFO')
    @level = level
  end
  
  def info(message, meta = {})
    log('INFO', message, meta) if should_log?('INFO')
  end
  
  def error(message, meta = {})
    log('ERROR', message, meta) if should_log?('ERROR')
  end
  
  def debug(message, meta = {})
    log('DEBUG', message, meta) if should_log?('DEBUG')
  end
  
  def warn(message, meta = {})
    log('WARN', message, meta) if should_log?('WARN')
  end
  
  private
  
  def log(level, message, meta)
    timestamp = Time.now.strftime('%Y-%m-%d %H:%M:%S')
    puts "[#{level}] #{timestamp}: #{message} #{meta}"
  end
  
  def should_log?(level)
    levels = %w[DEBUG INFO WARN ERROR]
    levels.index(level) >= levels.index(@level)
  end
end

# Query result class
class QueryResult
  attr_reader :rows, :row_count
  
  def initialize(rows = [])
    @rows = rows
    @row_count = rows.length
  end
end

# Exception classes
class DatabaseError < StandardError
  attr_reader :original_error
  
  def initialize(message, original_error = nil)
    super(message)
    @original_error = original_error
  end
end

class ValidationError < StandardError
  attr_reader :field
  
  def initialize(message, field = nil)
    super(message)
    @field = field
  end
end

class ServiceError < StandardError
  attr_reader :code
  
  def initialize(message, code = nil)
    super(message)
    @code = code
  end
end

# Utility functions
def initialize_database(config)
  db_config = config.database_config
  
  if db_config[:host].include?('postgres')
    Database::PostgresConnection.new(
      db_config[:host],
      db_config[:port],
      db_config[:database],
      db_config[:username],
      db_config[:password]
    )
  else
    connection_string = "mysql://#{db_config[:username]}:#{db_config[:password]}@#{db_config[:host]}:#{db_config[:port]}/#{db_config[:database]}"
    Database::MySQLConnection.new(connection_string)
  end
end

def create_logger(level = 'INFO')
  ConsoleLogger.new(level)
end

# Main application
def main
  config = AppConfig.from_environment
  logger = create_logger(config.log_level)
  database = initialize_database(config)
  cache = MemoryCache.new(config.cache_size)
  
  database.connect
  
  user_service = UserService.new(database, cache, logger)
  product_service = ProductService.new(database, cache)
  
  # Create sample user
  create_result = user_service.create_user('john_doe', 'john@example.com')
  if create_result[:success]
    logger.info('Created user successfully', create_result[:data].to_hash)
    
    # Get user
    get_result = user_service.get_user(create_result[:data].id)
    if get_result[:success]
      logger.info('Retrieved user successfully', get_result[:data].to_hash)
    end
  end
  
  # Create sample product
  product = product_service.create_product('Laptop', 999.99, 'Electronics')
  logger.info('Created product successfully', { product_id: product.id })
  
rescue => e
  puts "Application error: #{e.message}"
  puts e.backtrace
end

# Run if this is the main file
main if __FILE__ == $0
