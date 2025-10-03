use crate::models::Language;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use tokio::fs;

pub struct FileSystemWalker;

impl FileSystemWalker {
    pub fn new() -> Self {
        Self
    }

    /// Find all source files in a directory recursively, respecting .gitignore
    pub fn find_source_files<P: AsRef<Path>>(
        path: P,
    ) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let mut source_files = Vec::new();

        // Use ignore crate to respect .gitignore files
        let walker = WalkBuilder::new(&path)
            .follow_links(false)
            .git_ignore(true)
            .git_exclude(true)
            .git_global(true)
            .hidden(false) // Include hidden files but respect .gitignore
            .build();

        for entry in walker.filter_map(|e| e.ok()) {
            let path = entry.path();

            // Skip directories
            if !path.is_file() {
                continue;
            }

            // Check if it's a supported source file
            if Language::is_source_file(path) {
                source_files.push(path.to_path_buf());
            }
        }

        Ok(source_files)
    }

    /// Check if path exists and is accessible
    pub async fn is_accessible<P: AsRef<Path>>(path: P) -> bool {
        match fs::metadata(path).await {
            Ok(metadata) => metadata.is_file() || metadata.is_dir(),
            Err(_) => false,
        }
    }

    /// Get file size
    pub async fn get_file_size<P: AsRef<Path>>(path: P) -> Result<u64, std::io::Error> {
        let metadata = fs::metadata(path).await?;
        Ok(metadata.len())
    }

    /// Read file content as string
    pub async fn read_file_content<P: AsRef<Path>>(path: P) -> Result<String, std::io::Error> {
        fs::read_to_string(path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_source_file_discovery() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create test files
        let rust_file = base_path.join("test.rs");
        let python_file = base_path.join("script.py");
        let text_file = base_path.join("readme.txt");

        fs::write(&rust_file, "fn main() {}").await.unwrap();
        fs::write(&python_file, "print('hello')").await.unwrap();
        fs::write(&text_file, "Not source code").await.unwrap();

        // Find source files
        let source_files = FileSystemWalker::find_source_files(base_path).unwrap();

        // Should find only Rust and Python files
        assert_eq!(source_files.len(), 2);
        assert!(source_files.contains(&rust_file));
        assert!(source_files.contains(&python_file));
        assert!(!source_files.contains(&text_file));
    }

    #[tokio::test]
    async fn test_nested_directory_traversal() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create nested structure
        let src_dir = base_path.join("src");
        fs::create_dir(&src_dir).await.unwrap();

        let nested_file = src_dir.join("lib.rs");
        fs::write(&nested_file, "pub fn test() {}").await.unwrap();

        let root_file = base_path.join("main.py");
        fs::write(&root_file, "def main(): pass").await.unwrap();

        // Find all source files
        let source_files = FileSystemWalker::find_source_files(base_path).unwrap();

        assert_eq!(source_files.len(), 2);
        assert!(source_files.contains(&nested_file));
        assert!(source_files.contains(&root_file));
    }

    #[tokio::test]
    async fn test_file_accessibility() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        // File doesn't exist yet
        assert!(!FileSystemWalker::is_accessible(&test_file).await);

        // Create file
        fs::write(&test_file, "fn test() {}").await.unwrap();

        // Now it should be accessible
        assert!(FileSystemWalker::is_accessible(&test_file).await);
        assert!(FileSystemWalker::is_accessible(temp_dir.path()).await);
    }

    #[tokio::test]
    async fn test_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        let content = "fn hello() { println!(\"Hello, world!\"); }";

        fs::write(&test_file, content).await.unwrap();

        // Test file size
        let size = FileSystemWalker::get_file_size(&test_file).await.unwrap();
        assert_eq!(size, content.len() as u64);

        // Test content reading
        let read_content = FileSystemWalker::read_file_content(&test_file)
            .await
            .unwrap();
        assert_eq!(read_content, content);
    }
}
