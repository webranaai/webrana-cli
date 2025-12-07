// ============================================
// WEBRANA CLI - Secret Scanner
// Sprint 5.3: Security Hardening
// Created by: SENTINEL (Team Beta)
// ============================================

use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Types of secrets that can be detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecretType {
    // API Keys
    OpenAIKey,
    AnthropicKey,
    GoogleApiKey,
    StripeKey,
    SendGridKey,
    TwilioKey,
    SlackToken,
    
    // Cloud Provider Credentials
    AwsAccessKey,
    AwsSecretKey,
    GcpServiceAccount,
    AzureSecret,
    
    // Version Control
    GitHubToken,
    GitHubPat,
    GitLabToken,
    BitbucketToken,
    
    // Database
    DatabaseUrl,
    MongoDbUri,
    RedisUrl,
    
    // SSH/Certificates
    PrivateKey,
    SshPrivateKey,
    
    // Generic
    GenericApiKey,
    GenericSecret,
    GenericToken,
    Password,
    JwtToken,
    BasicAuth,
}

impl SecretType {
    pub fn severity(&self) -> SecretSeverity {
        match self {
            SecretType::PrivateKey | SecretType::SshPrivateKey => SecretSeverity::Critical,
            SecretType::AwsAccessKey | SecretType::AwsSecretKey => SecretSeverity::Critical,
            SecretType::GcpServiceAccount => SecretSeverity::Critical,
            SecretType::DatabaseUrl | SecretType::MongoDbUri => SecretSeverity::High,
            SecretType::GitHubToken | SecretType::GitHubPat => SecretSeverity::High,
            SecretType::OpenAIKey | SecretType::AnthropicKey => SecretSeverity::High,
            SecretType::Password => SecretSeverity::High,
            SecretType::JwtToken => SecretSeverity::Medium,
            _ => SecretSeverity::Medium,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            SecretType::OpenAIKey => "OpenAI API Key",
            SecretType::AnthropicKey => "Anthropic API Key",
            SecretType::GoogleApiKey => "Google API Key",
            SecretType::StripeKey => "Stripe API Key",
            SecretType::SendGridKey => "SendGrid API Key",
            SecretType::TwilioKey => "Twilio API Key",
            SecretType::SlackToken => "Slack Token",
            SecretType::AwsAccessKey => "AWS Access Key ID",
            SecretType::AwsSecretKey => "AWS Secret Access Key",
            SecretType::GcpServiceAccount => "GCP Service Account Key",
            SecretType::AzureSecret => "Azure Secret",
            SecretType::GitHubToken => "GitHub Token",
            SecretType::GitHubPat => "GitHub Personal Access Token",
            SecretType::GitLabToken => "GitLab Token",
            SecretType::BitbucketToken => "Bitbucket Token",
            SecretType::DatabaseUrl => "Database Connection URL",
            SecretType::MongoDbUri => "MongoDB URI",
            SecretType::RedisUrl => "Redis URL",
            SecretType::PrivateKey => "Private Key",
            SecretType::SshPrivateKey => "SSH Private Key",
            SecretType::GenericApiKey => "Generic API Key",
            SecretType::GenericSecret => "Generic Secret",
            SecretType::GenericToken => "Generic Token",
            SecretType::Password => "Password",
            SecretType::JwtToken => "JWT Token",
            SecretType::BasicAuth => "Basic Auth Credentials",
        }
    }
}

/// Severity of detected secrets
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SecretSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// A detected secret
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedSecret {
    pub secret_type: SecretType,
    pub severity: SecretSeverity,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub matched_text: String,  // Redacted version
    pub context: String,       // Surrounding line (redacted)
}

/// Secret scanner configuration
#[derive(Debug, Clone)]
pub struct ScannerConfig {
    /// File extensions to scan
    pub extensions: Vec<String>,
    /// Files to ignore
    pub ignore_files: Vec<String>,
    /// Directories to ignore
    pub ignore_dirs: Vec<String>,
    /// Minimum severity to report
    pub min_severity: SecretSeverity,
    /// Custom patterns to detect
    pub custom_patterns: Vec<(String, SecretType)>,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            extensions: vec![
                "rs", "py", "js", "ts", "go", "java", "rb", "php", "sh", "bash",
                "env", "yml", "yaml", "json", "toml", "xml", "conf", "config",
                "ini", "properties", "txt", "md",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            ignore_files: vec![
                "package-lock.json",
                "Cargo.lock",
                "yarn.lock",
                "poetry.lock",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            ignore_dirs: vec![
                "node_modules",
                "target",
                ".git",
                "vendor",
                "venv",
                ".venv",
                "__pycache__",
                "dist",
                "build",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            min_severity: SecretSeverity::Low,
            custom_patterns: Vec::new(),
        }
    }
}

/// Secret scanner
pub struct SecretScanner {
    config: ScannerConfig,
    patterns: HashMap<SecretType, Regex>,
}

impl SecretScanner {
    pub fn new(config: ScannerConfig) -> Self {
        let mut patterns = HashMap::new();

        // OpenAI
        patterns.insert(
            SecretType::OpenAIKey,
            Regex::new(r"sk-[a-zA-Z0-9]{20,}").unwrap(),
        );

        // Anthropic
        patterns.insert(
            SecretType::AnthropicKey,
            Regex::new(r"sk-ant-[a-zA-Z0-9\-_]{20,}").unwrap(),
        );

        // AWS
        patterns.insert(
            SecretType::AwsAccessKey,
            Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(),
        );
        patterns.insert(
            SecretType::AwsSecretKey,
            Regex::new(r#"(?i)aws_secret_access_key[\s]*[=:][\s]*['"]?([a-zA-Z0-9/+=]{40})['"]?"#).unwrap(),
        );

        // GitHub
        patterns.insert(
            SecretType::GitHubToken,
            Regex::new(r"gh[pousr]_[a-zA-Z0-9]{36,}").unwrap(),
        );
        patterns.insert(
            SecretType::GitHubPat,
            Regex::new(r"github_pat_[a-zA-Z0-9_]{22,}").unwrap(),
        );

        // GitLab
        patterns.insert(
            SecretType::GitLabToken,
            Regex::new(r"glpat-[a-zA-Z0-9\-_]{20,}").unwrap(),
        );

        // Google
        patterns.insert(
            SecretType::GoogleApiKey,
            Regex::new(r"AIza[0-9A-Za-z\-_]{35}").unwrap(),
        );

        // Stripe
        patterns.insert(
            SecretType::StripeKey,
            Regex::new(r"(?:sk|pk)_(live|test)_[a-zA-Z0-9]{20,}").unwrap(),
        );

        // Slack
        patterns.insert(
            SecretType::SlackToken,
            Regex::new(r"xox[baprs]-[a-zA-Z0-9\-]{10,}").unwrap(),
        );

        // Private Keys
        patterns.insert(
            SecretType::PrivateKey,
            Regex::new(r"-----BEGIN\s+(RSA|EC|DSA|OPENSSH|PGP)\s+PRIVATE\s+KEY-----").unwrap(),
        );

        // Database URLs
        patterns.insert(
            SecretType::DatabaseUrl,
            Regex::new(r"(?i)(postgres|mysql|mongodb|redis)://[^\s]+:[^\s]+@").unwrap(),
        );

        // MongoDB
        patterns.insert(
            SecretType::MongoDbUri,
            Regex::new(r"mongodb\+srv://[^\s]+:[^\s]+@").unwrap(),
        );

        // JWT
        patterns.insert(
            SecretType::JwtToken,
            Regex::new(r"eyJ[a-zA-Z0-9_-]+\.eyJ[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+").unwrap(),
        );

        // Generic patterns (looser, lower priority)
        patterns.insert(
            SecretType::GenericApiKey,
            Regex::new(r#"(?i)api[_-]?key[\s]*[=:][\s]*['"]?([a-zA-Z0-9_-]{20,})['"]?"#).unwrap(),
        );
        patterns.insert(
            SecretType::GenericSecret,
            Regex::new(r#"(?i)secret[\s]*[=:][\s]*['"]?([a-zA-Z0-9_-]{16,})['"]?"#).unwrap(),
        );
        patterns.insert(
            SecretType::Password,
            Regex::new(r#"(?i)password[\s]*[=:][\s]*['"]?([^\s'"]{8,})['"]?"#).unwrap(),
        );
        patterns.insert(
            SecretType::BasicAuth,
            Regex::new(r"(?i)basic\s+[a-zA-Z0-9+/=]{20,}").unwrap(),
        );

        Self { config, patterns }
    }

    /// Scan a file for secrets
    pub fn scan_file(&self, path: &Path) -> Result<Vec<DetectedSecret>> {
        let content = std::fs::read_to_string(path)?;
        self.scan_content(&content, &path.to_string_lossy())
    }

    /// Scan content string for secrets
    pub fn scan_content(&self, content: &str, file_path: &str) -> Result<Vec<DetectedSecret>> {
        let mut secrets = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            // Skip comments in common formats
            let trimmed = line.trim();
            if trimmed.starts_with("//") && !trimmed.contains("=") {
                continue;
            }
            if trimmed.starts_with('#') && !trimmed.contains("=") {
                continue;
            }

            for (secret_type, pattern) in &self.patterns {
                for mat in pattern.find_iter(line) {
                    let severity = secret_type.severity();
                    
                    // Skip if below minimum severity
                    if severity < self.config.min_severity {
                        continue;
                    }

                    // Redact the matched text
                    let matched = mat.as_str();
                    let redacted = self.redact_secret(matched);

                    // Redact the context line
                    let context = self.redact_line(line);

                    secrets.push(DetectedSecret {
                        secret_type: *secret_type,
                        severity,
                        file: file_path.to_string(),
                        line: line_num + 1,
                        column: mat.start() + 1,
                        matched_text: redacted,
                        context,
                    });
                }
            }
        }

        // Remove duplicates (same line, same type)
        secrets.dedup_by(|a, b| a.line == b.line && a.secret_type == b.secret_type);

        Ok(secrets)
    }

    /// Scan a directory recursively
    pub fn scan_directory(&self, dir: &Path) -> Result<Vec<DetectedSecret>> {
        let mut all_secrets = Vec::new();

        self.scan_dir_recursive(dir, &mut all_secrets)?;

        // Sort by severity (critical first)
        all_secrets.sort_by(|a, b| b.severity.cmp(&a.severity));

        Ok(all_secrets)
    }

    fn scan_dir_recursive(&self, dir: &Path, secrets: &mut Vec<DetectedSecret>) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy();

            // Skip ignored directories
            if path.is_dir() {
                if self.config.ignore_dirs.iter().any(|d| name == *d) {
                    continue;
                }
                self.scan_dir_recursive(&path, secrets)?;
                continue;
            }

            // Skip ignored files
            if self.config.ignore_files.iter().any(|f| name == *f) {
                continue;
            }

            // Check extension
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            
            if !self.config.extensions.iter().any(|e| e == ext) {
                // Also check files without extension (like .env, Dockerfile)
                if !name.starts_with('.') && !name.contains("Dockerfile") {
                    continue;
                }
            }

            // Scan file
            match self.scan_file(&path) {
                Ok(file_secrets) => secrets.extend(file_secrets),
                Err(e) => {
                    tracing::debug!("Failed to scan {}: {}", path.display(), e);
                }
            }
        }

        Ok(())
    }

    /// Redact a secret value
    fn redact_secret(&self, secret: &str) -> String {
        if secret.len() <= 8 {
            return "[REDACTED]".to_string();
        }
        
        let prefix: String = secret.chars().take(4).collect();
        let suffix: String = secret.chars().rev().take(4).collect::<String>().chars().rev().collect();
        format!("{}...{}", prefix, suffix)
    }

    /// Redact secrets in a line
    fn redact_line(&self, line: &str) -> String {
        let mut result = line.to_string();

        for (_, pattern) in &self.patterns {
            result = pattern.replace_all(&result, "[REDACTED]").to_string();
        }

        result
    }

    /// Check if text contains any secrets (quick check)
    pub fn contains_secrets(&self, text: &str) -> bool {
        for (_, pattern) in &self.patterns {
            if pattern.is_match(text) {
                return true;
            }
        }
        false
    }
}

/// Summary of scan results
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ScanSummary {
    pub total_files: usize,
    pub files_with_secrets: usize,
    pub total_secrets: usize,
    pub by_severity: HashMap<String, usize>,
    pub by_type: HashMap<String, usize>,
}

impl ScanSummary {
    pub fn from_secrets(secrets: &[DetectedSecret]) -> Self {
        let mut summary = Self::default();
        summary.total_secrets = secrets.len();

        let mut files = std::collections::HashSet::new();
        
        for secret in secrets {
            files.insert(&secret.file);
            
            *summary
                .by_severity
                .entry(format!("{:?}", secret.severity))
                .or_insert(0) += 1;
            
            *summary
                .by_type
                .entry(secret.secret_type.description().to_string())
                .or_insert(0) += 1;
        }

        summary.files_with_secrets = files.len();
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_openai_key() {
        let scanner = SecretScanner::new(ScannerConfig::default());
        let content = r#"OPENAI_API_KEY="sk-abcdefghijklmnopqrstuvwxyz1234567890""#;
        
        let secrets = scanner.scan_content(content, "test.env").unwrap();
        assert!(!secrets.is_empty());
        // May detect as OpenAIKey or GenericApiKey depending on pattern order
        let has_api_key = secrets.iter().any(|s| 
            matches!(s.secret_type, SecretType::OpenAIKey | SecretType::GenericApiKey)
        );
        assert!(has_api_key);
    }

    #[test]
    fn test_detect_github_pat() {
        let scanner = SecretScanner::new(ScannerConfig::default());
        let content = r#"token = "github_pat_11ABCDEFG0123456789_abcdefghijklmnopqrstuvwxyz""#;
        
        let secrets = scanner.scan_content(content, "test.toml").unwrap();
        assert!(!secrets.is_empty());
        assert_eq!(secrets[0].secret_type, SecretType::GitHubPat);
    }

    #[test]
    fn test_detect_aws_key() {
        let scanner = SecretScanner::new(ScannerConfig::default());
        let content = r#"AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE"#;
        
        let secrets = scanner.scan_content(content, ".env").unwrap();
        assert!(!secrets.is_empty());
        assert_eq!(secrets[0].secret_type, SecretType::AwsAccessKey);
    }

    #[test]
    fn test_detect_private_key() {
        let scanner = SecretScanner::new(ScannerConfig::default());
        let content = r#"-----BEGIN RSA PRIVATE KEY-----
MIIEowIBAAKCAQEA...
-----END RSA PRIVATE KEY-----"#;
        
        let secrets = scanner.scan_content(content, "key.pem").unwrap();
        assert!(!secrets.is_empty());
        assert_eq!(secrets[0].secret_type, SecretType::PrivateKey);
        assert_eq!(secrets[0].severity, SecretSeverity::Critical);
    }

    #[test]
    fn test_redact_secret() {
        let scanner = SecretScanner::new(ScannerConfig::default());
        
        let redacted = scanner.redact_secret("sk-1234567890abcdefghij");
        assert!(redacted.starts_with("sk-1"));
        assert!(redacted.ends_with("ghij"));
        assert!(redacted.contains("..."));
    }

    #[test]
    fn test_contains_secrets() {
        let scanner = SecretScanner::new(ScannerConfig::default());
        
        assert!(scanner.contains_secrets("API key: sk-abcdefghijklmnopqrst"));
        assert!(!scanner.contains_secrets("This is just normal text"));
    }
}
