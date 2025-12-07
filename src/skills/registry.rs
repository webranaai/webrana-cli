use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use super::file_ops::*;
use super::git_ops::{
    GitAddSkill, GitBranchSkill, GitCheckoutSkill, GitCommitSkill, GitDiffSkill, GitLogSkill,
    GitStatusSkill,
};
use super::shell::*;
use crate::config::Settings;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
    pub requires_confirmation: bool,
}

#[async_trait]
pub trait Skill: Send + Sync {
    fn definition(&self) -> SkillDefinition;
    async fn execute(&self, args: &Value, settings: &Settings) -> Result<String>;
}

pub struct SkillRegistry {
    skills: HashMap<String, Box<dyn Skill>>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        let mut skills: HashMap<String, Box<dyn Skill>> = HashMap::new();

        // File operations (with SENTINEL security integration)
        skills.insert("read_file".to_string(), Box::new(ReadFileSkill::new()));
        skills.insert("write_file".to_string(), Box::new(WriteFileSkill::new()));
        skills.insert("list_files".to_string(), Box::new(ListFilesSkill));
        skills.insert("search_files".to_string(), Box::new(SearchFilesSkill));

        // Shell (with SENTINEL security integration)
        skills.insert(
            "execute_command".to_string(),
            Box::new(ExecuteCommandSkill::new()),
        );

        // Git operations
        skills.insert("git_status".to_string(), Box::new(GitStatusSkill));
        skills.insert("git_diff".to_string(), Box::new(GitDiffSkill));
        skills.insert("git_log".to_string(), Box::new(GitLogSkill));
        skills.insert("git_commit".to_string(), Box::new(GitCommitSkill));
        skills.insert("git_add".to_string(), Box::new(GitAddSkill));
        skills.insert("git_branch".to_string(), Box::new(GitBranchSkill));
        skills.insert("git_checkout".to_string(), Box::new(GitCheckoutSkill));

        // Edit operations
        skills.insert("edit_file".to_string(), Box::new(EditFileSkillWrapper));

        // Codebase operations
        skills.insert("grep_codebase".to_string(), Box::new(GrepCodebaseSkill));
        skills.insert("list_symbols".to_string(), Box::new(ListSymbolsSkill));
        skills.insert(
            "get_project_info".to_string(),
            Box::new(GetProjectInfoSkill),
        );

        Self { skills }
    }

    pub fn register(&mut self, skill: Box<dyn Skill>) {
        let def = skill.definition();
        self.skills.insert(def.name, skill);
    }

    pub fn get(&self, name: &str) -> Option<&Box<dyn Skill>> {
        self.skills.get(name)
    }

    pub fn list(&self) -> Vec<SkillDefinition> {
        self.skills.values().map(|s| s.definition()).collect()
    }

    pub async fn execute(&self, name: &str, args: &Value, settings: &Settings) -> Result<String> {
        let skill = self
            .skills
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Skill not found: {}", name))?;

        skill.execute(args, settings).await
    }

    pub fn to_tool_definitions(&self) -> Vec<Value> {
        self.skills
            .values()
            .map(|skill| {
                let def = skill.definition();
                serde_json::json!({
                    "name": def.name,
                    "description": def.description,
                    "input_schema": def.parameters
                })
            })
            .collect()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Edit File Skill Wrapper
pub struct EditFileSkillWrapper;

#[async_trait]
impl Skill for EditFileSkillWrapper {
    fn definition(&self) -> SkillDefinition {
        SkillDefinition {
            name: "edit_file".to_string(),
            description:
                "Edit a file by searching and replacing text. Use search/replace for precise edits."
                    .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to edit"
                    },
                    "search": {
                        "type": "string",
                        "description": "Text to search for (must match exactly)"
                    },
                    "replace": {
                        "type": "string",
                        "description": "Text to replace with"
                    }
                },
                "required": ["path", "search", "replace"]
            }),
            requires_confirmation: true,
        }
    }

    async fn execute(&self, args: &Value, _settings: &Settings) -> Result<String> {
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
        let search = args
            .get("search")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing search"))?;
        let replace = args
            .get("replace")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing replace"))?;

        let skill = super::edit_file::EditFileSkill::new();
        let result = skill.edit_file(path, search, replace)?;

        Ok(serde_json::to_string_pretty(&result)?)
    }
}

// Grep Codebase Skill
pub struct GrepCodebaseSkill;

#[async_trait]
impl Skill for GrepCodebaseSkill {
    fn definition(&self) -> SkillDefinition {
        SkillDefinition {
            name: "grep_codebase".to_string(),
            description: "Search for a pattern across all code files in the project".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Text pattern to search for"
                    },
                    "path": {
                        "type": "string",
                        "description": "Directory to search in (defaults to current dir)"
                    }
                },
                "required": ["pattern"]
            }),
            requires_confirmation: false,
        }
    }

    async fn execute(&self, args: &Value, _settings: &Settings) -> Result<String> {
        let pattern = args
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing pattern"))?;
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");

        let skill = super::codebase::CodebaseSkill::new(path);
        let results = skill.grep(pattern)?;

        if results.is_empty() {
            return Ok("No matches found".to_string());
        }

        let output: Vec<String> = results
            .iter()
            .take(50)
            .map(|r| format!("{}:{}: {}", r.file, r.line_number, r.content.trim()))
            .collect();

        Ok(output.join("\n"))
    }
}

// List Symbols Skill
pub struct ListSymbolsSkill;

#[async_trait]
impl Skill for ListSymbolsSkill {
    fn definition(&self) -> SkillDefinition {
        SkillDefinition {
            name: "list_symbols".to_string(),
            description: "List functions, classes, and other symbols in a source file".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the source file"
                    }
                },
                "required": ["path"]
            }),
            requires_confirmation: false,
        }
    }

    async fn execute(&self, args: &Value, _settings: &Settings) -> Result<String> {
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing path"))?;

        let current_dir = std::env::current_dir()?;
        let skill = super::codebase::CodebaseSkill::new(&current_dir);
        let symbols = skill.list_symbols(path)?;

        if symbols.is_empty() {
            return Ok("No symbols found".to_string());
        }

        let output: Vec<String> = symbols
            .iter()
            .map(|s| format!("{}:{} {} {}", path, s.line, s.kind.as_str(), s.name))
            .collect();

        Ok(output.join("\n"))
    }
}

// Get Project Info Skill
pub struct GetProjectInfoSkill;

#[async_trait]
impl Skill for GetProjectInfoSkill {
    fn definition(&self) -> SkillDefinition {
        SkillDefinition {
            name: "get_project_info".to_string(),
            description: "Get information about the current project (type, dependencies, etc.)"
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Project directory path (defaults to current dir)"
                    }
                }
            }),
            requires_confirmation: false,
        }
    }

    async fn execute(&self, args: &Value, _settings: &Settings) -> Result<String> {
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");

        let mut skill = super::codebase::CodebaseSkill::new(path);
        let info = skill.detect_project()?;

        Ok(info.to_string())
    }
}
