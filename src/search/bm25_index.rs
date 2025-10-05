use bm25::{Document, Language, SearchEngine, SearchEngineBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    RwLock,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeDocument {
    pub id: usize,
    pub file_path: PathBuf,
    pub content: String,
    pub language: String,
}

#[derive(Debug)]
pub struct BM25CodeIndex {
    engine: RwLock<SearchEngine<usize>>,
    documents: RwLock<HashMap<usize, CodeDocument>>,
    next_id: AtomicUsize,
    doc_count: AtomicUsize,
}

impl BM25CodeIndex {
    pub fn new() -> Self {
        let engine = SearchEngineBuilder::<usize>::with_avgdl(100.0) // Average code file ~100 lines
            .language_mode(Language::English)
            .build();

        Self {
            engine: RwLock::new(engine),
            documents: RwLock::new(HashMap::new()),
            next_id: AtomicUsize::new(0),
            doc_count: AtomicUsize::new(0),
        }
    }

    pub fn add_document(&self, file_path: PathBuf, content: String, language: String) -> usize {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        let code_doc = CodeDocument {
            id,
            file_path: file_path.clone(),
            content: content.clone(),
            language,
        };

        // Add to BM25 engine
        let document = Document {
            id,
            contents: content,
        };
        {
            let mut engine = self.engine.write().unwrap();
            engine.upsert(document);
        }

        // Store document metadata
        {
            let mut docs = self.documents.write().unwrap();
            docs.insert(id, code_doc);
        }

        self.doc_count.fetch_add(1, Ordering::SeqCst);
        id
    }

    pub fn remove_document(&self, file_path: &PathBuf) -> bool {
        let mut removed = false;

        // Find document by file path
        let doc_id = {
            let docs = self.documents.read().unwrap();
            docs.iter()
                .find(|(_, doc)| &doc.file_path == file_path)
                .map(|(id, _)| *id)
        };

        if let Some(id) = doc_id {
            // Remove from BM25 engine
            {
                let mut engine = self.engine.write().unwrap();
                engine.remove(&id);
            }

            // Remove from document store
            {
                let mut docs = self.documents.write().unwrap();
                docs.remove(&id);
            }

            self.doc_count.fetch_sub(1, Ordering::SeqCst);
            removed = true;
        }

        removed
    }

    pub fn search(&self, query: &str, limit: usize, context_lines: usize) -> Vec<CodeSearchResult> {
        let engine = self.engine.read().unwrap();
        let results = engine.search(query, limit);
        let docs = self.documents.read().unwrap();

        results
            .into_iter()
            .filter_map(|result| {
                docs.get(&result.document.id).map(|doc| CodeSearchResult {
                    score: result.score,
                    file_path: doc.file_path.clone(),
                    language: doc.language.clone(),
                    content_snippet: self.extract_snippet(
                        &result.document.contents,
                        query,
                        context_lines,
                    ),
                })
            })
            .collect()
    }

    fn extract_snippet(&self, content: &str, query: &str, context_lines: usize) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let query_lower = query.to_lowercase();

        // Find first line containing query terms
        for (i, line) in lines.iter().enumerate() {
            if line.to_lowercase().contains(&query_lower) {
                let start = i.saturating_sub(context_lines);
                let end = std::cmp::min(i + context_lines + 1, lines.len());
                return lines[start..end].join("\n");
            }
        }

        // Fallback: first few lines (context_lines * 2 + 1)
        let fallback_lines = context_lines * 2 + 1;
        lines
            .iter()
            .take(fallback_lines)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn len(&self) -> usize {
        self.doc_count.load(Ordering::SeqCst)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSearchResult {
    pub score: f32,
    pub file_path: PathBuf,
    pub language: String,
    pub content_snippet: String,
}

impl Default for BM25CodeIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_bm25_index_creation() {
        let index = BM25CodeIndex::new();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_add_and_search_document() {
        let index = BM25CodeIndex::new();

        let content = "fn hello_world() {\n    println!(\"Hello, world!\");\n}";
        let file_path = PathBuf::from("test.rs");

        index.add_document(file_path.clone(), content.to_string(), "rust".to_string());

        assert_eq!(index.len(), 1);

        let results = index.search("hello_world", 10, 2);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].file_path, file_path);
        assert_eq!(results[0].language, "rust");
    }

    #[test]
    fn test_remove_document() {
        let index = BM25CodeIndex::new();

        let file_path = PathBuf::from("test.rs");
        index.add_document(
            file_path.clone(),
            "fn test() {}".to_string(),
            "rust".to_string(),
        );

        assert_eq!(index.len(), 1);

        let removed = index.remove_document(&file_path);
        assert!(removed);
        assert_eq!(index.len(), 0);

        let results = index.search("test", 10, 2);
        assert_eq!(results.len(), 0);
    }
}
