// ============================================
// Plugin Loader - CIPHER (Team Beta)
// ============================================

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::manifest::{PluginConfig, PluginManifest};
use super::runtime::PluginInstance;

/// Plugin loader responsible for discovering and loading plugins
pub struct PluginLoader {
    /// Plugin search directories
    plugin_dirs: Vec<PathBuf>,

    /// Loaded plugin manifests
    manifests: HashMap<String, PluginManifest>,

    /// Plugin configurations
    configs: HashMap<String, PluginConfig>,

    /// Active plugin instances
    instances: HashMap<String, PluginInstance>,
}

impl PluginLoader {
    /// Create new plugin loader
    pub fn new() -> Self {
        let mut plugin_dirs = Vec::new();

        // Default plugin directories
        // 1. Project-local plugins
        plugin_dirs.push(PathBuf::from(".webrana/plugins"));

        // 2. User plugins
        if let Some(home) = dirs::home_dir() {
            plugin_dirs.push(home.join(".config/webrana/plugins"));
        }

        // 3. System plugins
        plugin_dirs.push(PathBuf::from("/usr/share/webrana/plugins"));

        Self {
            plugin_dirs,
            manifests: HashMap::new(),
            configs: HashMap::new(),
            instances: HashMap::new(),
        }
    }

    /// Add custom plugin directory
    pub fn add_plugin_dir(&mut self, dir: impl AsRef<Path>) {
        self.plugin_dirs.push(dir.as_ref().to_path_buf());
    }

    /// Discover all available plugins
    pub fn discover(&mut self) -> Result<Vec<String>> {
        let mut discovered = Vec::new();

        for dir in &self.plugin_dirs {
            if !dir.exists() {
                continue;
            }

            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    // Look for manifest in plugin directory
                    if let Some(manifest) = self.load_manifest(&path)? {
                        let id = manifest.id.clone();
                        self.manifests.insert(id.clone(), manifest);
                        discovered.push(id);
                    }
                }
            }
        }

        Ok(discovered)
    }

    /// Load manifest from plugin directory
    fn load_manifest(&self, plugin_dir: &Path) -> Result<Option<PluginManifest>> {
        // Try manifest.yaml first
        let yaml_path = plugin_dir.join("manifest.yaml");
        if yaml_path.exists() {
            let content = fs::read_to_string(&yaml_path)?;
            let manifest = PluginManifest::from_yaml(&content)
                .map_err(|e| anyhow!("Failed to parse manifest.yaml: {}", e))?;
            manifest.validate().map_err(|e| anyhow!(e))?;
            return Ok(Some(manifest));
        }

        // Try manifest.toml
        let toml_path = plugin_dir.join("manifest.toml");
        if toml_path.exists() {
            let content = fs::read_to_string(&toml_path)?;
            let manifest = PluginManifest::from_toml(&content)
                .map_err(|e| anyhow!("Failed to parse manifest.toml: {}", e))?;
            manifest.validate().map_err(|e| anyhow!(e))?;
            return Ok(Some(manifest));
        }

        Ok(None)
    }

    /// Get manifest for a plugin
    pub fn get_manifest(&self, plugin_id: &str) -> Option<&PluginManifest> {
        self.manifests.get(plugin_id)
    }

    /// List all discovered plugins
    pub fn list_plugins(&self) -> Vec<&PluginManifest> {
        self.manifests.values().collect()
    }

    /// Load and initialize a plugin
    pub fn load(&mut self, plugin_id: &str) -> Result<()> {
        let manifest = self
            .manifests
            .get(plugin_id)
            .ok_or_else(|| anyhow!("Plugin not found: {}", plugin_id))?
            .clone();

        // Find plugin directory
        let plugin_dir = self.find_plugin_dir(plugin_id)?;

        // Create plugin instance
        let mut instance = PluginInstance::new(manifest, plugin_dir)?;

        // Initialize plugin
        instance.init()?;

        // Store instance
        self.instances.insert(plugin_id.to_string(), instance);

        Ok(())
    }

    /// Unload a plugin
    pub fn unload(&mut self, plugin_id: &str) -> Result<()> {
        if let Some(mut instance) = self.instances.remove(plugin_id) {
            instance.cleanup()?;
        }
        Ok(())
    }

    /// Get active plugin instance
    pub fn get_instance(&self, plugin_id: &str) -> Option<&PluginInstance> {
        self.instances.get(plugin_id)
    }

    /// Find plugin directory by ID
    fn find_plugin_dir(&self, plugin_id: &str) -> Result<PathBuf> {
        for dir in &self.plugin_dirs {
            let plugin_dir = dir.join(plugin_id);
            if plugin_dir.exists() {
                return Ok(plugin_dir);
            }
        }
        Err(anyhow!("Plugin directory not found for: {}", plugin_id))
    }

    /// Load configuration for a plugin
    pub fn set_config(&mut self, plugin_id: &str, config: PluginConfig) {
        self.configs.insert(plugin_id.to_string(), config);
    }

    /// Get configuration for a plugin
    pub fn get_config(&self, plugin_id: &str) -> Option<&PluginConfig> {
        self.configs.get(plugin_id)
    }

    /// Check if plugin is loaded
    pub fn is_loaded(&self, plugin_id: &str) -> bool {
        self.instances.contains_key(plugin_id)
    }

    /// Get all skill definitions from loaded plugins
    pub fn get_all_skills(&self) -> Vec<(&str, &super::manifest::SkillDefinition)> {
        let mut skills = Vec::new();

        for (plugin_id, instance) in &self.instances {
            for skill in &instance.manifest().skills {
                skills.push((plugin_id.as_str(), skill));
            }
        }

        skills
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

// Add dirs crate dependency for home_dir
// This is a placeholder - actual implementation would use the dirs crate
mod dirs {
    use std::path::PathBuf;

    pub fn home_dir() -> Option<PathBuf> {
        std::env::var("HOME").ok().map(PathBuf::from)
    }
}
