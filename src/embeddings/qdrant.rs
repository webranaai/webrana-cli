// ============================================
// WEBRANA CLI - Qdrant Vector Database Store
// Sprint 5.2: Intelligence & RAG
// Created by: SYNAPSE (Team Beta)
// ============================================

#![cfg(feature = "qdrant")]

use anyhow::{Context, Result};
use qdrant_client::prelude::*;
use qdrant_client::qdrant::{
    vectors_config::Config, CreateCollection, Distance, PointStruct, SearchPoints,
    VectorParams, VectorsConfig, Filter, Condition, FieldCondition, Match,
    value::Kind, Value as QdrantValue,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::StoredEmbedding;

/// Qdrant vector store configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantConfig {
    pub url: String,
    pub collection_name: String,
    pub vector_size: u64,
    pub on_disk: bool,
}

impl Default for QdrantConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:6334".to_string(),
            collection_name: "webrana_embeddings".to_string(),
            vector_size: 1536, // OpenAI text-embedding-3-small
            on_disk: false,
        }
    }
}

/// Qdrant-backed vector store
pub struct QdrantStore {
    client: QdrantClient,
    config: QdrantConfig,
}

impl QdrantStore {
    /// Create new Qdrant store
    pub async fn new(config: QdrantConfig) -> Result<Self> {
        let client = QdrantClient::from_url(&config.url)
            .build()
            .context("Failed to create Qdrant client")?;

        let store = Self { client, config };
        store.ensure_collection().await?;

        Ok(store)
    }

    /// Ensure collection exists
    async fn ensure_collection(&self) -> Result<()> {
        let collections = self.client.list_collections().await?;
        
        let exists = collections
            .collections
            .iter()
            .any(|c| c.name == self.config.collection_name);

        if !exists {
            self.client
                .create_collection(&CreateCollection {
                    collection_name: self.config.collection_name.clone(),
                    vectors_config: Some(VectorsConfig {
                        config: Some(Config::Params(VectorParams {
                            size: self.config.vector_size,
                            distance: Distance::Cosine.into(),
                            on_disk: Some(self.config.on_disk),
                            ..Default::default()
                        })),
                    }),
                    ..Default::default()
                })
                .await
                .context("Failed to create collection")?;

            tracing::info!("Created Qdrant collection: {}", self.config.collection_name);
        }

        Ok(())
    }

    /// Add embeddings to the store
    pub async fn add(&self, embeddings: Vec<StoredEmbedding>) -> Result<()> {
        if embeddings.is_empty() {
            return Ok(());
        }

        let points: Vec<PointStruct> = embeddings
            .into_iter()
            .enumerate()
            .map(|(idx, emb)| {
                // Convert metadata to Qdrant payload
                let mut payload: HashMap<String, QdrantValue> = HashMap::new();
                payload.insert(
                    "id".to_string(),
                    QdrantValue { kind: Some(Kind::StringValue(emb.id)) },
                );
                payload.insert(
                    "text".to_string(),
                    QdrantValue { kind: Some(Kind::StringValue(emb.text)) },
                );
                
                for (key, value) in emb.metadata {
                    payload.insert(
                        key,
                        QdrantValue { kind: Some(Kind::StringValue(value)) },
                    );
                }

                PointStruct {
                    id: Some(qdrant_client::qdrant::PointId {
                        point_id_options: Some(
                            qdrant_client::qdrant::point_id::PointIdOptions::Num(idx as u64)
                        ),
                    }),
                    vectors: Some(qdrant_client::qdrant::Vectors {
                        vectors_options: Some(
                            qdrant_client::qdrant::vectors::VectorsOptions::Vector(
                                qdrant_client::qdrant::Vector {
                                    data: emb.embedding,
                                    ..Default::default()
                                }
                            )
                        ),
                    }),
                    payload,
                }
            })
            .collect();

        self.client
            .upsert_points(&self.config.collection_name, None, points, None)
            .await
            .context("Failed to upsert points")?;

        Ok(())
    }

    /// Search for similar embeddings
    pub async fn search(
        &self,
        query_vector: &[f32],
        top_k: usize,
        min_score: f32,
    ) -> Result<Vec<SearchResult>> {
        let search_result = self
            .client
            .search_points(&SearchPoints {
                collection_name: self.config.collection_name.clone(),
                vector: query_vector.to_vec(),
                limit: top_k as u64,
                score_threshold: Some(min_score),
                with_payload: Some(true.into()),
                ..Default::default()
            })
            .await
            .context("Failed to search points")?;

        let results = search_result
            .result
            .into_iter()
            .map(|point| {
                let payload = point.payload;
                
                let id = payload
                    .get("id")
                    .and_then(|v| match &v.kind {
                        Some(Kind::StringValue(s)) => Some(s.clone()),
                        _ => None,
                    })
                    .unwrap_or_default();

                let text = payload
                    .get("text")
                    .and_then(|v| match &v.kind {
                        Some(Kind::StringValue(s)) => Some(s.clone()),
                        _ => None,
                    })
                    .unwrap_or_default();

                let mut metadata: HashMap<String, String> = HashMap::new();
                for (key, value) in payload {
                    if key != "id" && key != "text" {
                        if let Some(Kind::StringValue(s)) = value.kind {
                            metadata.insert(key, s);
                        }
                    }
                }

                SearchResult {
                    id,
                    text,
                    score: point.score,
                    metadata,
                }
            })
            .collect();

        Ok(results)
    }

    /// Search with file filter
    pub async fn search_in_file(
        &self,
        query_vector: &[f32],
        file_path: &str,
        top_k: usize,
    ) -> Result<Vec<SearchResult>> {
        let filter = Filter {
            must: vec![Condition {
                condition_one_of: Some(
                    qdrant_client::qdrant::condition::ConditionOneOf::Field(
                        FieldCondition {
                            key: "file".to_string(),
                            r#match: Some(Match {
                                match_value: Some(
                                    qdrant_client::qdrant::r#match::MatchValue::Keyword(
                                        file_path.to_string(),
                                    ),
                                ),
                            }),
                            ..Default::default()
                        },
                    ),
                ),
            }],
            ..Default::default()
        };

        let search_result = self
            .client
            .search_points(&SearchPoints {
                collection_name: self.config.collection_name.clone(),
                vector: query_vector.to_vec(),
                limit: top_k as u64,
                filter: Some(filter),
                with_payload: Some(true.into()),
                ..Default::default()
            })
            .await
            .context("Failed to search points")?;

        let results = search_result
            .result
            .into_iter()
            .map(|point| {
                let payload = point.payload;
                
                let id = payload
                    .get("id")
                    .and_then(|v| match &v.kind {
                        Some(Kind::StringValue(s)) => Some(s.clone()),
                        _ => None,
                    })
                    .unwrap_or_default();

                let text = payload
                    .get("text")
                    .and_then(|v| match &v.kind {
                        Some(Kind::StringValue(s)) => Some(s.clone()),
                        _ => None,
                    })
                    .unwrap_or_default();

                let mut metadata: HashMap<String, String> = HashMap::new();
                for (key, value) in payload {
                    if key != "id" && key != "text" {
                        if let Some(Kind::StringValue(s)) = value.kind {
                            metadata.insert(key, s);
                        }
                    }
                }

                SearchResult {
                    id,
                    text,
                    score: point.score,
                    metadata,
                }
            })
            .collect();

        Ok(results)
    }

    /// Get collection info
    pub async fn info(&self) -> Result<CollectionInfo> {
        let info = self
            .client
            .collection_info(&self.config.collection_name)
            .await
            .context("Failed to get collection info")?;

        let points_count = info
            .result
            .and_then(|r| r.points_count)
            .unwrap_or(0);

        Ok(CollectionInfo {
            name: self.config.collection_name.clone(),
            points_count,
            vector_size: self.config.vector_size,
        })
    }

    /// Delete collection
    pub async fn delete_collection(&self) -> Result<()> {
        self.client
            .delete_collection(&self.config.collection_name)
            .await
            .context("Failed to delete collection")?;

        tracing::info!("Deleted Qdrant collection: {}", self.config.collection_name);
        Ok(())
    }

    /// Clear all points in collection
    pub async fn clear(&self) -> Result<()> {
        self.delete_collection().await?;
        self.ensure_collection().await?;
        Ok(())
    }
}

/// Search result from Qdrant
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub text: String,
    pub score: f32,
    pub metadata: HashMap<String, String>,
}

/// Collection info
#[derive(Debug)]
pub struct CollectionInfo {
    pub name: String,
    pub points_count: u64,
    pub vector_size: u64,
}

#[cfg(all(test, feature = "qdrant"))]
mod tests {
    use super::*;

    // Integration tests require running Qdrant instance
    // Run with: docker run -p 6333:6333 -p 6334:6334 qdrant/qdrant

    #[tokio::test]
    #[ignore] // Requires running Qdrant
    async fn test_qdrant_store() {
        let config = QdrantConfig {
            collection_name: "test_webrana".to_string(),
            vector_size: 4,
            ..Default::default()
        };

        let store = QdrantStore::new(config).await.unwrap();

        // Add embeddings
        let embeddings = vec![
            StoredEmbedding {
                id: "test1".to_string(),
                text: "Hello world".to_string(),
                embedding: vec![0.1, 0.2, 0.3, 0.4],
                metadata: HashMap::from([("file".to_string(), "test.rs".to_string())]),
            },
            StoredEmbedding {
                id: "test2".to_string(),
                text: "Goodbye world".to_string(),
                embedding: vec![0.5, 0.6, 0.7, 0.8],
                metadata: HashMap::from([("file".to_string(), "test.rs".to_string())]),
            },
        ];

        store.add(embeddings).await.unwrap();

        // Search
        let results = store.search(&[0.1, 0.2, 0.3, 0.4], 5, 0.0).await.unwrap();
        assert!(!results.is_empty());

        // Get info
        let info = store.info().await.unwrap();
        assert!(info.points_count >= 2);

        // Cleanup
        store.delete_collection().await.unwrap();
    }
}
