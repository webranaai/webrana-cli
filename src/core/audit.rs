// ============================================
// WEBRANA CLI - Audit Logging System
// Sprint 5.3: Security Hardening
// Created by: SENTINEL (Team Beta)
// ============================================

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Audit event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditEventType {
    // Command operations
    CommandExecuted,
    CommandBlocked,
    CommandFailed,

    // File operations
    FileRead,
    FileWrite,
    FileDelete,
    FileAccessDenied,

    // LLM operations
    LlmRequest,
    LlmResponse,
    LlmError,

    // Authentication/Security
    SessionStart,
    SessionEnd,
    SecurityViolation,
    SecretDetected,

    // System operations
    ConfigChange,
    PluginLoaded,
    SkillExecuted,
    IndexingStarted,
    IndexingCompleted,

    // User interactions
    UserInput,
    UserConfirmation,
}

/// Severity levels for audit events
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Ord, PartialOrd, Eq)]
pub enum AuditSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

impl std::fmt::Display for AuditSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditSeverity::Debug => write!(f, "DEBUG"),
            AuditSeverity::Info => write!(f, "INFO"),
            AuditSeverity::Warning => write!(f, "WARN"),
            AuditSeverity::Error => write!(f, "ERROR"),
            AuditSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Single audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub timestamp: u64,
    pub event_type: AuditEventType,
    pub severity: AuditSeverity,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub session_id: Option<String>,
    pub user: Option<String>,
    pub source: Option<String>,
}

impl AuditEvent {
    pub fn new(event_type: AuditEventType, severity: AuditSeverity, message: impl Into<String>) -> Self {
        Self {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            event_type,
            severity,
            message: message.into(),
            details: None,
            session_id: None,
            user: None,
            source: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn to_log_line(&self) -> String {
        let details_str = self
            .details
            .as_ref()
            .map(|d| format!(" | {}", d))
            .unwrap_or_default();

        format!(
            "[{}] {} {:?}: {}{}",
            self.timestamp,
            self.severity,
            self.event_type,
            self.message,
            details_str
        )
    }
}

/// Audit logger configuration
#[derive(Debug, Clone)]
pub struct AuditConfig {
    /// Path to audit log file
    pub log_file: Option<PathBuf>,
    /// Minimum severity to log
    pub min_severity: AuditSeverity,
    /// Maximum events to keep in memory
    pub max_memory_events: usize,
    /// Whether to log to stdout
    pub log_to_stdout: bool,
    /// Redact sensitive data in logs
    pub redact_sensitive: bool,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            log_file: None,
            min_severity: AuditSeverity::Info,
            max_memory_events: 1000,
            log_to_stdout: false,
            redact_sensitive: true,
        }
    }
}

/// Audit logger
pub struct AuditLogger {
    config: AuditConfig,
    events: Mutex<VecDeque<AuditEvent>>,
    file_writer: Option<Mutex<BufWriter<File>>>,
    session_id: String,
}

impl AuditLogger {
    pub fn new(config: AuditConfig) -> Result<Self> {
        let file_writer = if let Some(ref path) = config.log_file {
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?;
            Some(Mutex::new(BufWriter::new(file)))
        } else {
            None
        };

        Ok(Self {
            config,
            events: Mutex::new(VecDeque::new()),
            file_writer,
            session_id: uuid::Uuid::new_v4().to_string(),
        })
    }

    /// Log an audit event
    pub fn log(&self, mut event: AuditEvent) {
        // Check severity threshold
        if event.severity < self.config.min_severity {
            return;
        }

        // Add session ID
        event.session_id = Some(self.session_id.clone());

        // Redact sensitive data if configured
        if self.config.redact_sensitive {
            event.message = self.redact_sensitive_data(&event.message);
        }

        let log_line = event.to_log_line();

        // Log to stdout if configured
        if self.config.log_to_stdout {
            eprintln!("[AUDIT] {}", log_line);
        }

        // Log to file if configured
        if let Some(ref writer) = self.file_writer {
            if let Ok(mut w) = writer.lock() {
                let _ = writeln!(w, "{}", log_line);
                let _ = w.flush();
            }
        }

        // Store in memory
        if let Ok(mut events) = self.events.lock() {
            events.push_back(event);
            while events.len() > self.config.max_memory_events {
                events.pop_front();
            }
        }
    }

    /// Log command execution
    pub fn log_command(&self, command: &str, success: bool, output: Option<&str>) {
        let event_type = if success {
            AuditEventType::CommandExecuted
        } else {
            AuditEventType::CommandFailed
        };

        let severity = if success {
            AuditSeverity::Info
        } else {
            AuditSeverity::Warning
        };

        let mut event = AuditEvent::new(event_type, severity, format!("Command: {}", command));

        if let Some(out) = output {
            let truncated: String = out.chars().take(500).collect();
            event = event.with_details(serde_json::json!({
                "output_preview": truncated,
                "output_length": out.len()
            }));
        }

        self.log(event);
    }

    /// Log blocked command
    pub fn log_command_blocked(&self, command: &str, reason: &str) {
        let event = AuditEvent::new(
            AuditEventType::CommandBlocked,
            AuditSeverity::Warning,
            format!("Blocked: {} - Reason: {}", command, reason),
        );
        self.log(event);
    }

    /// Log file operation
    pub fn log_file_op(&self, op: AuditEventType, path: &str, success: bool) {
        let severity = if success {
            AuditSeverity::Info
        } else {
            AuditSeverity::Warning
        };

        let event = AuditEvent::new(
            op,
            severity,
            format!("File: {} (success: {})", path, success),
        );
        self.log(event);
    }

    /// Log security violation
    pub fn log_security_violation(&self, message: &str, details: Option<serde_json::Value>) {
        let mut event = AuditEvent::new(
            AuditEventType::SecurityViolation,
            AuditSeverity::Critical,
            message,
        );

        if let Some(d) = details {
            event = event.with_details(d);
        }

        self.log(event);
    }

    /// Log secret detection
    pub fn log_secret_detected(&self, file: &str, secret_type: &str, line: usize) {
        let event = AuditEvent::new(
            AuditEventType::SecretDetected,
            AuditSeverity::Critical,
            format!("Secret detected in {}: {} at line {}", file, secret_type, line),
        );
        self.log(event);
    }

    /// Log LLM request
    pub fn log_llm_request(&self, model: &str, token_count: Option<usize>) {
        let mut event = AuditEvent::new(
            AuditEventType::LlmRequest,
            AuditSeverity::Debug,
            format!("LLM request to {}", model),
        );

        if let Some(tokens) = token_count {
            event = event.with_details(serde_json::json!({ "tokens": tokens }));
        }

        self.log(event);
    }

    /// Get recent events
    pub fn recent_events(&self, count: usize) -> Vec<AuditEvent> {
        if let Ok(events) = self.events.lock() {
            events.iter().rev().take(count).cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Get events by type
    pub fn events_by_type(&self, event_type: AuditEventType) -> Vec<AuditEvent> {
        if let Ok(events) = self.events.lock() {
            events
                .iter()
                .filter(|e| e.event_type == event_type)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get security violations
    pub fn security_violations(&self) -> Vec<AuditEvent> {
        if let Ok(events) = self.events.lock() {
            events
                .iter()
                .filter(|e| {
                    matches!(
                        e.event_type,
                        AuditEventType::SecurityViolation
                            | AuditEventType::CommandBlocked
                            | AuditEventType::SecretDetected
                    )
                })
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Export events to JSON
    pub fn export_json(&self) -> Result<String> {
        if let Ok(events) = self.events.lock() {
            let events_vec: Vec<_> = events.iter().collect();
            Ok(serde_json::to_string_pretty(&events_vec)?)
        } else {
            Ok("[]".to_string())
        }
    }

    /// Redact sensitive data from strings
    fn redact_sensitive_data(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Patterns to redact
        let patterns = [
            (r"sk-[a-zA-Z0-9]{20,}", "[REDACTED_KEY]"),
            (r"ghp_[a-zA-Z0-9]{36}", "[REDACTED_GH_TOKEN]"),
            (r"gho_[a-zA-Z0-9]{36}", "[REDACTED_GH_TOKEN]"),
            (r"github_pat_[a-zA-Z0-9_]{36,}", "[REDACTED_GH_PAT]"),
            (r"AKIA[0-9A-Z]{16}", "[REDACTED_AWS]"),
            (r"password[=:\s]+\S+", "password=[REDACTED]"),
            (r"secret[=:\s]+\S+", "secret=[REDACTED]"),
            (r"token[=:\s]+\S+", "token=[REDACTED]"),
            (r"Bearer\s+\S+", "Bearer [REDACTED]"),
        ];

        for (pattern, replacement) in patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                result = re.replace_all(&result, replacement).to_string();
            }
        }

        result
    }
}

/// Global audit logger instance
lazy_static::lazy_static! {
    pub static ref AUDIT: Arc<AuditLogger> = Arc::new(
        AuditLogger::new(AuditConfig::default()).expect("Failed to create audit logger")
    );
}

/// Convenience macros for audit logging
#[macro_export]
macro_rules! audit_info {
    ($event_type:expr, $($arg:tt)*) => {
        $crate::core::audit::AUDIT.log(
            $crate::core::audit::AuditEvent::new(
                $event_type,
                $crate::core::audit::AuditSeverity::Info,
                format!($($arg)*)
            )
        )
    };
}

#[macro_export]
macro_rules! audit_warn {
    ($event_type:expr, $($arg:tt)*) => {
        $crate::core::audit::AUDIT.log(
            $crate::core::audit::AuditEvent::new(
                $event_type,
                $crate::core::audit::AuditSeverity::Warning,
                format!($($arg)*)
            )
        )
    };
}

#[macro_export]
macro_rules! audit_error {
    ($event_type:expr, $($arg:tt)*) => {
        $crate::core::audit::AUDIT.log(
            $crate::core::audit::AuditEvent::new(
                $event_type,
                $crate::core::audit::AuditSeverity::Error,
                format!($($arg)*)
            )
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent::new(
            AuditEventType::CommandExecuted,
            AuditSeverity::Info,
            "Test command",
        );

        assert_eq!(event.event_type, AuditEventType::CommandExecuted);
        assert_eq!(event.severity, AuditSeverity::Info);
        assert!(event.timestamp > 0);
    }

    #[test]
    fn test_audit_logger() {
        let config = AuditConfig {
            max_memory_events: 10,
            ..Default::default()
        };

        let logger = AuditLogger::new(config).unwrap();

        // Log some events
        logger.log_command("ls -la", true, Some("file1\nfile2"));
        logger.log_command_blocked("rm -rf /", "Dangerous command");

        let events = logger.recent_events(10);
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_redact_sensitive_data() {
        let logger = AuditLogger::new(AuditConfig::default()).unwrap();

        let text = "API key: sk-1234567890abcdefghij password=secret123";
        let redacted = logger.redact_sensitive_data(text);

        assert!(!redacted.contains("sk-1234567890"));
        assert!(!redacted.contains("secret123"));
        assert!(redacted.contains("[REDACTED"));
    }

    #[test]
    fn test_severity_ordering() {
        assert!(AuditSeverity::Debug < AuditSeverity::Info);
        assert!(AuditSeverity::Info < AuditSeverity::Warning);
        assert!(AuditSeverity::Warning < AuditSeverity::Error);
        assert!(AuditSeverity::Error < AuditSeverity::Critical);
    }
}
