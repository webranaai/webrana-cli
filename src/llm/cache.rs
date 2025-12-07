// ============================================
// WEBRANA CLI - LLM Response Cache
// Sprint 5.1: Stability & Performance
// Created by: FORGE (Team Beta)
// ============================================

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Cache entry with TTL
struct CacheEntry {
    response: String,
    created_at: Instant,
    hits: u32,
}

/// LRU Cache for LLM responses
pub struct ResponseCache {
    entries: RwLock<HashMap<u64, CacheEntry>>,
    max_entries: usize,
    ttl: Duration,
}

impl Default for ResponseCache {
    fn default() -> Self {
        Self::new(100, Duration::from_secs(3600)) // 100 entries, 1 hour TTL
    }
}

impl ResponseCache {
    pub fn new(max_entries: usize, ttl: Duration) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            max_entries,
            ttl,
        }
    }

    /// Generate cache key from messages
    fn cache_key(messages: &[super::Message]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        for msg in messages {
            msg.role.hash(&mut hasher);
            msg.content.hash(&mut hasher);
        }
        hasher.finish()
    }

    /// Get cached response if exists and not expired
    pub fn get(&self, messages: &[super::Message]) -> Option<String> {
        let key = Self::cache_key(messages);
        let mut entries = self.entries.write().ok()?;
        
        if let Some(entry) = entries.get_mut(&key) {
            if entry.created_at.elapsed() < self.ttl {
                entry.hits += 1;
                return Some(entry.response.clone());
            } else {
                // Expired, remove it
                entries.remove(&key);
            }
        }
        None
    }

    /// Store response in cache
    pub fn set(&self, messages: &[super::Message], response: String) {
        let key = Self::cache_key(messages);
        
        if let Ok(mut entries) = self.entries.write() {
            // Evict oldest entries if at capacity
            if entries.len() >= self.max_entries {
                self.evict_oldest(&mut entries);
            }
            
            entries.insert(key, CacheEntry {
                response,
                created_at: Instant::now(),
                hits: 0,
            });
        }
    }

    /// Evict oldest/least used entries
    fn evict_oldest(&self, entries: &mut HashMap<u64, CacheEntry>) {
        // Find entry with oldest access time and lowest hits
        if let Some((&key_to_remove, _)) = entries
            .iter()
            .min_by(|(_, a), (_, b)| {
                // Prioritize removing expired entries
                let a_expired = a.created_at.elapsed() >= self.ttl;
                let b_expired = b.created_at.elapsed() >= self.ttl;
                
                if a_expired != b_expired {
                    return b_expired.cmp(&a_expired);
                }
                
                // Then by hits (remove least used)
                a.hits.cmp(&b.hits)
            })
        {
            entries.remove(&key_to_remove);
        }
    }

    /// Clear all cache entries
    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.write() {
            entries.clear();
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        if let Ok(entries) = self.entries.read() {
            let total_entries = entries.len();
            let total_hits: u32 = entries.values().map(|e| e.hits).sum();
            let expired = entries.values().filter(|e| e.created_at.elapsed() >= self.ttl).count();
            
            CacheStats {
                total_entries,
                total_hits,
                expired_entries: expired,
                max_entries: self.max_entries,
            }
        } else {
            CacheStats::default()
        }
    }
}

#[derive(Debug, Default)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_hits: u32,
    pub expired_entries: usize,
    pub max_entries: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::Message;

    #[test]
    fn test_cache_set_get() {
        let cache = ResponseCache::new(10, Duration::from_secs(60));
        let messages = vec![Message::user("Hello")];
        
        cache.set(&messages, "Hi there!".to_string());
        
        let result = cache.get(&messages);
        assert_eq!(result, Some("Hi there!".to_string()));
    }

    #[test]
    fn test_cache_miss() {
        let cache = ResponseCache::new(10, Duration::from_secs(60));
        let messages = vec![Message::user("Hello")];
        
        let result = cache.get(&messages);
        assert_eq!(result, None);
    }

    #[test]
    fn test_cache_different_messages() {
        let cache = ResponseCache::new(10, Duration::from_secs(60));
        
        let messages1 = vec![Message::user("Hello")];
        let messages2 = vec![Message::user("Goodbye")];
        
        cache.set(&messages1, "Hi!".to_string());
        cache.set(&messages2, "Bye!".to_string());
        
        assert_eq!(cache.get(&messages1), Some("Hi!".to_string()));
        assert_eq!(cache.get(&messages2), Some("Bye!".to_string()));
    }
}
