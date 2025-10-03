"""
Complex Python example with multiple constructs
"""
import asyncio
import json
from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Dict, List, Optional, Union, Generic, TypeVar
from datetime import datetime
import logging

# Type variables
T = TypeVar('T')
K = TypeVar('K')
V = TypeVar('V')

# Constants
MAX_CONNECTIONS = 100
DEFAULT_TIMEOUT = 30
API_VERSION = "v1.0.0"

# Database module
class DatabaseConnection(ABC):
    """Abstract base class for database connections"""
    
    @abstractmethod
    async def connect(self) -> bool:
        pass
    
    @abstractmethod
    async def execute_query(self, query: str) -> Dict:
        pass
    
    @abstractmethod
    async def close(self) -> None:
        pass

class PostgresConnection(DatabaseConnection):
    """PostgreSQL database connection implementation"""
    
    def __init__(self, host: str, port: int, database: str, username: str, password: str):
        self.host = host
        self.port = port
        self.database = database
        self.username = username
        self.password = password
        self.connection = None
        self._is_connected = False
    
    async def connect(self) -> bool:
        """Connect to PostgreSQL database"""
        try:
            # Simulate connection logic
            self._is_connected = True
            return True
        except Exception as e:
            logging.error(f"Failed to connect to PostgreSQL: {e}")
            return False
    
    async def execute_query(self, query: str) -> Dict:
        """Execute SQL query"""
        if not self._is_connected:
            raise ConnectionError("Not connected to database")
        
        # Simulate query execution
        return {"rows": [], "affected": 0}
    
    async def close(self) -> None:
        """Close database connection"""
        self._is_connected = False

class MySQLConnection(DatabaseConnection):
    """MySQL database connection implementation"""
    
    def __init__(self, connection_string: str):
        self.connection_string = connection_string
        self.connection = None
    
    async def connect(self) -> bool:
        return True
    
    async def execute_query(self, query: str) -> Dict:
        return {"rows": [], "affected": 0}
    
    async def close(self) -> None:
        pass

# Data models
@dataclass
class User:
    """User data model"""
    id: int
    username: str
    email: str
    created_at: datetime
    is_active: bool = True
    metadata: Optional[Dict] = None
    
    def __post_init__(self):
        if self.metadata is None:
            self.metadata = {}
    
    def validate_email(self) -> bool:
        """Validate email format"""
        return '@' in self.email and '.' in self.email
    
    def to_dict(self) -> Dict:
        """Convert user to dictionary"""
        return {
            'id': self.id,
            'username': self.username,
            'email': self.email,
            'created_at': self.created_at.isoformat(),
            'is_active': self.is_active,
            'metadata': self.metadata
        }
    
    @classmethod
    def from_dict(cls, data: Dict) -> 'User':
        """Create user from dictionary"""
        return cls(
            id=data['id'],
            username=data['username'],
            email=data['email'],
            created_at=datetime.fromisoformat(data['created_at']),
            is_active=data.get('is_active', True),
            metadata=data.get('metadata')
        )

@dataclass
class Product:
    """Product data model"""
    id: int
    name: str
    price: float
    category: str
    description: Optional[str] = None
    tags: List[str] = None
    
    def __post_init__(self):
        if self.tags is None:
            self.tags = []

class ProductCategory:
    """Product category enumeration"""
    ELECTRONICS = "electronics"
    CLOTHING = "clothing"
    BOOKS = "books"
    HOME = "home"
    SPORTS = "sports"

# Generic cache class
class Cache(Generic[K, V]):
    """Generic cache implementation"""
    
    def __init__(self, max_size: int = 1000):
        self.max_size = max_size
        self._data: Dict[K, V] = {}
        self._access_order: List[K] = []
    
    def get(self, key: K) -> Optional[V]:
        """Get value from cache"""
        if key in self._data:
            self._access_order.remove(key)
            self._access_order.append(key)
            return self._data[key]
        return None
    
    def put(self, key: K, value: V) -> None:
        """Put value in cache"""
        if len(self._data) >= self.max_size and key not in self._data:
            # Remove least recently used
            oldest_key = self._access_order.pop(0)
            del self._data[oldest_key]
        
        self._data[key] = value
        if key in self._access_order:
            self._access_order.remove(key)
        self._access_order.append(key)
    
    def clear(self) -> None:
        """Clear cache"""
        self._data.clear()
        self._access_order.clear()

# Service classes with decorators
def retry(max_attempts: int = 3):
    """Retry decorator"""
    def decorator(func):
        async def wrapper(*args, **kwargs):
            for attempt in range(max_attempts):
                try:
                    return await func(*args, **kwargs)
                except Exception as e:
                    if attempt == max_attempts - 1:
                        raise e
                    await asyncio.sleep(0.1 * (2 ** attempt))
            return None
        return wrapper
    return decorator

def log_execution(func):
    """Logging decorator"""
    async def wrapper(*args, **kwargs):
        logging.info(f"Executing {func.__name__}")
        try:
            result = await func(*args, **kwargs)
            logging.info(f"Successfully executed {func.__name__}")
            return result
        except Exception as e:
            logging.error(f"Error in {func.__name__}: {e}")
            raise
    return wrapper

class UserService:
    """User service with database operations"""
    
    def __init__(self, db: DatabaseConnection):
        self.db = db
        self.cache: Cache[int, User] = Cache(max_size=500)
        self.logger = logging.getLogger(__name__)
    
    @retry(max_attempts=3)
    @log_execution
    async def create_user(self, username: str, email: str) -> User:
        """Create new user"""
        user = User(
            id=0,  # Would be generated by database
            username=username,
            email=email,
            created_at=datetime.now()
        )
        
        if not user.validate_email():
            raise ValueError("Invalid email format")
        
        # Save to database
        query = f"INSERT INTO users (username, email) VALUES ('{username}', '{email}')"
        result = await self.db.execute_query(query)
        
        # Cache the user
        self.cache.put(user.id, user)
        
        return user
    
    @log_execution
    async def get_user(self, user_id: int) -> Optional[User]:
        """Get user by ID"""
        # Check cache first
        cached_user = self.cache.get(user_id)
        if cached_user:
            return cached_user
        
        # Query database
        query = f"SELECT * FROM users WHERE id = {user_id}"
        result = await self.db.execute_query(query)
        
        if result['rows']:
            user_data = result['rows'][0]
            user = User.from_dict(user_data)
            self.cache.put(user_id, user)
            return user
        
        return None
    
    async def update_user(self, user_id: int, **kwargs) -> bool:
        """Update user information"""
        user = await self.get_user(user_id)
        if not user:
            return False
        
        # Update user attributes
        for key, value in kwargs.items():
            if hasattr(user, key):
                setattr(user, key, value)
        
        # Save to database
        # ... database update logic
        
        # Update cache
        self.cache.put(user_id, user)
        return True

class ProductService:
    """Product service"""
    
    def __init__(self, db: DatabaseConnection):
        self.db = db
        self.cache: Cache[int, Product] = Cache()
    
    async def create_product(self, name: str, price: float, category: str) -> Product:
        """Create new product"""
        product = Product(
            id=0,  # Generated by database
            name=name,
            price=price,
            category=category
        )
        
        # Save to database
        return product
    
    async def search_products(self, query: str, category: Optional[str] = None) -> List[Product]:
        """Search products"""
        # Implementation would query database
        return []

# Application configuration
class AppConfig:
    """Application configuration"""
    
    def __init__(self):
        self.debug_mode = False
        self.log_level = "INFO"
        self.database_url = "postgresql://localhost:5432/myapp"
        self.cache_size = 1000
        self.max_connections = MAX_CONNECTIONS
    
    @classmethod
    def from_file(cls, config_file: str) -> 'AppConfig':
        """Load configuration from file"""
        with open(config_file, 'r') as f:
            data = json.load(f)
        
        config = cls()
        for key, value in data.items():
            if hasattr(config, key):
                setattr(config, key, value)
        
        return config

# Exception classes
class DatabaseError(Exception):
    """Database operation error"""
    pass

class ValidationError(Exception):
    """Data validation error"""
    pass

class ServiceError(Exception):
    """Service operation error"""
    pass

# Utility functions
def setup_logging(level: str = "INFO") -> None:
    """Setup application logging"""
    logging.basicConfig(
        level=getattr(logging, level),
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )

async def initialize_database(config: AppConfig) -> DatabaseConnection:
    """Initialize database connection"""
    if config.database_url.startswith('postgresql'):
        db = PostgresConnection(
            host='localhost',
            port=5432,
            database='myapp',
            username='user',
            password='password'
        )
    else:
        db = MySQLConnection(config.database_url)
    
    await db.connect()
    return db

async def main():
    """Main application entry point"""
    # Setup
    setup_logging("DEBUG")
    config = AppConfig()
    
    # Initialize services
    db = await initialize_database(config)
    user_service = UserService(db)
    product_service = ProductService(db)
    
    try:
        # Create sample user
        user = await user_service.create_user("john_doe", "john@example.com")
        print(f"Created user: {user}")
        
        # Get user
        retrieved_user = await user_service.get_user(user.id)
        print(f"Retrieved user: {retrieved_user}")
        
    except Exception as e:
        logging.error(f"Application error: {e}")
    finally:
        await db.close()

if __name__ == "__main__":
    asyncio.run(main())
