// ============================================
// File Operations Skills
// Security Integration by: SENTINEL (Team Beta)
// ============================================

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

use crate::config::Settings;
use crate::core::{InputSanitizer, SecurityConfig};
use super::registry::{Skill, SkillDefinition};

pub struct ReadFileSkill {
    sanitizer: InputSanitizer,
}

impl ReadFileSkill {
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

impl Default for ReadFileSkill {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for ReadFileSkill {
    fn definition(&self) -> SkillDefinition {
        SkillDefinition {
            name: "read_file".to_string(),
            description: "Read the contents of a file with security validation".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to read"
                    }
                },
                "required": ["path"]
            }),
            requires_confirmation: false,
        }
    }

    async fn execute(&self, args: &Value, _settings: &Settings) -> Result<String> {
        let path = args["path"].as_str()
            .context("Missing 'path' argument")?;

        // SENTINEL Security: Validate path against sensitive files
        if self.sanitizer.is_sensitive_file(path) {
            anyhow::bail!("🛡️ SECURITY: Access denied to sensitive file: {}", path);
        }

        let content = fs::read_to_string(path)
            .context(format!("Failed to read file: {}", path))?;

        // SENTINEL Security: Sanitize output to remove any secrets
        let sanitized = self.sanitizer.sanitize_output(&content);

        Ok(sanitized)
    }
}

pub struct WriteFileSkill {
    sanitizer: InputSanitizer,
}

impl WriteFileSkill {
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

impl Default for WriteFileSkill {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for WriteFileSkill {
    fn definition(&self) -> SkillDefinition {
        SkillDefinition {
            name: "write_file".to_string(),
            description: "Write content to a file with security validation".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to write"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to write to the file"
                    }
                },
                "required": ["path", "content"]
            }),
            requires_confirmation: true,
        }
    }

    async fn execute(&self, args: &Value, settings: &Settings) -> Result<String> {
        let path = args["path"].as_str()
            .context("Missing 'path' argument")?;
        let content = args["content"].as_str()
            .context("Missing 'content' argument")?;

        // SENTINEL Security Layer 1: Check if path is blocked by settings
        for blocked in &settings.safety.blocked_paths {
            if path.starts_with(blocked) {
                anyhow::bail!("🛡️ SECURITY: Path blocked by safety rules: {}", path);
            }
        }

        // SENTINEL Security Layer 2: Validate path against sensitive files
        if self.sanitizer.is_sensitive_file(path) {
            anyhow::bail!("🛡️ SECURITY: Cannot write to sensitive file: {}", path);
        }

        // SENTINEL Security Layer 3: Validate path is within working directory
        match self.sanitizer.validate_path(path) {
            Ok(validated_path) => {
                // Create parent directories if needed
                if let Some(parent) = validated_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                fs::write(&validated_path, content)
                    .context(format!("Failed to write file: {}", path))?;

                tracing::info!("📝 File written: {} ({} bytes)", path, content.len());
                Ok(format!("✅ Successfully wrote {} bytes to {}", content.len(), path))
            }
            Err(e) => {
                anyhow::bail!("🛡️ SECURITY: Path validation failed - {}", e);
            }
        }
    }
}

pub struct ListFilesSkill;

#[async_trait]
impl Skill for ListFilesSkill {
    fn definition(&self) -> SkillDefinition {
        SkillDefinition {
            name: "list_files".to_string(),
            description: "List files in a directory".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the directory"
                    },
                    "recursive": {
                        "type": "boolean",
                        "description": "Whether to list recursively"
                    }
                },
                "required": ["path"]
            }),
            requires_confirmation: false,
        }
    }

    async fn execute(&self, args: &Value, _settings: &Settings) -> Result<String> {
        let path = args["path"].as_str()
            .context("Missing 'path' argument")?;
        let recursive = args["recursive"].as_bool().unwrap_or(false);

        let mut files = Vec::new();
        collect_files(Path::new(path), recursive, &mut files)?;

        Ok(files.join("\n"))
    }
}

fn collect_files(path: &Path, recursive: bool, files: &mut Vec<String>) -> Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            
            if entry_path.is_dir() {
                files.push(format!("{}/", entry_path.display()));
                if recursive {
                    collect_files(&entry_path, recursive, files)?;
                }
            } else {
                files.push(entry_path.display().to_string());
            }
        }
    }
    Ok(())
}

pub struct SearchFilesSkill;

#[async_trait]
impl Skill for SearchFilesSkill {
    fn definition(&self) -> SkillDefinition {
        SkillDefinition {
            name: "search_files".to_string(),
            description: "Search for text in files".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Directory to search in"
                    },
                    "pattern": {
                        "type": "string",
                        "description": "Text pattern to search for"
                    }
                },
                "required": ["path", "pattern"]
            }),
            requires_confirmation: false,
        }
    }

    async fn execute(&self, args: &Value, _settings: &Settings) -> Result<String> {
        let path = args["path"].as_str()
            .context("Missing 'path' argument")?;
        let pattern = args["pattern"].as_str()
            .context("Missing 'pattern' argument")?;

        let mut results = Vec::new();
        search_in_dir(Path::new(path), pattern, &mut results)?;

        if results.is_empty() {
            Ok("No matches found".to_string())
        } else {
            Ok(results.join("\n"))
        }
    }
}

fn search_in_dir(path: &Path, pattern: &str, results: &mut Vec<String>) -> Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            
            if entry_path.is_dir() {
                search_in_dir(&entry_path, pattern, results)?;
            } else if entry_path.is_file() {
                if let Ok(content) = fs::read_to_string(&entry_path) {
                    for (line_num, line) in content.lines().enumerate() {
                        if line.contains(pattern) {
                            results.push(format!(
                                "{}:{}: {}",
                                entry_path.display(),
                                line_num + 1,
                                line.trim()
                            ));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
