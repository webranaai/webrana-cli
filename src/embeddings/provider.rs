// ============================================
// WEBRANA CLI - Embedding Providers
// Sprint 5.2: Intelligence & RAG
// Created by: SYNAPSE (Team Beta)
// ============================================

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::Embedding;

/// Trait for embedding providers
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate embeddings for a batch of texts
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>>;

    /// Generate embedding for a single text
    async fn embed(&self, text: &str) -> Result<Embedding> {
        let results = self.embed_batch(&[text.to_string()]).await?;
        results
            .into_iter()
            .next()
            .context("No embedding returned")
    }

    /// Get the embedding dimension
    fn dimension(&self) -> usize;

    /// Get the model name
    fn model_name(&self) -> &str;
}

/// OpenAI Embeddings Provider
pub struct OpenAIEmbeddings {
    api_key: String,
    model: String,
    dimension: usize,
    base_url: Option<String>,
}

impl OpenAIEmbeddings {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            model: "text-embedding-3-small".to_string(),
            dimension: 1536,
            base_url: None,
        }
    }

    pub fn with_model(mut self, model: &str, dimension: usize) -> Self {
        self.model = model.to_string();
        self.dimension = dimension;
        self
    }

    pub fn with_base_url(mut self, url: &str) -> Self {
        self.base_url = Some(url.to_string());
        self
    }
}

#[derive(Serialize)]
struct EmbeddingRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

#[async_trait]
impl EmbeddingProvider for OpenAIEmbeddings {
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let base_url = self
            .base_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1");
        let url = format!("{}/embeddings", base_url);

        let request = EmbeddingRequest {
            model: self.model.clone(),
            input: texts.to_vec(),
        };

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send embedding request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Embedding API error ({}): {}", status, body);
        }

        let result: EmbeddingResponse = response
            .json()
            .await
            .context("Failed to parse embedding response")?;

        Ok(result.data.into_iter().map(|d| d.embedding).collect())
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}

/// Mock embedding provider for testing
pub struct MockEmbeddingProvider {
    dimension: usize,
}

impl MockEmbeddingProvider {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

#[async_trait]
impl EmbeddingProvider for MockEmbeddingProvider {
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        // Generate deterministic embeddings based on text content
        Ok(texts
            .iter()
            .map(|text| {
                let hash = text.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32));
                (0..self.dimension)
                    .map(|i| {
                        let val = ((hash.wrapping_mul(i as u32 + 1)) % 1000) as f32 / 1000.0;
                        val * 2.0 - 1.0 // Range: -1 to 1
                    })
                    .collect()
            })
            .collect())
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_name(&self) -> &str {
        "mock-embedding"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider() {
        let provider = MockEmbeddingProvider::new(384);
        
        let embedding = provider.embed("Hello world").await.unwrap();
        assert_eq!(embedding.len(), 384);
        
        // Same text should produce same embedding
        let embedding2 = provider.embed("Hello world").await.unwrap();
        assert_eq!(embedding, embedding2);
        
        // Different text should produce different embedding
        let embedding3 = provider.embed("Goodbye world").await.unwrap();
        assert_ne!(embedding, embedding3);
    }

    #[tokio::test]
    async fn test_mock_provider_batch() {
        let provider = MockEmbeddingProvider::new(128);
        
        let embeddings = provider
            .embed_batch(&["Hello".to_string(), "World".to_string()])
            .await
            .unwrap();
        
        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].len(), 128);
        assert_eq!(embeddings[1].len(), 128);
    }
}
