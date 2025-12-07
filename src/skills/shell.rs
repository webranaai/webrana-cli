// ============================================
// Shell Command Execution Skill
// Security Integration by: SENTINEL (Team Beta)
// ============================================

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::process::Command;

use crate::config::Settings;
use crate::core::{InputSanitizer, CommandRisk, SecurityConfig};
use super::registry::{Skill, SkillDefinition};

pub struct ExecuteCommandSkill {
    sanitizer: InputSanitizer,
}

impl ExecuteCommandSkill {
    pub fn new() -> Self {
        Self {
            sanitizer: InputSanitizer::with_default(),
        }
    }

    pub fn with_config(config: SecurityConfig) -> Self {
        Self {
            sanitizer: InputSanitizer::new(config),
        }
    }
}

impl Default for ExecuteCommandSkill {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for ExecuteCommandSkill {
    fn definition(&self) -> SkillDefinition {
        SkillDefinition {
            name: "execute_command".to_string(),
            description: "Execute a shell command with security validation".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The command to execute"
                    },
                    "working_dir": {
                        "type": "string",
                        "description": "Working directory for the command"
                    }
                },
                "required": ["command"]
            }),
            requires_confirmation: true,
        }
    }

    async fn execute(&self, args: &Value, settings: &Settings) -> Result<String> {
        let command = args["command"].as_str()
            .context("Missing 'command' argument")?;
        let working_dir = args["working_dir"].as_str();

        // SENTINEL Security Layer 1: Validate command against allowed list
        if !settings.safety.allowed_commands.is_empty() {
            let cmd_name = command.split_whitespace().next().unwrap_or("");
            if !settings.safety.allowed_commands.contains(&cmd_name.to_string()) {
                anyhow::bail!(
                    "🛡️ SECURITY: Command '{}' not in allowed list. Allowed: {:?}",
                    cmd_name,
                    settings.safety.allowed_commands
                );
            }
        }

        // SENTINEL Security Layer 2: Comprehensive command risk assessment
        let risk = self.sanitizer.validate_command(command)?;
        
        match &risk {
            CommandRisk::Blocked(reason) => {
                anyhow::bail!("🛡️ BLOCKED: {}", reason);
            }
            CommandRisk::High(reason) => {
                tracing::warn!("⚠️ High-risk command: {} - {}", command, reason);
                // In auto mode, high-risk commands should be blocked or require confirmation
                // The orchestrator handles confirmation via requires_confirmation flag
            }
            CommandRisk::Medium(reason) => {
                tracing::info!("📝 Medium-risk command: {} - {}", command, reason);
            }
            CommandRisk::Low => {
                tracing::debug!("✅ Low-risk command: {}", command);
            }
        }

        // Execute the command
        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.args(["/C", command]);
            c
        } else {
            let mut c = Command::new("sh");
            c.args(["-c", command]);
            c
        };

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        let output = cmd.output()
            .context("Failed to execute command")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // SENTINEL Security Layer 3: Sanitize output to remove secrets
        let sanitized_stdout = self.sanitizer.sanitize_output(&stdout);
        let sanitized_stderr = self.sanitizer.sanitize_output(&stderr);

        let mut result = String::new();
        
        // Add risk level indicator
        result.push_str(&format!("[Risk: {}]\n", risk.description()));
        
        if !sanitized_stdout.is_empty() {
            result.push_str(&sanitized_stdout);
        }
        
        if !sanitized_stderr.is_empty() {
            if !result.is_empty() {
                result.push_str("\n--- stderr ---\n");
            }
            result.push_str(&sanitized_stderr);
        }

        if !output.status.success() {
            result.push_str(&format!("\n[Exit code: {}]", output.status.code().unwrap_or(-1)));
        }

        Ok(result)
    }
}
