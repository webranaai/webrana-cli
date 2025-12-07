// ============================================
// WEBRANA CLI - Performance Metrics
// Sprint 5.1: Stability & Performance
// Created by: FORGE (Team Beta)
// ============================================

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Performance metrics collector
pub struct Metrics {
    timers: RwLock<HashMap<String, Vec<Duration>>>,
    counters: RwLock<HashMap<String, u64>>,
    start_time: Instant,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            timers: RwLock::new(HashMap::new()),
            counters: RwLock::new(HashMap::new()),
            start_time: Instant::now(),
        }
    }

    /// Record a timing measurement
    pub fn record_time(&self, name: &str, duration: Duration) {
        if let Ok(mut timers) = self.timers.write() {
            timers
                .entry(name.to_string())
                .or_insert_with(Vec::new)
                .push(duration);
        }
    }

    /// Time a closure and record the duration
    pub fn time<F, T>(&self, name: &str, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        let start = Instant::now();
        let result = f();
        self.record_time(name, start.elapsed());
        result
    }

    /// Time an async closure and record the duration
    pub async fn time_async<F, Fut, T>(&self, name: &str, f: F) -> T
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let start = Instant::now();
        let result = f().await;
        self.record_time(name, start.elapsed());
        result
    }

    /// Increment a counter
    pub fn increment(&self, name: &str) {
        self.increment_by(name, 1);
    }

    /// Increment a counter by a specific amount
    pub fn increment_by(&self, name: &str, amount: u64) {
        if let Ok(mut counters) = self.counters.write() {
            *counters.entry(name.to_string()).or_insert(0) += amount;
        }
    }

    /// Get counter value
    pub fn get_counter(&self, name: &str) -> u64 {
        self.counters
            .read()
            .ok()
            .and_then(|c| c.get(name).copied())
            .unwrap_or(0)
    }

    /// Get timing statistics for a metric
    pub fn get_timing_stats(&self, name: &str) -> Option<TimingStats> {
        let timers = self.timers.read().ok()?;
        let durations = timers.get(name)?;
        
        if durations.is_empty() {
            return None;
        }

        let total: Duration = durations.iter().sum();
        let count = durations.len();
        let avg = total / count as u32;
        
        let mut sorted: Vec<_> = durations.iter().collect();
        sorted.sort();
        
        let min = **sorted.first()?;
        let max = **sorted.last()?;
        let p50 = *sorted[count / 2];
        let p95 = *sorted[(count as f64 * 0.95) as usize];
        let p99 = *sorted[(count as f64 * 0.99).min((count - 1) as f64) as usize];

        Some(TimingStats {
            count,
            total,
            avg,
            min,
            max,
            p50,
            p95,
            p99,
        })
    }

    /// Get all metrics as a summary
    pub fn summary(&self) -> MetricsSummary {
        let uptime = self.start_time.elapsed();
        
        let timers = self.timers.read().ok();
        let counters = self.counters.read().ok();

        let timing_stats: HashMap<String, TimingStats> = timers
            .map(|t| {
                t.keys()
                    .filter_map(|k| self.get_timing_stats(k).map(|s| (k.clone(), s)))
                    .collect()
            })
            .unwrap_or_default();

        let counter_values: HashMap<String, u64> = counters
            .map(|c| c.clone())
            .unwrap_or_default();

        MetricsSummary {
            uptime,
            timings: timing_stats,
            counters: counter_values,
        }
    }

    /// Reset all metrics
    pub fn reset(&self) {
        if let Ok(mut timers) = self.timers.write() {
            timers.clear();
        }
        if let Ok(mut counters) = self.counters.write() {
            counters.clear();
        }
    }
}

#[derive(Debug, Clone)]
pub struct TimingStats {
    pub count: usize,
    pub total: Duration,
    pub avg: Duration,
    pub min: Duration,
    pub max: Duration,
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
}

#[derive(Debug)]
pub struct MetricsSummary {
    pub uptime: Duration,
    pub timings: HashMap<String, TimingStats>,
    pub counters: HashMap<String, u64>,
}

impl std::fmt::Display for MetricsSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Performance Metrics ===")?;
        writeln!(f, "Uptime: {:?}", self.uptime)?;
        
        if !self.counters.is_empty() {
            writeln!(f, "\nCounters:")?;
            for (name, value) in &self.counters {
                writeln!(f, "  {}: {}", name, value)?;
            }
        }

        if !self.timings.is_empty() {
            writeln!(f, "\nTimings:")?;
            for (name, stats) in &self.timings {
                writeln!(
                    f,
                    "  {}: count={}, avg={:?}, p50={:?}, p95={:?}, p99={:?}",
                    name, stats.count, stats.avg, stats.p50, stats.p95, stats.p99
                )?;
            }
        }

        Ok(())
    }
}

/// Global metrics instance
lazy_static::lazy_static! {
    pub static ref METRICS: Metrics = Metrics::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter() {
        let metrics = Metrics::new();
        metrics.increment("test_counter");
        metrics.increment("test_counter");
        metrics.increment_by("test_counter", 3);
        
        assert_eq!(metrics.get_counter("test_counter"), 5);
    }

    #[test]
    fn test_timing() {
        let metrics = Metrics::new();
        
        metrics.time("test_op", || {
            std::thread::sleep(Duration::from_millis(10));
        });
        
        let stats = metrics.get_timing_stats("test_op");
        assert!(stats.is_some());
        assert_eq!(stats.unwrap().count, 1);
    }

    #[test]
    fn test_timing_stats() {
        let metrics = Metrics::new();
        
        metrics.record_time("op", Duration::from_millis(10));
        metrics.record_time("op", Duration::from_millis(20));
        metrics.record_time("op", Duration::from_millis(30));
        
        let stats = metrics.get_timing_stats("op").unwrap();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.min, Duration::from_millis(10));
        assert_eq!(stats.max, Duration::from_millis(30));
    }

    #[test]
    fn test_summary() {
        let metrics = Metrics::new();
        metrics.increment("requests");
        metrics.record_time("latency", Duration::from_millis(50));
        
        let summary = metrics.summary();
        assert!(summary.counters.contains_key("requests"));
        assert!(summary.timings.contains_key("latency"));
    }
}
