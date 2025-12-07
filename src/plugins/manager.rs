// ============================================
// WEBRANA CLI - Plugin Manager
// Sprint 5.4: Ecosystem & Plugins
// Created by: CIPHER (Team Beta)
// ============================================

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::manifest::{PluginConfig, PluginManifest};

/// Plugin installation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPlugin {
    pub manifest: PluginManifest,
    pub config: PluginConfig,
    pub install_path: PathBuf,
    pub installed_at: u64,
    pub source: PluginSource,
}

/// Where the plugin was installed from
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginSource {
    /// Local file path
    Local(PathBuf),
    /// Remote registry
    Registry { name: String, version: String },
    /// Git repository
    Git { url: String, rev: Option<String> },
}

/// Plugin manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagerConfig {
    /// Directory for installed plugins
    pub plugins_dir: PathBuf,
    /// Registry URLs
    pub registries: Vec<String>,
    /// Auto-update enabled
    pub auto_update: bool,
}

impl Default for ManagerConfig {
    fn default() -> Self {
        let plugins_dir = directories::ProjectDirs::from("dev", "webrana", "webrana-cli")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".webrana"))
            .join("plugins");

        Self {
            plugins_dir,
            registries: vec!["https://plugins.webrana.dev".to_string()],
            auto_update: false,
        }
    }
}

/// Plugin manager for installing, updating, and managing plugins
pub struct PluginManager {
    config: ManagerConfig,
    installed: HashMap<String, InstalledPlugin>,
    state_file: PathBuf,
}

impl PluginManager {
    /// Create new plugin manager
    pub fn new(config: ManagerConfig) -> Result<Self> {
        // Ensure plugins directory exists
        fs::create_dir_all(&config.plugins_dir)?;

        let state_file = config.plugins_dir.join("plugins.json");
        let installed = Self::load_state(&state_file)?;

        Ok(Self {
            config,
            installed,
            state_file,
        })
    }

    /// Create with default config
    pub fn default_manager() -> Result<Self> {
        Self::new(ManagerConfig::default())
    }

    /// Load plugin state from file
    fn load_state(path: &Path) -> Result<HashMap<String, InstalledPlugin>> {
        if !path.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(path)?;
        let state: HashMap<String, InstalledPlugin> = serde_json::from_str(&content)?;
        Ok(state)
    }

    /// Save plugin state to file
    fn save_state(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.installed)?;
        fs::write(&self.state_file, content)?;
        Ok(())
    }

    /// Install plugin from local path
    pub fn install_local(&mut self, path: &Path) -> Result<InstallResult> {
        let manifest_path = path.join("plugin.yaml");
        if !manifest_path.exists() {
            anyhow::bail!("No plugin.yaml found at {}", path.display());
        }

        let manifest_content = fs::read_to_string(&manifest_path)?;
        let manifest = PluginManifest::from_yaml(&manifest_content)
            .context("Failed to parse plugin.yaml")?;

        manifest.validate().map_err(|e| anyhow::anyhow!(e))?;

        // Check if already installed
        if self.installed.contains_key(&manifest.id) {
            return Ok(InstallResult::AlreadyInstalled(manifest.id.clone()));
        }

        // Copy to plugins directory
        let install_dir = self.config.plugins_dir.join(&manifest.id);
        if install_dir.exists() {
            fs::remove_dir_all(&install_dir)?;
        }
        
        self.copy_dir_recursive(path, &install_dir)?;

        // Register plugin
        let installed = InstalledPlugin {
            manifest: manifest.clone(),
            config: PluginConfig::default(),
            install_path: install_dir,
            installed_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            source: PluginSource::Local(path.to_path_buf()),
        };

        self.installed.insert(manifest.id.clone(), installed);
        self.save_state()?;

        Ok(InstallResult::Installed(manifest))
    }

    /// Copy directory recursively
    fn copy_dir_recursive(&self, src: &Path, dst: &Path) -> Result<()> {
        fs::create_dir_all(dst)?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                self.copy_dir_recursive(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }

        Ok(())
    }

    /// Uninstall a plugin
    pub fn uninstall(&mut self, plugin_id: &str) -> Result<bool> {
        if let Some(plugin) = self.installed.remove(plugin_id) {
            // Remove plugin directory
            if plugin.install_path.exists() {
                fs::remove_dir_all(&plugin.install_path)?;
            }
            self.save_state()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Enable a plugin
    pub fn enable(&mut self, plugin_id: &str) -> Result<bool> {
        if let Some(plugin) = self.installed.get_mut(plugin_id) {
            plugin.config.enabled = true;
            self.save_state()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Disable a plugin
    pub fn disable(&mut self, plugin_id: &str) -> Result<bool> {
        if let Some(plugin) = self.installed.get_mut(plugin_id) {
            plugin.config.enabled = false;
            self.save_state()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get list of installed plugins
    pub fn list(&self) -> Vec<&InstalledPlugin> {
        self.installed.values().collect()
    }

    /// Get list of enabled plugins
    pub fn list_enabled(&self) -> Vec<&InstalledPlugin> {
        self.installed
            .values()
            .filter(|p| p.config.enabled)
            .collect()
    }

    /// Get a specific plugin
    pub fn get(&self, plugin_id: &str) -> Option<&InstalledPlugin> {
        self.installed.get(plugin_id)
    }

    /// Check if plugin is installed
    pub fn is_installed(&self, plugin_id: &str) -> bool {
        self.installed.contains_key(plugin_id)
    }

    /// Update plugin config
    pub fn update_config(&mut self, plugin_id: &str, config: PluginConfig) -> Result<bool> {
        if let Some(plugin) = self.installed.get_mut(plugin_id) {
            plugin.config = config;
            self.save_state()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get plugins directory
    pub fn plugins_dir(&self) -> &Path {
        &self.config.plugins_dir
    }

    /// Get summary statistics
    pub fn stats(&self) -> ManagerStats {
        let total = self.installed.len();
        let enabled = self.installed.values().filter(|p| p.config.enabled).count();
        let disabled = total - enabled;

        let by_type: HashMap<String, usize> = self
            .installed
            .values()
            .fold(HashMap::new(), |mut acc, p| {
                let type_name = format!("{:?}", p.manifest.plugin_type);
                *acc.entry(type_name).or_insert(0) += 1;
                acc
            });

        ManagerStats {
            total,
            enabled,
            disabled,
            by_type,
        }
    }
}

/// Result of plugin installation
#[derive(Debug)]
pub enum InstallResult {
    Installed(PluginManifest),
    Updated(PluginManifest),
    AlreadyInstalled(String),
}

/// Manager statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct ManagerStats {
    pub total: usize,
    pub enabled: usize,
    pub disabled: usize,
    pub by_type: HashMap<String, usize>,
}

/// Registry plugin info (from remote registry)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryPlugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub downloads: u64,
    pub rating: Option<f32>,
    pub tags: Vec<String>,
    pub download_url: String,
}

/// Registry client for fetching plugins
pub struct RegistryClient {
    base_url: String,
    client: reqwest::Client,
}

impl RegistryClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Search plugins in registry
    pub async fn search(&self, query: &str) -> Result<Vec<RegistryPlugin>> {
        let url = format!("{}/api/plugins/search?q={}", self.base_url, query);
        
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to registry")?;

        if !response.status().is_success() {
            anyhow::bail!("Registry returned error: {}", response.status());
        }

        let plugins: Vec<RegistryPlugin> = response.json().await?;
        Ok(plugins)
    }

    /// Get plugin info from registry
    pub async fn get_plugin(&self, plugin_id: &str) -> Result<RegistryPlugin> {
        let url = format!("{}/api/plugins/{}", self.base_url, plugin_id);
        
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to registry")?;

        if !response.status().is_success() {
            anyhow::bail!("Plugin not found: {}", plugin_id);
        }

        let plugin: RegistryPlugin = response.json().await?;
        Ok(plugin)
    }

    /// List featured plugins
    pub async fn featured(&self) -> Result<Vec<RegistryPlugin>> {
        let url = format!("{}/api/plugins/featured", self.base_url);
        
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to registry")?;

        let plugins: Vec<RegistryPlugin> = response.json().await?;
        Ok(plugins)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_manager_create() {
        let dir = tempdir().unwrap();
        let config = ManagerConfig {
            plugins_dir: dir.path().to_path_buf(),
            ..Default::default()
        };

        let manager = PluginManager::new(config).unwrap();
        assert!(manager.list().is_empty());
    }

    #[test]
    fn test_manager_stats() {
        let dir = tempdir().unwrap();
        let config = ManagerConfig {
            plugins_dir: dir.path().to_path_buf(),
            ..Default::default()
        };

        let manager = PluginManager::new(config).unwrap();
        let stats = manager.stats();

        assert_eq!(stats.total, 0);
        assert_eq!(stats.enabled, 0);
        assert_eq!(stats.disabled, 0);
    }
}
