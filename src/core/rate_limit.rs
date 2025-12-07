// ============================================
// WEBRANA CLI - Rate Limiter
// Sprint 5.3: Security Hardening
// Created by: SENTINEL (Team Beta)
// ============================================

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Rate limiter using token bucket algorithm
pub struct RateLimiter {
    buckets: Mutex<HashMap<String, TokenBucket>>,
    default_config: RateLimitConfig,
}

/// Configuration for rate limiting
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub max_requests: usize,
    /// Time window duration
    pub window: Duration,
    /// Burst allowance (extra requests allowed in short bursts)
    pub burst: usize,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 60,           // 60 requests
            window: Duration::from_secs(60), // per minute
            burst: 10,                  // allow 10 extra in bursts
        }
    }
}

impl RateLimitConfig {
    /// Create config for API calls
    pub fn api() -> Self {
        Self {
            max_requests: 100,
            window: Duration::from_secs(60),
            burst: 20,
        }
    }

    /// Create config for LLM calls (more restrictive)
    pub fn llm() -> Self {
        Self {
            max_requests: 20,
            window: Duration::from_secs(60),
            burst: 5,
        }
    }

    /// Create config for file operations
    pub fn file_ops() -> Self {
        Self {
            max_requests: 200,
            window: Duration::from_secs(60),
            burst: 50,
        }
    }

    /// Create config for command execution
    pub fn commands() -> Self {
        Self {
            max_requests: 30,
            window: Duration::from_secs(60),
            burst: 10,
        }
    }
}

/// Token bucket for rate limiting
struct TokenBucket {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64,  // tokens per second
    last_update: Instant,
}

impl TokenBucket {
    fn new(config: &RateLimitConfig) -> Self {
        let max_tokens = (config.max_requests + config.burst) as f64;
        let refill_rate = config.max_requests as f64 / config.window.as_secs_f64();

        Self {
            tokens: max_tokens,
            max_tokens,
            refill_rate,
            last_update: Instant::now(),
        }
    }

    fn try_acquire(&mut self, tokens: f64) -> bool {
        self.refill();

        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_update = now;
    }

    fn available_tokens(&mut self) -> f64 {
        self.refill();
        self.tokens
    }

    fn time_until_available(&mut self, tokens: f64) -> Duration {
        self.refill();
        
        if self.tokens >= tokens {
            Duration::ZERO
        } else {
            let needed = tokens - self.tokens;
            Duration::from_secs_f64(needed / self.refill_rate)
        }
    }
}

impl RateLimiter {
    pub fn new(default_config: RateLimitConfig) -> Self {
        Self {
            buckets: Mutex::new(HashMap::new()),
            default_config,
        }
    }

    /// Try to acquire permission for a request
    pub fn try_acquire(&self, key: &str) -> bool {
        self.try_acquire_n(key, 1.0)
    }

    /// Try to acquire N tokens
    pub fn try_acquire_n(&self, key: &str, tokens: f64) -> bool {
        let mut buckets = self.buckets.lock().unwrap();
        
        let bucket = buckets
            .entry(key.to_string())
            .or_insert_with(|| TokenBucket::new(&self.default_config));

        bucket.try_acquire(tokens)
    }

    /// Check if a request would be allowed (without consuming tokens)
    pub fn would_allow(&self, key: &str) -> bool {
        let mut buckets = self.buckets.lock().unwrap();
        
        if let Some(bucket) = buckets.get_mut(key) {
            bucket.available_tokens() >= 1.0
        } else {
            true
        }
    }

    /// Get time until next request would be allowed
    pub fn time_until_allowed(&self, key: &str) -> Duration {
        let mut buckets = self.buckets.lock().unwrap();
        
        if let Some(bucket) = buckets.get_mut(key) {
            bucket.time_until_available(1.0)
        } else {
            Duration::ZERO
        }
    }

    /// Get remaining tokens for a key
    pub fn remaining(&self, key: &str) -> usize {
        let mut buckets = self.buckets.lock().unwrap();
        
        if let Some(bucket) = buckets.get_mut(key) {
            bucket.available_tokens() as usize
        } else {
            self.default_config.max_requests + self.default_config.burst
        }
    }

    /// Reset rate limit for a key
    pub fn reset(&self, key: &str) {
        let mut buckets = self.buckets.lock().unwrap();
        buckets.remove(key);
    }

    /// Reset all rate limits
    pub fn reset_all(&self) {
        let mut buckets = self.buckets.lock().unwrap();
        buckets.clear();
    }

    /// Create a scoped rate limiter with custom config
    pub fn scoped(&self, key: &str, config: RateLimitConfig) -> ScopedRateLimiter {
        ScopedRateLimiter {
            limiter: self,
            key: key.to_string(),
            config,
        }
    }
}

/// Scoped rate limiter with custom configuration
pub struct ScopedRateLimiter<'a> {
    limiter: &'a RateLimiter,
    key: String,
    config: RateLimitConfig,
}

impl<'a> ScopedRateLimiter<'a> {
    pub fn try_acquire(&self) -> bool {
        // Use a separate bucket for scoped limiters
        let scoped_key = format!("{}:{}", self.key, self.config.max_requests);
        
        let mut buckets = self.limiter.buckets.lock().unwrap();
        let bucket = buckets
            .entry(scoped_key)
            .or_insert_with(|| TokenBucket::new(&self.config));

        bucket.try_acquire(1.0)
    }
}

/// Global rate limiters for different operations
lazy_static::lazy_static! {
    pub static ref API_LIMITER: RateLimiter = RateLimiter::new(RateLimitConfig::api());
    pub static ref LLM_LIMITER: RateLimiter = RateLimiter::new(RateLimitConfig::llm());
    pub static ref FILE_LIMITER: RateLimiter = RateLimiter::new(RateLimitConfig::file_ops());
    pub static ref CMD_LIMITER: RateLimiter = RateLimiter::new(RateLimitConfig::commands());
}

/// Result of rate limit check
#[derive(Debug, Clone)]
pub enum RateLimitResult {
    Allowed,
    Limited { retry_after: Duration },
}

impl RateLimitResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, RateLimitResult::Allowed)
    }
}

/// Check rate limit and return result
pub fn check_rate_limit(limiter: &RateLimiter, key: &str) -> RateLimitResult {
    if limiter.try_acquire(key) {
        RateLimitResult::Allowed
    } else {
        RateLimitResult::Limited {
            retry_after: limiter.time_until_allowed(key),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_rate_limiter_basic() {
        let config = RateLimitConfig {
            max_requests: 5,
            window: Duration::from_secs(1),
            burst: 2,
        };
        let limiter = RateLimiter::new(config);

        // Should allow initial requests up to max + burst
        for _ in 0..7 {
            assert!(limiter.try_acquire("test"));
        }

        // 8th request should be denied
        assert!(!limiter.try_acquire("test"));
    }

    #[test]
    fn test_rate_limiter_refill() {
        let config = RateLimitConfig {
            max_requests: 10,
            window: Duration::from_millis(100),
            burst: 0,
        };
        let limiter = RateLimiter::new(config);

        // Exhaust tokens
        for _ in 0..10 {
            limiter.try_acquire("test");
        }
        assert!(!limiter.try_acquire("test"));

        // Wait for refill
        sleep(Duration::from_millis(50));

        // Should have some tokens now
        assert!(limiter.try_acquire("test"));
    }

    #[test]
    fn test_rate_limiter_different_keys() {
        let limiter = RateLimiter::new(RateLimitConfig {
            max_requests: 2,
            window: Duration::from_secs(1),
            burst: 0,
        });

        // Different keys have separate buckets
        assert!(limiter.try_acquire("key1"));
        assert!(limiter.try_acquire("key1"));
        assert!(!limiter.try_acquire("key1"));

        assert!(limiter.try_acquire("key2"));
        assert!(limiter.try_acquire("key2"));
        assert!(!limiter.try_acquire("key2"));
    }

    #[test]
    fn test_remaining_tokens() {
        let config = RateLimitConfig {
            max_requests: 10,
            window: Duration::from_secs(1),
            burst: 5,
        };
        let limiter = RateLimiter::new(config);

        assert_eq!(limiter.remaining("test"), 15); // max + burst

        limiter.try_acquire("test");
        assert_eq!(limiter.remaining("test"), 14);
    }

    #[test]
    fn test_reset() {
        let limiter = RateLimiter::new(RateLimitConfig {
            max_requests: 2,
            window: Duration::from_secs(60),
            burst: 0,
        });

        limiter.try_acquire("test");
        limiter.try_acquire("test");
        assert!(!limiter.try_acquire("test"));

        limiter.reset("test");
        assert!(limiter.try_acquire("test"));
    }
}
