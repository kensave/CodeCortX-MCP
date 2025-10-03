use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct MemoryManager {
    max_memory: u64,
    current_usage: Arc<AtomicU64>,
    warning_threshold: u64, // 80% of max
    last_check: std::sync::Mutex<Instant>,
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub current_usage: u64,
    pub max_memory: u64,
    pub usage_percentage: f64,
    pub is_under_pressure: bool,
}

impl MemoryManager {
    pub fn new(max_memory_mb: u64) -> Self {
        let max_memory = max_memory_mb * 1024 * 1024; // Convert MB to bytes
        let warning_threshold = (max_memory as f64 * 0.8) as u64;

        Self {
            max_memory,
            current_usage: Arc::new(AtomicU64::new(0)),
            warning_threshold,
            last_check: std::sync::Mutex::new(Instant::now()),
        }
    }

    pub fn default() -> Self {
        // Default to 1GB limit
        Self::new(1024)
    }

    pub fn from_env() -> Self {
        let max_mb = std::env::var("CODECORTEXT_MEMORY_LIMIT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1024); // Default 1GB

        Self::new(max_mb)
    }

    pub fn track_allocation(&self, size: u64) {
        self.current_usage.fetch_add(size, Ordering::Relaxed);
    }

    pub fn track_deallocation(&self, size: u64) {
        self.current_usage.fetch_sub(size, Ordering::Relaxed);
    }

    pub fn get_current_usage(&self) -> u64 {
        self.current_usage.load(Ordering::Relaxed)
    }

    pub fn get_usage_percentage(&self) -> f64 {
        let current = self.get_current_usage() as f64;
        let max = self.max_memory as f64;
        (current / max) * 100.0
    }

    pub fn is_under_pressure(&self) -> bool {
        self.get_current_usage() > self.warning_threshold
    }

    pub fn is_over_limit(&self) -> bool {
        self.get_current_usage() > self.max_memory
    }

    pub fn get_stats(&self) -> MemoryStats {
        let current = self.get_current_usage();
        MemoryStats {
            current_usage: current,
            max_memory: self.max_memory,
            usage_percentage: self.get_usage_percentage(),
            is_under_pressure: current > self.warning_threshold,
        }
    }

    pub fn should_trigger_cleanup(&self) -> bool {
        // Only check every 5 seconds to avoid excessive checking
        let mut last_check = self.last_check.lock().unwrap();
        let now = Instant::now();

        if now.duration_since(*last_check) < Duration::from_secs(5) {
            return false;
        }

        *last_check = now;
        self.is_under_pressure()
    }

    pub fn get_available_memory(&self) -> u64 {
        let current = self.get_current_usage();
        if current >= self.max_memory {
            0
        } else {
            self.max_memory - current
        }
    }

    pub fn estimate_symbol_size(symbol_name: &str, source_code: Option<&str>) -> u64 {
        let base_size = std::mem::size_of::<crate::models::Symbol>() as u64;
        let name_size = symbol_name.len() as u64;
        let source_size = source_code.map(|s| s.len() as u64).unwrap_or(0);

        base_size + name_size + source_size + 64 // Extra overhead
    }

    pub fn can_allocate(&self, size: u64) -> bool {
        self.get_current_usage() + size <= self.max_memory
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_tracking() {
        let manager = MemoryManager::new(100); // 100MB limit

        assert_eq!(manager.get_current_usage(), 0);
        assert!(!manager.is_under_pressure());

        // Track some allocations
        manager.track_allocation(50 * 1024 * 1024); // 50MB
        assert_eq!(manager.get_current_usage(), 50 * 1024 * 1024);
        assert!(!manager.is_under_pressure());

        // Go over warning threshold (80MB)
        manager.track_allocation(35 * 1024 * 1024); // 35MB more = 85MB total
        assert!(manager.is_under_pressure());
        assert!(!manager.is_over_limit());

        // Go over limit
        manager.track_allocation(20 * 1024 * 1024); // 20MB more = 105MB total
        assert!(manager.is_over_limit());
    }

    #[test]
    fn test_memory_deallocation() {
        let manager = MemoryManager::new(100);

        manager.track_allocation(90 * 1024 * 1024); // 90MB
        assert!(manager.is_under_pressure());

        manager.track_deallocation(50 * 1024 * 1024); // Remove 50MB
        assert_eq!(manager.get_current_usage(), 40 * 1024 * 1024);
        assert!(!manager.is_under_pressure());
    }

    #[test]
    fn test_usage_percentage() {
        let manager = MemoryManager::new(100); // 100MB

        manager.track_allocation(50 * 1024 * 1024); // 50MB
        assert_eq!(manager.get_usage_percentage(), 50.0);

        manager.track_allocation(25 * 1024 * 1024); // 25MB more = 75MB total
        assert_eq!(manager.get_usage_percentage(), 75.0);
    }

    #[test]
    fn test_can_allocate() {
        let manager = MemoryManager::new(100); // 100MB

        assert!(manager.can_allocate(50 * 1024 * 1024)); // 50MB - OK
        assert!(manager.can_allocate(100 * 1024 * 1024)); // 100MB - OK (exactly at limit)
        assert!(!manager.can_allocate(101 * 1024 * 1024)); // 101MB - Too much

        manager.track_allocation(80 * 1024 * 1024); // Use 80MB
        assert!(manager.can_allocate(20 * 1024 * 1024)); // 20MB more - OK
        assert!(!manager.can_allocate(21 * 1024 * 1024)); // 21MB more - Too much
    }

    #[test]
    fn test_symbol_size_estimation() {
        let size1 = MemoryManager::estimate_symbol_size("test_function", None);
        let size2 = MemoryManager::estimate_symbol_size("test_function", Some("fn test() {}"));

        assert!(size2 > size1); // With source should be larger
        assert!(size1 > 0); // Should have some base size
    }

    #[test]
    fn test_env_configuration() {
        std::env::set_var("CODECORTEXT_MEMORY_LIMIT", "512");
        let manager = MemoryManager::from_env();
        assert_eq!(manager.max_memory, 512 * 1024 * 1024);

        std::env::remove_var("CODECORTEXT_MEMORY_LIMIT");
        let manager2 = MemoryManager::from_env();
        assert_eq!(manager2.max_memory, 1024 * 1024 * 1024); // Default 1GB
    }
}
