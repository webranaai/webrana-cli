// ============================================
// WEBRANA CLI - Plugin System
// Created by: CIPHER (Team Beta)
// ============================================

mod loader;
mod manifest;
mod runtime;

pub use loader::PluginLoader;
pub use manifest::{PluginConfig, PluginManifest};
pub use runtime::{PluginInstance, PluginRuntime};

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Plugin trait that all plugins must implement
pub trait Plugin: Send + Sync {
    /// Get plugin metadata
    fn manifest(&self) -> &PluginManifest;

    /// Initialize the plugin
    fn init(&mut self) -> Result<()>;

    /// Execute the plugin with given input
    fn execute(&self, input: &PluginInput) -> Result<PluginOutput>;

    /// Cleanup when plugin is unloaded
    fn cleanup(&mut self) -> Result<()>;
}

/// Input passed to plugin execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInput {
    pub action: String,
    pub params: serde_json::Value,
    pub context: PluginContext,
}

/// Context available to plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginContext {
    pub working_dir: String,
    pub project_type: Option<String>,
    pub user_config: serde_json::Value,
}

/// Output from plugin execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginOutput {
    pub success: bool,
    pub result: serde_json::Value,
    pub logs: Vec<String>,
    pub artifacts: Vec<PluginArtifact>,
}

/// Artifact produced by plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginArtifact {
    pub name: String,
    pub artifact_type: ArtifactType,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactType {
    File,
    Log,
    Metric,
    Event,
}

impl Default for PluginOutput {
    fn default() -> Self {
        Self {
            success: true,
            result: serde_json::Value::Null,
            logs: Vec::new(),
            artifacts: Vec::new(),
        }
    }
}

impl PluginOutput {
    pub fn success(result: serde_json::Value) -> Self {
        Self {
            success: true,
            result,
            ..Default::default()
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            result: serde_json::json!({ "error": message }),
            ..Default::default()
        }
    }

    pub fn with_log(mut self, log: &str) -> Self {
        self.logs.push(log.to_string());
        self
    }
}
