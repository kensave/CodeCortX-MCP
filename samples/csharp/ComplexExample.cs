using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using System.Linq;

namespace MyApp.Database
{
    public interface IDatabaseConnection
    {
        Task<bool> ConnectAsync();
        Task<QueryResult> ExecuteQueryAsync(string query, params object[] parameters);
        Task CloseAsync();
    }

    public class PostgresConnection : IDatabaseConnection
    {
        private readonly string host;
        private readonly int port;
        private readonly string database;
        private bool isConnected;

        public PostgresConnection(string host, int port, string database)
        {
            this.host = host;
            this.port = port;
            this.database = database;
        }

        public async Task<bool> ConnectAsync()
        {
            await Task.Delay(100);
            isConnected = true;
            return true;
        }

        public async Task<QueryResult> ExecuteQueryAsync(string query, params object[] parameters)
        {
            if (!isConnected)
                throw new InvalidOperationException("Not connected");

            await Task.Delay(50);
            return new QueryResult();
        }

        public async Task CloseAsync()
        {
            await Task.Delay(10);
            isConnected = false;
        }
    }
}

namespace MyApp.Models
{
    public class User
    {
        public long Id { get; set; }
        public string Username { get; set; }
        public string Email { get; set; }
        public DateTime CreatedAt { get; set; }
        public DateTime UpdatedAt { get; set; }
        public bool IsActive { get; set; }
        public Dictionary<string, object> Metadata { get; set; }

        public User(string username, string email)
        {
            Username = username;
            Email = email;
            CreatedAt = DateTime.UtcNow;
            UpdatedAt = DateTime.UtcNow;
            IsActive = true;
            Metadata = new Dictionary<string, object>();
        }

        public bool ValidateEmail()
        {
            return !string.IsNullOrEmpty(Email) && 
                   Email.Contains("@") && 
                   Email.Contains(".");
        }

        public void UpdateTimestamp()
        {
            UpdatedAt = DateTime.UtcNow;
        }
    }

    public class Product
    {
        public long Id { get; set; }
        public string Name { get; set; }
        public decimal Price { get; set; }
        public ProductCategory Category { get; set; }
        public string Description { get; set; }
        public List<string> Tags { get; set; }

        public Product(string name, decimal price, ProductCategory category)
        {
            Name = name;
            Price = price;
            Category = category;
            Tags = new List<string>();
        }
    }

    public enum ProductCategory
    {
        Electronics,
        Clothing,
        Books,
        Home,
        Sports
    }
}

namespace MyApp.Services
{
    public class UserService<T> where T : IDatabaseConnection
    {
        private readonly T database;
        private readonly ICache<long, User> cache;
        private readonly ILogger logger;

        public UserService(T database, ICache<long, User> cache, ILogger logger)
        {
            this.database = database;
            this.cache = cache;
            this.logger = logger;
        }

        public async Task<User> CreateUserAsync(string username, string email)
        {
            var user = new User(username, email);

            if (!user.ValidateEmail())
            {
                throw new ArgumentException("Invalid email format");
            }

            var query = "INSERT INTO users (username, email) VALUES (@username, @email)";
            await database.ExecuteQueryAsync(query, username, email);

            cache.Set(user.Id, user, TimeSpan.FromHours(1));
            logger.LogInfo($"User created: {username}");

            return user;
        }

        public async Task<User> GetUserAsync(long id)
        {
            if (cache.TryGet(id, out User cachedUser))
            {
                return cachedUser;
            }

            var query = "SELECT * FROM users WHERE id = @id";
            var result = await database.ExecuteQueryAsync(query, id);

            if (result.Rows.Count == 0)
            {
                return null;
            }

            var user = MapToUser(result.Rows.First());
            cache.Set(id, user, TimeSpan.FromHours(1));

            return user;
        }

        private User MapToUser(Dictionary<string, object> row)
        {
            return new User(row["username"].ToString(), row["email"].ToString())
            {
                Id = (long)row["id"],
                IsActive = (bool)row["is_active"]
            };
        }
    }

    public interface ICache<TKey, TValue>
    {
        bool TryGet(TKey key, out TValue value);
        void Set(TKey key, TValue value, TimeSpan expiry);
        void Remove(TKey key);
    }

    public interface ILogger
    {
        void LogInfo(string message);
        void LogError(string message, Exception exception = null);
    }
}

namespace MyApp
{
    public class QueryResult
    {
        public List<Dictionary<string, object>> Rows { get; set; }
        public int RowCount => Rows?.Count ?? 0;

        public QueryResult()
        {
            Rows = new List<Dictionary<string, object>>();
        }
    }

    public static class Constants
    {
        public const int MaxUsers = 1000;
        public const string ApiVersion = "1.0.0";
    }

    public class Program
    {
        public static async Task Main(string[] args)
        {
            try
            {
                await InitializeApp();
                Console.WriteLine("Application started successfully");
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Application error: {ex.Message}");
            }
        }

        private static async Task InitializeApp()
        {
            var connection = new PostgresConnection("localhost", 5432, "myapp");
            await connection.ConnectAsync();
            Console.WriteLine("Database connected");
        }
    }
}
