// ============================================
// WEBRANA CLI - Embedding Store
// Sprint 5.2: Intelligence & RAG
// Created by: SYNAPSE (Team Beta)
// ============================================

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::{cosine_similarity, Embedding};

/// Stored embedding with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEmbedding {
    pub id: String,
    pub text: String,
    pub embedding: Embedding,
    pub metadata: HashMap<String, String>,
}

/// Search result with similarity score
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub text: String,
    pub score: f32,
    pub metadata: HashMap<String, String>,
}

/// In-memory embedding store with persistence
pub struct EmbeddingStore {
    embeddings: Vec<StoredEmbedding>,
    dimension: usize,
    id_index: HashMap<String, usize>,
}

impl EmbeddingStore {
    pub fn new(dimension: usize) -> Self {
        Self {
            embeddings: Vec::new(),
            dimension,
            id_index: HashMap::new(),
        }
    }

    /// Load store from file
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).context("Failed to read embedding store")?;
        let data: StoreData = serde_json::from_str(&content).context("Failed to parse store")?;

        let mut store = Self::new(data.dimension);
        for emb in data.embeddings {
            store.add(emb);
        }

        Ok(store)
    }

    /// Save store to file
    pub fn save(&self, path: &Path) -> Result<()> {
        let data = StoreData {
            dimension: self.dimension,
            embeddings: self.embeddings.clone(),
        };

        let content = serde_json::to_string_pretty(&data)?;
        
        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, content).context("Failed to write embedding store")?;
        Ok(())
    }

    /// Add an embedding to the store
    pub fn add(&mut self, embedding: StoredEmbedding) {
        if embedding.embedding.len() != self.dimension {
            tracing::warn!(
                "Embedding dimension mismatch: expected {}, got {}",
                self.dimension,
                embedding.embedding.len()
            );
            return;
        }

        let idx = self.embeddings.len();
        self.id_index.insert(embedding.id.clone(), idx);
        self.embeddings.push(embedding);
    }

    /// Add multiple embeddings
    pub fn add_batch(&mut self, embeddings: Vec<StoredEmbedding>) {
        for emb in embeddings {
            self.add(emb);
        }
    }

    /// Search for similar embeddings
    pub fn search(&self, query_embedding: &[f32], top_k: usize) -> Vec<SearchResult> {
        if query_embedding.len() != self.dimension {
            return vec![];
        }

        let mut results: Vec<_> = self
            .embeddings
            .iter()
            .map(|emb| {
                let score = cosine_similarity(query_embedding, &emb.embedding);
                SearchResult {
                    id: emb.id.clone(),
                    text: emb.text.clone(),
                    score,
                    metadata: emb.metadata.clone(),
                }
            })
            .collect();

        // Sort by score descending
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        results.truncate(top_k);
        results
    }

    /// Search with minimum similarity threshold
    pub fn search_with_threshold(
        &self,
        query_embedding: &[f32],
        top_k: usize,
        min_score: f32,
    ) -> Vec<SearchResult> {
        self.search(query_embedding, top_k)
            .into_iter()
            .filter(|r| r.score >= min_score)
            .collect()
    }

    /// Get embedding by ID
    pub fn get(&self, id: &str) -> Option<&StoredEmbedding> {
        self.id_index.get(id).map(|&idx| &self.embeddings[idx])
    }

    /// Remove embedding by ID
    pub fn remove(&mut self, id: &str) -> Option<StoredEmbedding> {
        if let Some(&idx) = self.id_index.get(id) {
            self.id_index.remove(id);
            // Note: This invalidates indices, so we need to rebuild
            let removed = self.embeddings.remove(idx);
            self.rebuild_index();
            Some(removed)
        } else {
            None
        }
    }

    /// Rebuild the ID index
    fn rebuild_index(&mut self) {
        self.id_index.clear();
        for (idx, emb) in self.embeddings.iter().enumerate() {
            self.id_index.insert(emb.id.clone(), idx);
        }
    }

    /// Get number of stored embeddings
    pub fn len(&self) -> usize {
        self.embeddings.len()
    }

    /// Check if store is empty
    pub fn is_empty(&self) -> bool {
        self.embeddings.is_empty()
    }

    /// Get store dimension
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Clear all embeddings
    pub fn clear(&mut self) {
        self.embeddings.clear();
        self.id_index.clear();
    }
}

#[derive(Serialize, Deserialize)]
struct StoreData {
    dimension: usize,
    embeddings: Vec<StoredEmbedding>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_embedding(id: &str, values: Vec<f32>) -> StoredEmbedding {
        StoredEmbedding {
            id: id.to_string(),
            text: format!("Text for {}", id),
            embedding: values,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_store_add_and_get() {
        let mut store = EmbeddingStore::new(3);
        
        let emb = create_test_embedding("doc1", vec![1.0, 0.0, 0.0]);
        store.add(emb);

        assert_eq!(store.len(), 1);
        assert!(store.get("doc1").is_some());
        assert!(store.get("doc2").is_none());
    }

    #[test]
    fn test_store_search() {
        let mut store = EmbeddingStore::new(3);
        
        store.add(create_test_embedding("doc1", vec![1.0, 0.0, 0.0]));
        store.add(create_test_embedding("doc2", vec![0.0, 1.0, 0.0]));
        store.add(create_test_embedding("doc3", vec![0.9, 0.1, 0.0]));

        let query = vec![1.0, 0.0, 0.0];
        let results = store.search(&query, 2);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "doc1"); // Most similar
        assert_eq!(results[1].id, "doc3"); // Second most similar
    }

    #[test]
    fn test_store_search_with_threshold() {
        let mut store = EmbeddingStore::new(3);
        
        store.add(create_test_embedding("doc1", vec![1.0, 0.0, 0.0]));
        store.add(create_test_embedding("doc2", vec![0.0, 1.0, 0.0]));

        let query = vec![1.0, 0.0, 0.0];
        let results = store.search_with_threshold(&query, 10, 0.5);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "doc1");
    }

    #[test]
    fn test_store_remove() {
        let mut store = EmbeddingStore::new(3);
        
        store.add(create_test_embedding("doc1", vec![1.0, 0.0, 0.0]));
        store.add(create_test_embedding("doc2", vec![0.0, 1.0, 0.0]));

        assert_eq!(store.len(), 2);
        
        store.remove("doc1");
        
        assert_eq!(store.len(), 1);
        assert!(store.get("doc1").is_none());
        assert!(store.get("doc2").is_some());
    }

    #[test]
    fn test_store_persistence() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("embeddings.json");

        // Create and save
        {
            let mut store = EmbeddingStore::new(3);
            store.add(create_test_embedding("doc1", vec![1.0, 0.0, 0.0]));
            store.save(&path).unwrap();
        }

        // Load and verify
        {
            let store = EmbeddingStore::load(&path).unwrap();
            assert_eq!(store.len(), 1);
            assert!(store.get("doc1").is_some());
        }
    }
}
