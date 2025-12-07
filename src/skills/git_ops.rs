use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::process::Command;

use crate::config::Settings;
use super::registry::{Skill, SkillDefinition};

fn run_git_command(args: &[&str], cwd: Option<&str>) -> Result<String> {
    let mut cmd = Command::new("git");
    cmd.args(args);
    
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    let output = cmd.output()
        .context("Failed to execute git command")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        if !stderr.is_empty() {
            anyhow::bail!("Git error: {}", stderr.trim());
        }
        anyhow::bail!("Git command failed");
    }

    Ok(stdout.to_string())
}

pub struct GitStatusSkill;

#[async_trait]
impl Skill for GitStatusSkill {
    fn definition(&self) -> SkillDefinition {
        SkillDefinition {
            name: "git_status".to_string(),
            description: "Show the working tree status (modified, staged, untracked files)".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the git repository (defaults to current directory)"
                    }
                },
                "required": []
            }),
            requires_confirmation: false,
        }
    }

    async fn execute(&self, args: &Value, _settings: &Settings) -> Result<String> {
        let path = args["path"].as_str();
        run_git_command(&["status"], path)
    }
}

pub struct GitDiffSkill;

#[async_trait]
impl Skill for GitDiffSkill {
    fn definition(&self) -> SkillDefinition {
        SkillDefinition {
            name: "git_diff".to_string(),
            description: "Show changes between commits, commit and working tree, etc.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the git repository (defaults to current directory)"
                    },
                    "file": {
                        "type": "string",
                        "description": "Specific file to diff (optional)"
                    },
                    "staged": {
                        "type": "boolean",
                        "description": "Show staged changes (--cached)"
                    }
                },
                "required": []
            }),
            requires_confirmation: false,
        }
    }

    async fn execute(&self, args: &Value, _settings: &Settings) -> Result<String> {
        let path = args["path"].as_str();
        let file = args["file"].as_str();
        let staged = args["staged"].as_bool().unwrap_or(false);

        let mut git_args = vec!["diff"];
        if staged {
            git_args.push("--cached");
        }
        if let Some(f) = file {
            git_args.push("--");
            git_args.push(f);
        }

        let result = run_git_command(&git_args, path)?;
        if result.trim().is_empty() {
            Ok("No changes".to_string())
        } else {
            Ok(result)
        }
    }
}

pub struct GitLogSkill;

#[async_trait]
impl Skill for GitLogSkill {
    fn definition(&self) -> SkillDefinition {
        SkillDefinition {
            name: "git_log".to_string(),
            description: "Show commit logs".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the git repository (defaults to current directory)"
                    },
                    "count": {
                        "type": "integer",
                        "description": "Number of commits to show (default: 10)"
                    },
                    "oneline": {
                        "type": "boolean",
                        "description": "Show one line per commit (default: true)"
                    }
                },
                "required": []
            }),
            requires_confirmation: false,
        }
    }

    async fn execute(&self, args: &Value, _settings: &Settings) -> Result<String> {
        let path = args["path"].as_str();
        let count = args["count"].as_u64().unwrap_or(10);
        let oneline = args["oneline"].as_bool().unwrap_or(true);

        let count_str = format!("-{}", count);
        let mut git_args = vec!["log", &count_str];
        
        if oneline {
            git_args.push("--oneline");
        }

        run_git_command(&git_args, path)
    }
}

pub struct GitCommitSkill;

#[async_trait]
impl Skill for GitCommitSkill {
    fn definition(&self) -> SkillDefinition {
        SkillDefinition {
            name: "git_commit".to_string(),
            description: "Create a new commit with staged changes".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the git repository (defaults to current directory)"
                    },
                    "message": {
                        "type": "string",
                        "description": "Commit message"
                    }
                },
                "required": ["message"]
            }),
            requires_confirmation: true,
        }
    }

    async fn execute(&self, args: &Value, _settings: &Settings) -> Result<String> {
        let path = args["path"].as_str();
        let message = args["message"].as_str()
            .context("Commit message is required")?;

        run_git_command(&["commit", "-m", message], path)
    }
}

pub struct GitAddSkill;

#[async_trait]
impl Skill for GitAddSkill {
    fn definition(&self) -> SkillDefinition {
        SkillDefinition {
            name: "git_add".to_string(),
            description: "Add file contents to the staging area".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the git repository (defaults to current directory)"
                    },
                    "files": {
                        "type": "string",
                        "description": "Files to add (space-separated, or '.' for all)"
                    }
                },
                "required": ["files"]
            }),
            requires_confirmation: true,
        }
    }

    async fn execute(&self, args: &Value, _settings: &Settings) -> Result<String> {
        let path = args["path"].as_str();
        let files = args["files"].as_str()
            .context("Files argument is required")?;

        run_git_command(&["add", files], path)?;
        Ok(format!("Added: {}", files))
    }
}

pub struct GitBranchSkill;

#[async_trait]
impl Skill for GitBranchSkill {
    fn definition(&self) -> SkillDefinition {
        SkillDefinition {
            name: "git_branch".to_string(),
            description: "List, create, or switch branches".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the git repository (defaults to current directory)"
                    },
                    "action": {
                        "type": "string",
                        "enum": ["list", "create", "switch", "delete"],
                        "description": "Action to perform (default: list)"
                    },
                    "name": {
                        "type": "string",
                        "description": "Branch name (for create/switch/delete)"
                    }
                },
                "required": []
            }),
            requires_confirmation: false,
        }
    }

    async fn execute(&self, args: &Value, _settings: &Settings) -> Result<String> {
        let path = args["path"].as_str();
        let action = args["action"].as_str().unwrap_or("list");
        let name = args["name"].as_str();

        match action {
            "list" => run_git_command(&["branch", "-a"], path),
            "create" => {
                let name = name.context("Branch name is required for create")?;
                run_git_command(&["branch", name], path)?;
                Ok(format!("Created branch: {}", name))
            }
            "switch" => {
                let name = name.context("Branch name is required for switch")?;
                run_git_command(&["checkout", name], path)?;
                Ok(format!("Switched to branch: {}", name))
            }
            "delete" => {
                let name = name.context("Branch name is required for delete")?;
                run_git_command(&["branch", "-d", name], path)?;
                Ok(format!("Deleted branch: {}", name))
            }
            _ => anyhow::bail!("Unknown action: {}", action),
        }
    }
}

pub struct GitCheckoutSkill;

#[async_trait]
impl Skill for GitCheckoutSkill {
    fn definition(&self) -> SkillDefinition {
        SkillDefinition {
            name: "git_checkout".to_string(),
            description: "Switch branches or restore working tree files".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the git repository (defaults to current directory)"
                    },
                    "target": {
                        "type": "string",
                        "description": "Branch name or file path to checkout"
                    },
                    "create_branch": {
                        "type": "boolean",
                        "description": "Create a new branch (-b flag)"
                    }
                },
                "required": ["target"]
            }),
            requires_confirmation: false,
        }
    }

    async fn execute(&self, args: &Value, _settings: &Settings) -> Result<String> {
        let path = args["path"].as_str();
        let target = args["target"].as_str()
            .context("Target is required")?;
        let create_branch = args["create_branch"].as_bool().unwrap_or(false);

        if create_branch {
            run_git_command(&["checkout", "-b", target], path)
        } else {
            run_git_command(&["checkout", target], path)
        }
    }
}
