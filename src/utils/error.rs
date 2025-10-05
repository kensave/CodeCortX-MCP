use thiserror::Error;

#[derive(Error, Debug)]
pub enum CodeAnalysisError {
    #[error("Parse error in file {file}: {message}")]
    ParseError { file: String, message: String },

    #[error("File system error: {0}")]
    FileSystemError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] bincode::error::EncodeError),

    #[error("Tree-sitter query error: {0}")]
    QueryError(#[from] tree_sitter::QueryError),

    #[error("Symbol not found: {name}")]
    SymbolNotFound { name: String },

    #[error("Memory limit exceeded: {current_mb}MB > {limit_mb}MB")]
    MemoryLimitExceeded { current_mb: u64, limit_mb: u64 },

    #[error("Unsupported file type: {extension}")]
    UnsupportedFileType { extension: String },

    #[error("Cache error: {message}")]
    CacheError { message: String },

    #[error("Indexing error: {message}")]
    IndexingError { message: String },

    #[error("Invalid path: {path}")]
    InvalidPath { path: String },

    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },

    #[error("File too large: {size_mb}MB exceeds limit")]
    FileTooLarge { size_mb: u64 },
}

impl CodeAnalysisError {
    pub fn is_recoverable(&self) -> bool {
        match self {
            // Recoverable errors - continue processing other files
            CodeAnalysisError::ParseError { .. } => true,
            CodeAnalysisError::UnsupportedFileType { .. } => true,
            CodeAnalysisError::FileTooLarge { .. } => true,
            CodeAnalysisError::PermissionDenied { .. } => true,

            // Non-recoverable errors - should stop processing
            CodeAnalysisError::MemoryLimitExceeded { .. } => false,
            CodeAnalysisError::FileSystemError(_) => false,
            CodeAnalysisError::SerializationError(_) => false,

            // Context-dependent
            _ => true,
        }
    }

    pub fn should_retry(&self) -> bool {
        match self {
            CodeAnalysisError::FileSystemError(io_err) => {
                matches!(
                    io_err.kind(),
                    std::io::ErrorKind::Interrupted
                        | std::io::ErrorKind::TimedOut
                        | std::io::ErrorKind::WouldBlock
                )
            }
            _ => false,
        }
    }
}

// Convert to string for MCP error handling
impl From<CodeAnalysisError> for String {
    fn from(error: CodeAnalysisError) -> Self {
        error.to_string()
    }
}

pub struct ErrorRecovery;

impl ErrorRecovery {
    pub fn handle_parse_error(file_path: &str, error: &str) -> CodeAnalysisError {
        tracing::warn!("Parse error in {}: {}", file_path, error);
        CodeAnalysisError::ParseError {
            file: file_path.to_string(),
            message: error.to_string(),
        }
    }

    pub fn handle_file_error(file_path: &str, error: std::io::Error) -> CodeAnalysisError {
        match error.kind() {
            std::io::ErrorKind::PermissionDenied => {
                tracing::warn!("Permission denied: {}", file_path);
                CodeAnalysisError::PermissionDenied {
                    path: file_path.to_string(),
                }
            }
            std::io::ErrorKind::NotFound => {
                tracing::debug!("File not found: {}", file_path);
                CodeAnalysisError::FileSystemError(error)
            }
            _ => {
                tracing::error!("File system error in {}: {}", file_path, error);
                CodeAnalysisError::FileSystemError(error)
            }
        }
    }

    pub fn should_continue_indexing(error: &CodeAnalysisError) -> bool {
        error.is_recoverable()
    }

    pub fn log_error_and_continue(error: &CodeAnalysisError, context: &str) {
        if error.is_recoverable() {
            tracing::warn!("Recoverable error in {}: {}", context, error);
        } else {
            tracing::error!("Critical error in {}: {}", context, error);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_recoverability() {
        let parse_error = CodeAnalysisError::ParseError {
            file: "test.rs".to_string(),
            message: "syntax error".to_string(),
        };
        assert!(parse_error.is_recoverable());

        let memory_error = CodeAnalysisError::MemoryLimitExceeded {
            current_mb: 1024,
            limit_mb: 512,
        };
        assert!(!memory_error.is_recoverable());
    }

    #[test]
    fn test_error_string_conversion() {
        let symbol_error = CodeAnalysisError::SymbolNotFound {
            name: "test_symbol".to_string(),
        };

        let error_string: String = symbol_error.into();
        // Should convert to error with appropriate message
        assert!(error_string.contains("Symbol not found"));
    }

    #[test]
    fn test_retry_logic() {
        let interrupted_error = CodeAnalysisError::FileSystemError(std::io::Error::from(
            std::io::ErrorKind::Interrupted,
        ));
        assert!(interrupted_error.should_retry());

        let parse_error = CodeAnalysisError::ParseError {
            file: "test.rs".to_string(),
            message: "syntax error".to_string(),
        };
        assert!(!parse_error.should_retry());
    }
}
