// ============================================
// Plugin Manifest - CIPHER (Team Beta)
// ============================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Plugin manifest describing plugin metadata and capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Plugin unique identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Version (semver)
    pub version: String,

    /// Description
    pub description: String,

    /// Author information
    pub author: PluginAuthor,

    /// Plugin type
    pub plugin_type: PluginType,

    /// Minimum Webrana version required
    pub min_webrana_version: String,

    /// Permissions required
    pub permissions: Vec<Permission>,

    /// Skills provided by this plugin
    pub skills: Vec<SkillDefinition>,

    /// Configuration schema
    #[serde(default)]
    pub config_schema: Option<serde_json::Value>,

    /// Entry point (for WASM: .wasm file, for native: .so/.dll)
    pub entry_point: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginAuthor {
    pub name: String,
    pub email: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginType {
    /// WebAssembly plugin (sandboxed)
    #[serde(rename = "wasm")]
    Wasm,

    /// Native plugin (trusted, no sandbox)
    #[serde(rename = "native")]
    Native,

    /// Script plugin (interpreted)
    #[serde(rename = "script")]
    Script,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
    /// Read files from filesystem
    #[serde(rename = "fs:read")]
    FileRead,

    /// Write files to filesystem
    #[serde(rename = "fs:write")]
    FileWrite,

    /// Execute shell commands
    #[serde(rename = "shell:execute")]
    ShellExecute,

    /// Make network requests
    #[serde(rename = "net:request")]
    NetworkRequest,

    /// Access environment variables
    #[serde(rename = "env:read")]
    EnvRead,

    /// Access git operations
    #[serde(rename = "git:access")]
    GitAccess,

    /// Access LLM providers
    #[serde(rename = "llm:access")]
    LlmAccess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    /// Skill name (used in tool calls)
    pub name: String,

    /// Description for LLM
    pub description: String,

    /// Input schema (JSON Schema)
    pub input_schema: serde_json::Value,

    /// Whether confirmation is required
    #[serde(default)]
    pub requires_confirmation: bool,
}

/// Plugin configuration (user-provided)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginConfig {
    /// Whether plugin is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Plugin-specific settings
    #[serde(default)]
    pub settings: HashMap<String, serde_json::Value>,
}

fn default_true() -> bool {
    true
}

impl PluginManifest {
    /// Load manifest from YAML file
    pub fn from_yaml(content: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(content)
    }

    /// Load manifest from TOML file
    pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    /// Validate manifest
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Plugin ID cannot be empty".to_string());
        }

        if self.name.is_empty() {
            return Err("Plugin name cannot be empty".to_string());
        }

        if self.version.is_empty() {
            return Err("Plugin version cannot be empty".to_string());
        }

        if self.skills.is_empty() {
            return Err("Plugin must provide at least one skill".to_string());
        }

        // Validate skill names are unique
        let mut skill_names = std::collections::HashSet::new();
        for skill in &self.skills {
            if !skill_names.insert(&skill.name) {
                return Err(format!("Duplicate skill name: {}", skill.name));
            }
        }

        Ok(())
    }

    /// Check if plugin has specific permission
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }
}

// Example manifest YAML:
//
// id: my-plugin
// name: My Plugin
// version: 1.0.0
// description: A sample plugin
// author:
//   name: Developer
//   email: dev@example.com
// plugin_type: wasm
// min_webrana_version: 0.3.0
// permissions:
//   - fs:read
//   - fs:write
// skills:
//   - name: my_skill
//     description: Does something useful
//     input_schema:
//       type: object
//       properties:
//         input:
//           type: string
//       required: [input]
// entry_point: plugin.wasm
