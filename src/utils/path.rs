use std::path::{Path, PathBuf};
use rmcp::model::{ErrorCode, ErrorData};

/// Comprehensive path resolution utility for all MCP tools
pub struct PathResolver;

impl PathResolver {
    /// Resolve any path format to a canonical PathBuf
    /// Handles: relative paths, ~/ expansion, ./ prefix, absolute paths, .. resolution
    pub fn resolve_path<P: AsRef<Path>>(path: P) -> Result<PathBuf, ErrorData> {
        let path_str = path.as_ref().to_string_lossy();
        
        // Handle tilde expansion
        let expanded_path = if path_str.starts_with("~/") {
            match dirs::home_dir() {
                Some(home) => home.join(&path_str[2..]),
                None => return Err(ErrorData::new(
                    ErrorCode::INTERNAL_ERROR,
                    "Cannot determine home directory",
                    None,
                )),
            }
        } else if path_str == "~" {
            match dirs::home_dir() {
                Some(home) => home,
                None => return Err(ErrorData::new(
                    ErrorCode::INTERNAL_ERROR,
                    "Cannot determine home directory",
                    None,
                )),
            }
        } else if path_str.starts_with("./") || path_str.starts_with("/") {
            // Already properly prefixed or absolute
            PathBuf::from(path_str.as_ref())
        } else if path_str == "." {
            // Current directory
            std::env::current_dir().map_err(|e| ErrorData::new(
                ErrorCode::INTERNAL_ERROR,
                format!("Cannot get current directory: {}", e),
                None,
            ))?
        } else {
            // Relative path without ./ prefix - add current directory
            std::env::current_dir()
                .map_err(|e| ErrorData::new(
                    ErrorCode::INTERNAL_ERROR,
                    format!("Cannot get current directory: {}", e),
                    None,
                ))?
                .join(path_str.as_ref())
        };

        // Canonicalize to resolve .. and . components and ensure path exists
        expanded_path.canonicalize().map_err(|e| ErrorData::new(
            ErrorCode::INVALID_PARAMS,
            format!("Invalid path '{}': {}", path_str, e),
            None,
        ))
    }

    /// Resolve path and ensure it's a file
    pub fn resolve_file_path<P: AsRef<Path>>(path: P) -> Result<PathBuf, ErrorData> {
        let resolved = Self::resolve_path(path)?;
        
        if !resolved.is_file() {
            return Err(ErrorData::new(
                ErrorCode::INVALID_PARAMS,
                format!("Path is not a file: {}", resolved.display()),
                None,
            ));
        }
        
        Ok(resolved)
    }

    /// Resolve path and ensure it's a directory
    pub fn resolve_directory_path<P: AsRef<Path>>(path: P) -> Result<PathBuf, ErrorData> {
        let resolved = Self::resolve_path(path)?;
        
        if !resolved.is_dir() {
            return Err(ErrorData::new(
                ErrorCode::INVALID_PARAMS,
                format!("Path is not a directory: {}", resolved.display()),
                None,
            ));
        }
        
        Ok(resolved)
    }

    /// Resolve path that can be either file or directory
    pub fn resolve_file_or_directory_path<P: AsRef<Path>>(path: P) -> Result<PathBuf, ErrorData> {
        let resolved = Self::resolve_path(path)?;
        
        if !resolved.exists() {
            return Err(ErrorData::new(
                ErrorCode::INVALID_PARAMS,
                format!("Path does not exist: {}", resolved.display()),
                None,
            ));
        }
        
        Ok(resolved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_resolve_current_directory() {
        let result = PathResolver::resolve_path(".");
        assert!(result.is_ok());
        assert!(result.unwrap().is_dir());
    }

    #[test]
    fn test_resolve_relative_path() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test").unwrap();
        
        // Change to temp directory
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        let result = PathResolver::resolve_path("test.txt");
        assert!(result.is_ok());
        
        let resolved_path = result.unwrap();
        // Check that the resolved path ends with the expected file name
        // (handles macOS /private prefix canonicalization)
        assert!(resolved_path.ends_with("test.txt"));
        assert!(resolved_path.exists());
        
        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_resolve_dotslash_path() {
        let result = PathResolver::resolve_path("./src");
        // Should work if src directory exists
        if std::path::Path::new("src").exists() {
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_file_validation() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test").unwrap();
        
        let result = PathResolver::resolve_file_path(&test_file);
        assert!(result.is_ok());
        
        // Test directory fails file validation
        let dir_result = PathResolver::resolve_file_path(temp_dir.path());
        assert!(dir_result.is_err());
    }

    #[test]
    fn test_directory_validation() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test").unwrap();
        
        let result = PathResolver::resolve_directory_path(temp_dir.path());
        assert!(result.is_ok());
        
        // Test file fails directory validation
        let file_result = PathResolver::resolve_directory_path(&test_file);
        assert!(file_result.is_err());
    }
}
