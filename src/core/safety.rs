// ============================================
// WEBRANA CLI - Safety & Security Module
// Created by: SENTINEL (Team Beta)
// ============================================

use anyhow::{anyhow, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Security configuration for Webrana CLI
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Working directory boundary
    pub working_dir: PathBuf,

    /// Allow commands outside working dir
    pub allow_global_access: bool,

    /// Commands that are always blocked
    pub blocked_commands: HashSet<String>,

    /// Dangerous patterns to block
    pub dangerous_patterns: Vec<String>,

    /// Sensitive files that cannot be accessed
    pub sensitive_files: Vec<String>,

    /// Maximum file size for read operations (bytes)
    pub max_file_size: u64,

    /// Require confirmation for destructive operations
    pub require_confirmation: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        let mut blocked_commands = HashSet::new();
        // Block extremely dangerous commands
        blocked_commands.insert("rm -rf /".to_string());
        blocked_commands.insert("rm -rf /*".to_string());
        blocked_commands.insert("mkfs".to_string());
        blocked_commands.insert("dd if=/dev/zero".to_string());
        blocked_commands.insert(":(){:|:&};:".to_string()); // Fork bomb
        blocked_commands.insert("chmod -R 777 /".to_string());
        blocked_commands.insert("chown -R".to_string());

        let dangerous_patterns = vec![
            "rm -rf".to_string(),
            "rm -fr".to_string(),
            "> /dev/sd".to_string(),
            "mkfs.".to_string(),
            ":(){ :|:& };:".to_string(),
            "curl | bash".to_string(),
            "curl | sh".to_string(),
            "wget | bash".to_string(),
            "wget | sh".to_string(),
            "> /etc/".to_string(),
            "chmod 777".to_string(),
            "chmod -R 777".to_string(),
        ];

        let sensitive_files = vec![
            "/etc/passwd".to_string(),
            "/etc/shadow".to_string(),
            "/etc/sudoers".to_string(),
            ".ssh/id_rsa".to_string(),
            ".ssh/id_ed25519".to_string(),
            ".aws/credentials".to_string(),
            ".env".to_string(),
            ".netrc".to_string(),
            ".pgpass".to_string(),
            ".docker/config.json".to_string(),
        ];

        Self {
            working_dir: std::env::current_dir().unwrap_or_default(),
            allow_global_access: false,
            blocked_commands,
            dangerous_patterns,
            sensitive_files,
            max_file_size: 10 * 1024 * 1024, // 10MB
            require_confirmation: true,
        }
    }
}

/// Input sanitizer for various operations
pub struct InputSanitizer {
    config: SecurityConfig,
}

impl InputSanitizer {
    pub fn new(config: SecurityConfig) -> Self {
        Self { config }
    }

    pub fn with_default() -> Self {
        Self::new(SecurityConfig::default())
    }

    /// Validate and sanitize a file path
    pub fn validate_path(&self, path: &str) -> Result<PathBuf> {
        let path = Path::new(path);

        // Resolve to absolute path
        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.config.working_dir.join(path)
        };

        // Canonicalize to resolve .. and symlinks
        let canonical = absolute.canonicalize().unwrap_or_else(|_| absolute.clone());

        // Check if within working directory (unless global access allowed)
        if !self.config.allow_global_access {
            if !canonical.starts_with(&self.config.working_dir) {
                return Err(anyhow!(
                    "Access denied: path '{}' is outside working directory",
                    path.display()
                ));
            }
        }

        // Check against sensitive files
        let path_str = canonical.to_string_lossy();
        for sensitive in &self.config.sensitive_files {
            if path_str.contains(sensitive) {
                return Err(anyhow!(
                    "Access denied: '{}' matches sensitive file pattern",
                    sensitive
                ));
            }
        }

        Ok(canonical)
    }

    /// Validate a shell command
    pub fn validate_command(&self, command: &str) -> Result<CommandRisk> {
        let command_lower = command.to_lowercase();

        // Check blocked commands
        for blocked in &self.config.blocked_commands {
            if command_lower.contains(&blocked.to_lowercase()) {
                return Err(anyhow!(
                    "Command blocked: contains dangerous pattern '{}'",
                    blocked
                ));
            }
        }

        // Check dangerous patterns
        for pattern in &self.config.dangerous_patterns {
            if command_lower.contains(&pattern.to_lowercase()) {
                return Ok(CommandRisk::High(format!(
                    "Command contains dangerous pattern: {}",
                    pattern
                )));
            }
        }

        // Assess risk level
        let risk = self.assess_command_risk(command);

        Ok(risk)
    }

    /// Assess the risk level of a command
    fn assess_command_risk(&self, command: &str) -> CommandRisk {
        let command_lower = command.to_lowercase();

        // High risk patterns
        let high_risk = [
            "sudo",
            "su ",
            "doas",
            "rm ",
            "rmdir",
            "unlink",
            "mv ",
            "cp ",
            "chmod",
            "chown",
            "chgrp",
            "kill",
            "pkill",
            "killall",
            "shutdown",
            "reboot",
            "halt",
            "systemctl",
            "service",
            "iptables",
            "firewall",
            "mount",
            "umount",
            "fdisk",
            "parted",
            "useradd",
            "userdel",
            "usermod",
            "passwd",
            "crontab",
            "curl",
            "wget", // Network downloads
            "ssh",
            "scp",
            "docker",
            "podman",
            "git push",
            "git remote",
        ];

        for pattern in high_risk {
            if command_lower.contains(pattern) {
                return CommandRisk::High(format!("Contains high-risk command: {}", pattern));
            }
        }

        // Medium risk patterns
        let medium_risk = [
            "git ", "npm ", "cargo ", "pip ", "make", "cmake", "apt", "yum", "dnf", "brew", "cat ",
            "head ", "tail ", "grep ", "find ", "locate", "echo ", "printf", "touch ", "mkdir ",
            "sed ", "awk ", "tar ", "zip ", "unzip",
        ];

        for pattern in medium_risk {
            if command_lower.contains(pattern) {
                return CommandRisk::Medium(format!("Contains modification command: {}", pattern));
            }
        }

        // Low risk - read-only commands
        let low_risk = [
            "ls",
            "pwd",
            "whoami",
            "date",
            "cal",
            "cat ",
            "less ",
            "more ",
            "head ",
            "tail ",
            "wc ",
            "sort ",
            "uniq ",
            "ps ",
            "top",
            "htop",
            "df ",
            "du ",
            "env",
            "printenv",
            "which ",
            "whereis ",
            "type ",
            "file ",
            "stat ",
            "git status",
            "git log",
            "git diff",
            "git branch",
            "cargo check",
            "cargo test",
            "cargo build",
            "npm list",
            "npm info",
        ];

        for pattern in low_risk {
            if command_lower.starts_with(pattern)
                || command_lower.contains(&format!(" {}", pattern))
            {
                return CommandRisk::Low;
            }
        }

        // Default to medium for unknown commands
        CommandRisk::Medium("Unknown command - treating as medium risk".to_string())
    }

    /// Sanitize output to remove sensitive information
    pub fn sanitize_output(&self, output: &str) -> String {
        let mut sanitized = output.to_string();

        // Patterns to redact
        let redact_patterns = [
            // API keys
            (r"sk-[a-zA-Z0-9]{20,}", "[REDACTED_API_KEY]"),
            (r"api[_-]?key[=:][\s]*[\w-]+", "[REDACTED_API_KEY]"),
            // AWS
            (r"AKIA[0-9A-Z]{16}", "[REDACTED_AWS_KEY]"),
            (
                r"aws_secret_access_key[\s]*=[\s]*\S+",
                "[REDACTED_AWS_SECRET]",
            ),
            // Generic secrets
            (r"password[=:][\s]*\S+", "[REDACTED_PASSWORD]"),
            (r"secret[=:][\s]*\S+", "[REDACTED_SECRET]"),
            (r"token[=:][\s]*\S+", "[REDACTED_TOKEN]"),
            // SSH keys
            (r"-----BEGIN .+ PRIVATE KEY-----", "[REDACTED_PRIVATE_KEY]"),
        ];

        for (pattern, replacement) in redact_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                sanitized = re.replace_all(&sanitized, replacement).to_string();
            }
        }

        sanitized
    }

    /// Check if a path is a sensitive file
    pub fn is_sensitive_file(&self, path: &str) -> bool {
        for sensitive in &self.config.sensitive_files {
            if path.contains(sensitive) {
                return true;
            }
        }
        false
    }
}

/// Risk level for commands
#[derive(Debug, Clone, PartialEq)]
pub enum CommandRisk {
    /// Safe, read-only commands
    Low,
    /// Commands that modify files/state
    Medium(String),
    /// Dangerous commands requiring confirmation
    High(String),
    /// Blocked commands that cannot be executed
    Blocked(String),
}

impl CommandRisk {
    pub fn requires_confirmation(&self) -> bool {
        matches!(self, CommandRisk::High(_))
    }

    pub fn is_blocked(&self) -> bool {
        matches!(self, CommandRisk::Blocked(_))
    }

    pub fn description(&self) -> String {
        match self {
            CommandRisk::Low => "Low risk - safe to execute".to_string(),
            CommandRisk::Medium(reason) => format!("Medium risk - {}", reason),
            CommandRisk::High(reason) => format!("High risk - {}", reason),
            CommandRisk::Blocked(reason) => format!("BLOCKED - {}", reason),
        }
    }
}

/// Confirmation prompt for dangerous operations
pub struct ConfirmationPrompt;

impl ConfirmationPrompt {
    /// Display confirmation prompt and get user response
    pub fn confirm(message: &str) -> bool {
        use std::io::{self, Write};

        print!("\n⚠️  {} [y/N]: ", message);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
    }

    /// Confirm command execution
    pub fn confirm_command(command: &str, risk: &CommandRisk) -> bool {
        let message = format!(
            "Execute command?\n   Command: {}\n   Risk: {}",
            command,
            risk.description()
        );
        Self::confirm(&message)
    }

    /// Confirm file write
    pub fn confirm_write(path: &str) -> bool {
        Self::confirm(&format!("Write to file: {}?", path))
    }

    /// Confirm file delete
    pub fn confirm_delete(path: &str) -> bool {
        Self::confirm(&format!("DELETE file: {}? This cannot be undone", path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_risk_assessment() {
        let sanitizer = InputSanitizer::with_default();

        // Low risk
        assert!(matches!(
            sanitizer.validate_command("ls -la"),
            Ok(CommandRisk::Low)
        ));

        // Medium risk
        assert!(matches!(
            sanitizer.validate_command("cargo build"),
            Ok(CommandRisk::Medium(_))
        ));

        // High risk - use relative path to avoid matching "rm -rf /"
        assert!(matches!(
            sanitizer.validate_command("sudo rm -rf ./tmp/test"),
            Ok(CommandRisk::High(_))
        ));

        // Blocked
        assert!(sanitizer.validate_command("rm -rf /").is_err());
    }

    #[test]
    fn test_path_validation() {
        let sanitizer = InputSanitizer::with_default();

        // Should block sensitive files
        assert!(sanitizer.validate_path("/etc/passwd").is_err());
        assert!(sanitizer.validate_path("~/.ssh/id_rsa").is_err());
    }
}
