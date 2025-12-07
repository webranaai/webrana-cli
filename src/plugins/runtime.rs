// ============================================
// Plugin Runtime - CIPHER (Team Beta)
// WASM Integration (requires wasmtime feature)
// ============================================

use anyhow::{anyhow, Result};
use std::path::PathBuf;

use super::manifest::{PluginManifest, PluginType};
use super::{PluginContext, PluginInput, PluginOutput};

/// Plugin instance managing the lifecycle of a loaded plugin
pub struct PluginInstance {
    manifest: PluginManifest,
    plugin_dir: PathBuf,
    state: PluginState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PluginState {
    /// Plugin loaded but not initialized
    Loaded,
    /// Plugin initialized and ready
    Ready,
    /// Plugin execution in progress
    Executing,
    /// Plugin encountered an error
    Error(String),
    /// Plugin unloaded
    Unloaded,
}

impl PluginInstance {
    /// Create new plugin instance
    pub fn new(manifest: PluginManifest, plugin_dir: PathBuf) -> Result<Self> {
        Ok(Self {
            manifest,
            plugin_dir,
            state: PluginState::Loaded,
        })
    }

    /// Get plugin manifest
    pub fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    /// Get plugin state
    pub fn state(&self) -> &PluginState {
        &self.state
    }

    /// Initialize the plugin
    pub fn init(&mut self) -> Result<()> {
        match self.manifest.plugin_type {
            PluginType::Wasm => self.init_wasm()?,
            PluginType::Native => self.init_native()?,
            PluginType::Script => self.init_script()?,
        }

        self.state = PluginState::Ready;
        Ok(())
    }

    /// Execute plugin with given input
    pub fn execute(&self, input: &PluginInput) -> Result<PluginOutput> {
        if self.state != PluginState::Ready {
            return Err(anyhow!("Plugin not ready. State: {:?}", self.state));
        }

        // Find the skill
        let _skill = self
            .manifest
            .skills
            .iter()
            .find(|s| s.name == input.action)
            .ok_or_else(|| anyhow!("Skill not found: {}", input.action))?;

        // Check permissions if needed
        self.check_permissions(&input.action)?;

        // Execute based on plugin type
        match self.manifest.plugin_type {
            PluginType::Wasm => self.execute_wasm(input),
            PluginType::Native => self.execute_native(input),
            PluginType::Script => self.execute_script(input),
        }
    }

    /// Cleanup plugin resources
    pub fn cleanup(&mut self) -> Result<()> {
        self.state = PluginState::Unloaded;
        Ok(())
    }

    // ==========================================
    // WASM Plugin Implementation
    // ==========================================

    fn init_wasm(&mut self) -> Result<()> {
        let wasm_path = self.plugin_dir.join(&self.manifest.entry_point);

        if !wasm_path.exists() {
            return Err(anyhow!("WASM file not found: {:?}", wasm_path));
        }

        // WASM runtime disabled - requires Rust 1.80+ and wasmtime feature
        tracing::warn!(
            "WASM plugin '{}' loaded but WASM runtime is disabled. \
             Rebuild with wasmtime feature for full WASM support.",
            self.manifest.id
        );

        Ok(())
    }

    fn execute_wasm(&self, input: &PluginInput) -> Result<PluginOutput> {
        // WASM runtime disabled
        Ok(PluginOutput::error(
            "WASM runtime is disabled. Rebuild with wasmtime feature for WASM plugin support.",
        ))
    }

    // ==========================================
    // Native Plugin Implementation
    // ==========================================

    fn init_native(&mut self) -> Result<()> {
        let lib_path = self.plugin_dir.join(&self.manifest.entry_point);

        if !lib_path.exists() {
            return Err(anyhow!("Native library not found: {:?}", lib_path));
        }

        tracing::warn!("Native plugins are experimental and may be unsafe");
        Ok(())
    }

    fn execute_native(&self, _input: &PluginInput) -> Result<PluginOutput> {
        Err(anyhow!("Native plugin execution not yet implemented"))
    }

    // ==========================================
    // Script Plugin Implementation
    // ==========================================

    fn init_script(&mut self) -> Result<()> {
        let script_path = self.plugin_dir.join(&self.manifest.entry_point);

        if !script_path.exists() {
            return Err(anyhow!("Script file not found: {:?}", script_path));
        }

        std::fs::read_to_string(&script_path)?;
        Ok(())
    }

    fn execute_script(&self, input: &PluginInput) -> Result<PluginOutput> {
        Ok(PluginOutput::success(serde_json::json!({
            "message": format!("Script execution placeholder for action: {}", input.action),
            "plugin": self.manifest.id
        })))
    }

    // ==========================================
    // Permission Checking
    // ==========================================

    fn check_permissions(&self, action: &str) -> Result<()> {
        tracing::debug!(
            "Plugin {} executing action {} with permissions: {:?}",
            self.manifest.id,
            action,
            self.manifest.permissions
        );
        Ok(())
    }
}

/// Plugin runtime managing all plugins
pub struct PluginRuntime {
    loader: super::loader::PluginLoader,
}

impl PluginRuntime {
    pub fn new() -> Self {
        Self {
            loader: super::loader::PluginLoader::new(),
        }
    }

    /// Initialize runtime and discover plugins
    pub fn init(&mut self) -> Result<()> {
        let discovered = self.loader.discover()?;
        tracing::info!("Discovered {} plugins", discovered.len());
        Ok(())
    }

    /// Load a plugin by ID
    pub fn load_plugin(&mut self, plugin_id: &str) -> Result<()> {
        self.loader.load(plugin_id)
    }

    /// Execute a plugin skill
    pub fn execute_skill(
        &self,
        plugin_id: &str,
        skill_name: &str,
        params: serde_json::Value,
    ) -> Result<PluginOutput> {
        let instance = self
            .loader
            .get_instance(plugin_id)
            .ok_or_else(|| anyhow!("Plugin not loaded: {}", plugin_id))?;

        let input = PluginInput {
            action: skill_name.to_string(),
            params,
            context: PluginContext {
                working_dir: std::env::current_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default(),
                project_type: None,
                user_config: serde_json::Value::Null,
            },
        };

        instance.execute(&input)
    }

    /// Get all available skills from loaded plugins
    pub fn get_all_skills(&self) -> Vec<(&str, &super::manifest::SkillDefinition)> {
        self.loader.get_all_skills()
    }
}

impl Default for PluginRuntime {
    fn default() -> Self {
        Self::new()
    }
}
