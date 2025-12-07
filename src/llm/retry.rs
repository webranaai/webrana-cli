// ============================================
// WEBRANA CLI - Retry Logic with Exponential Backoff
// Sprint 5.1: Stability & Performance
// Created by: FORGE (Team Beta)
// ============================================

use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial delay before first retry
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Add random jitter to delays
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryConfig {
    /// Create config for aggressive retries (API rate limits)
    pub fn aggressive() -> Self {
        Self {
            max_retries: 5,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }

    /// Create config for quick retries (transient errors)
    pub fn quick() -> Self {
        Self {
            max_retries: 2,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(1),
            backoff_multiplier: 2.0,
            jitter: false,
        }
    }

    /// Calculate delay for given attempt number
    fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let base_delay = self.initial_delay.as_millis() as f64
            * self.backoff_multiplier.powi(attempt as i32);
        
        let mut delay_ms = base_delay.min(self.max_delay.as_millis() as f64);
        
        // Add jitter (±25%)
        if self.jitter {
            let jitter_range = delay_ms * 0.25;
            let jitter = (rand_simple() * 2.0 - 1.0) * jitter_range;
            delay_ms = (delay_ms + jitter).max(0.0);
        }
        
        Duration::from_millis(delay_ms as u64)
    }
}

/// Simple pseudo-random number generator (0.0 to 1.0)
fn rand_simple() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}

/// Error classification for retry decisions
pub enum RetryDecision {
    /// Should retry
    Retry,
    /// Should not retry (permanent error)
    NoRetry,
}

/// Check if an error is retryable
pub fn is_retryable_error(error: &anyhow::Error) -> RetryDecision {
    let error_str = error.to_string().to_lowercase();
    
    // Retryable errors
    let retryable_patterns = [
        "timeout",
        "rate limit",
        "429",
        "503",
        "502",
        "504",
        "connection refused",
        "connection reset",
        "temporarily unavailable",
        "overloaded",
        "try again",
    ];
    
    for pattern in &retryable_patterns {
        if error_str.contains(pattern) {
            return RetryDecision::Retry;
        }
    }
    
    // Non-retryable errors
    let permanent_patterns = [
        "invalid api key",
        "authentication",
        "unauthorized",
        "401",
        "403",
        "invalid request",
        "400",
    ];
    
    for pattern in &permanent_patterns {
        if error_str.contains(pattern) {
            return RetryDecision::NoRetry;
        }
    }
    
    // Default: retry for unknown errors
    RetryDecision::Retry
}

/// Execute an async operation with retry logic
pub async fn with_retry<F, Fut, T>(
    config: &RetryConfig,
    operation: F,
) -> anyhow::Result<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = anyhow::Result<T>>,
{
    let mut last_error = None;
    
    for attempt in 0..=config.max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                // Check if we should retry
                if attempt < config.max_retries {
                    match is_retryable_error(&e) {
                        RetryDecision::Retry => {
                            let delay = config.delay_for_attempt(attempt);
                            tracing::warn!(
                                "Attempt {} failed: {}. Retrying in {:?}...",
                                attempt + 1,
                                e,
                                delay
                            );
                            sleep(delay).await;
                        }
                        RetryDecision::NoRetry => {
                            tracing::error!("Permanent error, not retrying: {}", e);
                            return Err(e);
                        }
                    }
                }
                last_error = Some(e);
            }
        }
    }
    
    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Retry failed with no error")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delay_calculation() {
        let config = RetryConfig {
            initial_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            jitter: false,
            ..Default::default()
        };
        
        // Without jitter, delays should be deterministic
        assert_eq!(config.delay_for_attempt(0), Duration::from_millis(100));
        assert_eq!(config.delay_for_attempt(1), Duration::from_millis(200));
        assert_eq!(config.delay_for_attempt(2), Duration::from_millis(400));
    }

    #[test]
    fn test_retryable_error_detection() {
        let timeout_err = anyhow::anyhow!("Request timeout after 30s");
        assert!(matches!(is_retryable_error(&timeout_err), RetryDecision::Retry));
        
        let rate_limit_err = anyhow::anyhow!("Rate limit exceeded (429)");
        assert!(matches!(is_retryable_error(&rate_limit_err), RetryDecision::Retry));
        
        let auth_err = anyhow::anyhow!("Invalid API key");
        assert!(matches!(is_retryable_error(&auth_err), RetryDecision::NoRetry));
    }

    #[tokio::test]
    async fn test_retry_success_first_attempt() {
        let config = RetryConfig::default();
        let result = with_retry(&config, || async { Ok::<_, anyhow::Error>(42) }).await;
        assert_eq!(result.unwrap(), 42);
    }
}
