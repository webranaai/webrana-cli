// ============================================
// WEBRANA CLI - RAG Context Builder
// Sprint 5.2: Intelligence & RAG
// Created by: SYNAPSE (Team Beta)
// ============================================

use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

use crate::embeddings::{EmbeddingProvider, EmbeddingStore, StoredEmbedding};

/// Configuration for RAG context building
#[derive(Debug, Clone)]
pub struct RagConfig {
    /// Maximum number of chunks to retrieve
    pub top_k: usize,
    /// Minimum similarity score threshold
    pub min_score: f32,
    /// Maximum context length in characters
    pub max_context_chars: usize,
    /// Whether to include file paths in context
    pub include_file_paths: bool,
    /// Whether to include line numbers
    pub include_line_numbers: bool,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            top_k: 5,
            min_score: 0.3,
            max_context_chars: 8000,
            include_file_paths: true,
            include_line_numbers: true,
        }
    }
}

/// RAG context builder for augmenting LLM prompts
pub struct RagContext {
    provider: Arc<dyn EmbeddingProvider>,
    store: EmbeddingStore,
    config: RagConfig,
}

impl RagContext {
    /// Create new RAG context with embedding provider
    pub fn new(provider: Arc<dyn EmbeddingProvider>, config: RagConfig) -> Self {
        let dimension = provider.dimension();
        Self {
            provider,
            store: EmbeddingStore::new(dimension),
            config,
        }
    }

    /// Create with existing store
    pub fn with_store(
        provider: Arc<dyn EmbeddingProvider>,
        store: EmbeddingStore,
        config: RagConfig,
    ) -> Self {
        Self {
            provider,
            store,
            config,
        }
    }

    /// Add documents to the store
    pub async fn add_documents(&mut self, documents: Vec<Document>) -> Result<usize> {
        let mut added = 0;

        for doc in documents {
            let embedding = self.provider.embed(&doc.content).await?;
            
            let stored = StoredEmbedding {
                id: doc.id.clone(),
                text: doc.content,
                embedding,
                metadata: doc.metadata,
            };
            
            self.store.add(stored);
            added += 1;
        }

        Ok(added)
    }

    /// Retrieve relevant context for a query
    pub async fn retrieve(&self, query: &str) -> Result<Vec<RetrievedChunk>> {
        let query_embedding = self.provider.embed(query).await?;
        
        let results = self.store.search_with_threshold(
            &query_embedding,
            self.config.top_k,
            self.config.min_score,
        );

        Ok(results
            .into_iter()
            .map(|r| RetrievedChunk {
                id: r.id,
                content: r.text,
                score: r.score,
                file_path: r.metadata.get("file").cloned(),
                start_line: r.metadata.get("start_line").and_then(|s| s.parse().ok()),
                end_line: r.metadata.get("end_line").and_then(|s| s.parse().ok()),
            })
            .collect())
    }

    /// Build context string from retrieved chunks
    pub fn build_context(&self, chunks: &[RetrievedChunk]) -> String {
        let mut context = String::new();
        let mut total_chars = 0;

        for (i, chunk) in chunks.iter().enumerate() {
            // Build chunk header
            let mut header = format!("--- Relevant Code #{} ", i + 1);
            
            if self.config.include_file_paths {
                if let Some(ref path) = chunk.file_path {
                    header.push_str(&format!("({})", path));
                }
            }
            
            if self.config.include_line_numbers {
                if let (Some(start), Some(end)) = (chunk.start_line, chunk.end_line) {
                    header.push_str(&format!(" lines {}-{}", start, end));
                }
            }
            
            header.push_str(&format!(" [score: {:.2}] ---\n", chunk.score));

            // Check if adding this chunk would exceed limit
            let chunk_text = format!("{}{}\n\n", header, chunk.content);
            if total_chars + chunk_text.len() > self.config.max_context_chars {
                // Add truncated version if we have room
                let remaining = self.config.max_context_chars.saturating_sub(total_chars);
                if remaining > header.len() + 100 {
                    let truncated: String = chunk.content.chars().take(remaining - header.len() - 20).collect();
                    context.push_str(&header);
                    context.push_str(&truncated);
                    context.push_str("\n... [truncated]\n\n");
                }
                break;
            }

            context.push_str(&chunk_text);
            total_chars += chunk_text.len();
        }

        context
    }

    /// Augment a prompt with relevant context
    pub async fn augment_prompt(&self, query: &str, base_prompt: &str) -> Result<String> {
        let chunks = self.retrieve(query).await?;
        
        if chunks.is_empty() {
            return Ok(base_prompt.to_string());
        }

        let context = self.build_context(&chunks);
        
        let augmented = format!(
            "{}\n\n## Relevant Code Context\n\nThe following code snippets may be relevant to the user's query:\n\n{}\n## End of Context\n",
            base_prompt,
            context
        );

        Ok(augmented)
    }

    /// Get store reference for persistence
    pub fn store(&self) -> &EmbeddingStore {
        &self.store
    }

    /// Get mutable store reference
    pub fn store_mut(&mut self) -> &mut EmbeddingStore {
        &mut self.store
    }

    /// Load store from file
    pub fn load_store(&mut self, path: &Path) -> Result<()> {
        self.store = EmbeddingStore::load(path)?;
        Ok(())
    }

    /// Save store to file
    pub fn save_store(&self, path: &Path) -> Result<()> {
        self.store.save(path)
    }

    /// Get number of indexed documents
    pub fn document_count(&self) -> usize {
        self.store.len()
    }

    /// Clear all indexed documents
    pub fn clear(&mut self) {
        self.store.clear();
    }
}

/// Document to be indexed
#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub metadata: std::collections::HashMap<String, String>,
}

impl Document {
    pub fn new(id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Retrieved chunk with metadata
#[derive(Debug, Clone)]
pub struct RetrievedChunk {
    pub id: String,
    pub content: String,
    pub score: f32,
    pub file_path: Option<String>,
    pub start_line: Option<usize>,
    pub end_line: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embeddings::MockEmbeddingProvider;

    #[tokio::test]
    async fn test_rag_context_add_and_retrieve() {
        let provider = Arc::new(MockEmbeddingProvider::new(384));
        let config = RagConfig::default();
        let mut rag = RagContext::new(provider, config);

        // Add documents
        let docs = vec![
            Document::new("doc1", "fn authenticate(user: &str) { }"),
            Document::new("doc2", "fn connect_database() { }"),
        ];

        let added = rag.add_documents(docs).await.unwrap();
        assert_eq!(added, 2);
        assert_eq!(rag.document_count(), 2);

        // Retrieve
        let chunks = rag.retrieve("authentication").await.unwrap();
        // Mock provider returns random embeddings, so results may vary
        assert!(chunks.len() <= 5);
    }

    #[tokio::test]
    async fn test_rag_build_context() {
        let provider = Arc::new(MockEmbeddingProvider::new(384));
        let config = RagConfig {
            include_file_paths: true,
            include_line_numbers: true,
            ..Default::default()
        };
        let rag = RagContext::new(provider, config);

        let chunks = vec![
            RetrievedChunk {
                id: "chunk1".to_string(),
                content: "fn hello() { println!(\"Hello\"); }".to_string(),
                score: 0.95,
                file_path: Some("src/main.rs".to_string()),
                start_line: Some(10),
                end_line: Some(12),
            },
        ];

        let context = rag.build_context(&chunks);
        assert!(context.contains("src/main.rs"));
        assert!(context.contains("lines 10-12"));
        assert!(context.contains("0.95"));
        assert!(context.contains("fn hello()"));
    }

    #[tokio::test]
    async fn test_rag_augment_prompt() {
        let provider = Arc::new(MockEmbeddingProvider::new(384));
        let config = RagConfig::default();
        let mut rag = RagContext::new(provider, config);

        // Add a document
        let docs = vec![Document::new("doc1", "fn test_function() { }")];
        rag.add_documents(docs).await.unwrap();

        let base_prompt = "You are a helpful assistant.";
        let augmented = rag.augment_prompt("test", base_prompt).await.unwrap();

        // Should contain base prompt
        assert!(augmented.contains(base_prompt));
    }

    #[test]
    fn test_document_builder() {
        let doc = Document::new("id1", "content here")
            .with_metadata("file", "test.rs")
            .with_metadata("language", "rust");

        assert_eq!(doc.id, "id1");
        assert_eq!(doc.content, "content here");
        assert_eq!(doc.metadata.get("file"), Some(&"test.rs".to_string()));
        assert_eq!(doc.metadata.get("language"), Some(&"rust".to_string()));
    }
}
