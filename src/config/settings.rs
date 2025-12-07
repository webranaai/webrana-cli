use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub models: HashMap<String, ModelConfig>,

    #[serde(default)]
    pub agents: HashMap<String, AgentConfig>,

    #[serde(default)]
    pub default_model: String,

    #[serde(default)]
    pub default_agent: String,

    #[serde(default)]
    pub safety: SafetyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub provider: String,
    pub api_key: Option<String>,
    pub api_key_env: Option<String>,
    pub base_url: Option<String>,
    pub model: String,

    #[serde(default = "default_temperature")]
    pub temperature: f32,

    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub description: String,
    pub system_prompt: String,
    pub model: String,
    pub skills: Vec<String>,

    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SafetyConfig {
    #[serde(default = "default_true")]
    pub confirm_file_write: bool,

    #[serde(default = "default_true")]
    pub confirm_file_delete: bool,

    #[serde(default = "default_true")]
    pub confirm_shell_execute: bool,

    #[serde(default)]
    pub allowed_commands: Vec<String>,

    #[serde(default)]
    pub blocked_paths: Vec<String>,
}

fn default_temperature() -> f32 {
    0.7
}
fn default_max_tokens() -> u32 {
    4096
}
fn default_true() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        let mut models = HashMap::new();
        models.insert(
            "claude".to_string(),
            ModelConfig {
                provider: "anthropic".to_string(),
                api_key: None,
                api_key_env: Some("ANTHROPIC_API_KEY".to_string()),
                base_url: None,
                model: "claude-sonnet-4-20250514".to_string(),
                temperature: 0.7,
                max_tokens: 4096,
            },
        );
        models.insert(
            "gpt".to_string(),
            ModelConfig {
                provider: "openai".to_string(),
                api_key: None,
                api_key_env: Some("OPENAI_API_KEY".to_string()),
                base_url: None,
                model: "gpt-4o".to_string(),
                temperature: 0.7,
                max_tokens: 4096,
            },
        );
        models.insert(
            "ollama".to_string(),
            ModelConfig {
                provider: "ollama".to_string(),
                api_key: None,
                api_key_env: None,
                base_url: Some("http://localhost:11434".to_string()),
                model: "llama3".to_string(),
                temperature: 0.7,
                max_tokens: 4096,
            },
        );

        let mut agents = HashMap::new();
        agents.insert(
            "nexus".to_string(),
            AgentConfig {
                name: "NEXUS".to_string(),
                description: "Orchestrator - Task decomposition and routing".to_string(),
                system_prompt: include_str!("../../agents/nexus.txt").to_string(),
                model: "claude".to_string(),
                skills: vec!["*".to_string()],
                temperature: 0.3,
            },
        );

        Self {
            models,
            agents,
            default_model: "claude".to_string(),
            default_agent: "nexus".to_string(),
            safety: SafetyConfig::default(),
        }
    }
}

impl Settings {
    pub fn load() -> Result<Self> {
        dotenvy::dotenv().ok();

        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path).context("Failed to read config file")?;
            let settings: Settings =
                toml::from_str(&content).context("Failed to parse config file")?;
            Ok(settings)
        } else {
            let settings = Settings::default();
            settings.save()?;
            Ok(settings)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }

    pub fn config_path() -> Result<PathBuf> {
        let dirs = directories::ProjectDirs::from("ai", "webrana", "webrana")
            .context("Could not determine config directory")?;
        Ok(dirs.config_dir().join("config.toml"))
    }

    pub fn get_model(&self, name: &str) -> Option<&ModelConfig> {
        self.models.get(name)
    }

    pub fn get_agent(&self, name: &str) -> Option<&AgentConfig> {
        self.agents.get(name)
    }

    pub fn get_api_key(&self, model_config: &ModelConfig) -> Option<String> {
        if let Some(key) = &model_config.api_key {
            return Some(key.clone());
        }
        if let Some(env_var) = &model_config.api_key_env {
            return std::env::var(env_var).ok();
        }
        None
    }
}
