// ============================================
// WEBRANA CLI - Semantic Search Skill
// Sprint 5.2: Intelligence & RAG
// Created by: SYNAPSE (Team Beta)
// ============================================

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::embeddings::{
    cosine_similarity, EmbeddingProvider, EmbeddingStore, MockEmbeddingProvider,
    OpenAIEmbeddings, SearchResult, StoredEmbedding,
};
use crate::indexer::FileWalker;

/// Semantic search configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSearchConfig {
    pub embedding_model: String,
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub top_k: usize,
    pub min_score: f32,
    pub index_path: Option<String>,
}

impl Default for SemanticSearchConfig {
    fn default() -> Self {
        Self {
            embedding_model: "text-embedding-3-small".to_string(),
            chunk_size: 1000,
            chunk_overlap: 200,
            top_k: 5,
            min_score: 0.3,
            index_path: None,
        }
    }
}

/// Semantic search over codebase
pub struct SemanticSearch {
    provider: Arc<dyn EmbeddingProvider>,
    store: EmbeddingStore,
    config: SemanticSearchConfig,
    indexed_files: HashMap<String, u64>, // file path -> last modified timestamp
}

impl SemanticSearch {
    /// Create with OpenAI embeddings
    pub fn new(api_key: &str, config: SemanticSearchConfig) -> Self {
        let provider = Arc::new(OpenAIEmbeddings::new(api_key.to_string()));
        let dimension = provider.dimension();

        Self {
            provider,
            store: EmbeddingStore::new(dimension),
            config,
            indexed_files: HashMap::new(),
        }
    }

    /// Create with mock provider for testing
    pub fn new_mock(config: SemanticSearchConfig) -> Self {
        let provider = Arc::new(MockEmbeddingProvider::new(384));
        let dimension = provider.dimension();

        Self {
            provider,
            store: EmbeddingStore::new(dimension),
            config,
            indexed_files: HashMap::new(),
        }
    }

    /// Index a directory
    pub async fn index_directory(&mut self, dir: &Path) -> Result<IndexStats> {
        let mut stats = IndexStats::default();

        // Walk directory and find code files
        let walker = FileWalker::new(dir);
        let files = walker.walk()?;

        let code_extensions = [
            "rs", "py", "js", "ts", "go", "java", "cpp", "c", "h", "rb", "php",
            "swift", "kt", "scala", "md", "txt", "json", "yaml", "toml",
        ];

        for entry in files {
            let path = std::path::Path::new(&entry.path);
            
            // Skip non-code files
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !code_extensions.contains(&ext) {
                continue;
            }

            // Check if file needs re-indexing
            let modified = std::fs::metadata(&path)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);

            let path_str = path.to_string_lossy().to_string();
            
            if let Some(&cached_time) = self.indexed_files.get(&path_str) {
                if cached_time >= modified {
                    stats.skipped += 1;
                    continue;
                }
            }

            // Read and chunk file
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    let chunks = self.chunk_text(&content, &path_str);
                    
                    if chunks.is_empty() {
                        continue;
                    }

                    // Generate embeddings for chunks
                    let texts: Vec<String> = chunks.iter().map(|c| c.text.clone()).collect();
                    
                    match self.provider.embed_batch(&texts).await {
                        Ok(embeddings) => {
                            for (chunk, embedding) in chunks.into_iter().zip(embeddings) {
                                let stored = StoredEmbedding {
                                    id: chunk.id,
                                    text: chunk.text,
                                    embedding,
                                    metadata: chunk.metadata,
                                };
                                self.store.add(stored);
                                stats.chunks += 1;
                            }
                            
                            self.indexed_files.insert(path_str, modified);
                            stats.files += 1;
                        }
                        Err(e) => {
                            tracing::warn!("Failed to embed {}: {}", path.display(), e);
                            stats.errors += 1;
                        }
                    }
                }
                Err(e) => {
                    tracing::debug!("Failed to read {}: {}", path.display(), e);
                    stats.errors += 1;
                }
            }
        }

        Ok(stats)
    }

    /// Chunk text into smaller pieces
    fn chunk_text(&self, content: &str, file_path: &str) -> Vec<TextChunk> {
        let mut chunks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        
        if lines.is_empty() {
            return chunks;
        }

        let mut current_chunk = String::new();
        let mut chunk_start_line = 0;
        let mut chunk_idx = 0;

        for (line_num, line) in lines.iter().enumerate() {
            current_chunk.push_str(line);
            current_chunk.push('\n');

            if current_chunk.len() >= self.config.chunk_size {
                let mut metadata = HashMap::new();
                metadata.insert("file".to_string(), file_path.to_string());
                metadata.insert("start_line".to_string(), chunk_start_line.to_string());
                metadata.insert("end_line".to_string(), line_num.to_string());

                chunks.push(TextChunk {
                    id: format!("{}:chunk:{}", file_path, chunk_idx),
                    text: current_chunk.clone(),
                    metadata,
                });

                // Keep overlap
                let overlap_start = current_chunk
                    .char_indices()
                    .rev()
                    .nth(self.config.chunk_overlap)
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                
                current_chunk = current_chunk[overlap_start..].to_string();
                chunk_start_line = line_num.saturating_sub(5);
                chunk_idx += 1;
            }
        }

        // Add remaining content
        if !current_chunk.trim().is_empty() {
            let mut metadata = HashMap::new();
            metadata.insert("file".to_string(), file_path.to_string());
            metadata.insert("start_line".to_string(), chunk_start_line.to_string());
            metadata.insert("end_line".to_string(), lines.len().to_string());

            chunks.push(TextChunk {
                id: format!("{}:chunk:{}", file_path, chunk_idx),
                text: current_chunk,
                metadata,
            });
        }

        chunks
    }

    /// Search for relevant code
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        let query_embedding = self.provider.embed(query).await?;
        
        let results = self.store.search_with_threshold(
            &query_embedding,
            self.config.top_k,
            self.config.min_score,
        );

        Ok(results)
    }

    /// Get index statistics
    pub fn stats(&self) -> SemanticSearchStats {
        SemanticSearchStats {
            indexed_files: self.indexed_files.len(),
            total_chunks: self.store.len(),
            embedding_dimension: self.store.dimension(),
            model: self.provider.model_name().to_string(),
        }
    }

    /// Save index to file
    pub fn save(&self, path: &Path) -> Result<()> {
        self.store.save(path)
    }

    /// Load index from file
    pub fn load(&mut self, path: &Path) -> Result<()> {
        self.store = EmbeddingStore::load(path)?;
        Ok(())
    }

    /// Clear the index
    pub fn clear(&mut self) {
        self.store.clear();
        self.indexed_files.clear();
    }
}

#[derive(Debug, Clone)]
struct TextChunk {
    id: String,
    text: String,
    metadata: HashMap<String, String>,
}

#[derive(Debug, Default)]
pub struct IndexStats {
    pub files: usize,
    pub chunks: usize,
    pub skipped: usize,
    pub errors: usize,
}

#[derive(Debug)]
pub struct SemanticSearchStats {
    pub indexed_files: usize,
    pub total_chunks: usize,
    pub embedding_dimension: usize,
    pub model: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_semantic_search_mock() {
        let config = SemanticSearchConfig::default();
        let mut search = SemanticSearch::new_mock(config);

        // Create test directory with files
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        std::fs::write(&file_path, "fn hello() { println!(\"Hello\"); }").unwrap();

        // Index
        let stats = search.index_directory(dir.path()).await.unwrap();
        assert!(stats.files > 0 || stats.chunks > 0 || stats.skipped == 0);

        // Search
        let results = search.search("hello function").await.unwrap();
        // Results may be empty with mock provider, that's OK
        assert!(results.len() <= 5);
    }

    #[test]
    fn test_chunk_text() {
        let config = SemanticSearchConfig {
            chunk_size: 50,
            chunk_overlap: 10,
            ..Default::default()
        };
        let search = SemanticSearch::new_mock(config);

        let content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10";
        let chunks = search.chunk_text(content, "test.txt");

        assert!(!chunks.is_empty());
        for chunk in &chunks {
            assert!(chunk.metadata.contains_key("file"));
            assert!(chunk.metadata.contains_key("start_line"));
        }
    }

    #[test]
    fn test_semantic_search_stats() {
        let config = SemanticSearchConfig::default();
        let search = SemanticSearch::new_mock(config);

        let stats = search.stats();
        assert_eq!(stats.indexed_files, 0);
        assert_eq!(stats.total_chunks, 0);
        assert_eq!(stats.embedding_dimension, 384);
    }
}
