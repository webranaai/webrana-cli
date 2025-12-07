// ============================================
// Plugin Runtime - CIPHER (Team Beta)
// WASM Integration with wasmtime
// ============================================

use anyhow::{Result, anyhow};
use std::path::PathBuf;
use wasmtime::{Engine, Module, Store, Linker};

use super::manifest::{PluginManifest, PluginType};
use super::{PluginInput, PluginOutput, PluginContext};

/// Default memory limit for WASM plugins (64 MB)
const DEFAULT_MEMORY_LIMIT: usize = 64 * 1024 * 1024;

/// WASM plugin state containing compiled module
pub struct WasmPluginState {
    /// WASM engine
    engine: Engine,
    /// Compiled WASM module
    module: Module,
}

impl WasmPluginState {
    /// Create new WASM plugin state from file
    /// Supports both .wasm (binary) and .wat (text) formats
    pub fn from_file(wasm_path: &PathBuf) -> Result<Self> {
        let engine = Engine::default();
        
        // Check file extension to determine format
        let extension = wasm_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        
        let module = match extension {
            "wat" => {
                // Read WAT text and compile
                let wat_text = std::fs::read_to_string(wasm_path)
                    .map_err(|e| anyhow!("Failed to read WAT file: {}", e))?;
                Module::new(&engine, &wat_text)
                    .map_err(|e| anyhow!("Failed to compile WAT module: {}", e))?
            }
            "wasm" | _ => {
                // Load binary WASM
                Module::from_file(&engine, wasm_path)
                    .map_err(|e| anyhow!("Failed to load WASM module: {}", e))?
            }
        };
        
        Ok(Self { engine, module })
    }

    /// Execute WASM function with input
    pub fn execute(&self, func_name: &str, input: &str) -> Result<String> {
        let mut store = Store::new(&self.engine, ());
        let linker = Linker::new(&self.engine);
        
        let instance = linker.instantiate(&mut store, &self.module)
            .map_err(|e| anyhow!("Failed to instantiate WASM module: {}", e))?;

        // Try to get memory export for data passing
        let memory = instance.get_memory(&mut store, "memory");
        
        // Try to call the function
        // For simple plugins, we'll use a convention where:
        // - Input is passed via exported "alloc" + memory write
        // - Output is read from memory after function call
        
        // First, try simple function without parameters
        if let Some(func) = instance.get_func(&mut store, func_name) {
            // Call the function
            let mut results = vec![wasmtime::Val::I32(0)];
            func.call(&mut store, &[], &mut results)
                .map_err(|e| anyhow!("WASM function call failed: {}", e))?;
            
            // Return result as string
            if let Some(wasmtime::Val::I32(result)) = results.first() {
                return Ok(format!("{{ \"result\": {} }}", result));
            }
        }

        // If no matching function, return info about available exports
        let exports: Vec<String> = self.module.exports()
            .map(|e| e.name().to_string())
            .collect();
        
        Ok(serde_json::json!({
            "status": "executed",
            "available_exports": exports,
            "message": format!("Function '{}' not found or incompatible signature", func_name)
        }).to_string())
    }

    /// Get list of exported functions
    pub fn list_exports(&self) -> Vec<String> {
        self.module.exports()
            .map(|e| e.name().to_string())
            .collect()
    }
}

/// Plugin instance managing the lifecycle of a loaded plugin
pub struct PluginInstance {
    manifest: PluginManifest,
    plugin_dir: PathBuf,
    state: PluginState,
    /// WASM state (if WASM plugin)
    wasm_state: Option<WasmPluginState>,
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
            wasm_state: None,
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
        let skill = self.manifest.skills.iter()
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
        match self.manifest.plugin_type {
            PluginType::Wasm => self.cleanup_wasm()?,
            PluginType::Native => self.cleanup_native()?,
            PluginType::Script => self.cleanup_script()?,
        }
        
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

        // Compile and load the WASM module
        let wasm_state = WasmPluginState::from_file(&wasm_path)?;
        
        // Log available exports
        let exports = wasm_state.list_exports();
        tracing::info!(
            "WASM plugin '{}' loaded with {} exports: {:?}",
            self.manifest.id,
            exports.len(),
            exports
        );
        
        self.wasm_state = Some(wasm_state);
        Ok(())
    }

    fn execute_wasm(&self, input: &PluginInput) -> Result<PluginOutput> {
        let wasm_state = self.wasm_state.as_ref()
            .ok_or_else(|| anyhow!("WASM plugin not initialized"))?;

        // Determine which function to call based on the action
        // Convention: action name maps to exported function
        let func_name = &input.action;
        let input_json = serde_json::to_string(&input.params)?;
        
        // Execute the WASM function
        match wasm_state.execute(func_name, &input_json) {
            Ok(result) => {
                tracing::debug!(
                    "WASM plugin '{}' executed '{}' successfully",
                    self.manifest.id,
                    func_name
                );
                
                // Parse result as JSON if possible
                let result_value: serde_json::Value = serde_json::from_str(&result)
                    .unwrap_or_else(|_| serde_json::json!({ "raw": result }));
                
                Ok(PluginOutput::success(result_value)
                    .with_log(&format!("Executed WASM function: {}", func_name)))
            }
            Err(e) => {
                tracing::error!(
                    "WASM plugin '{}' execution failed: {}",
                    self.manifest.id,
                    e
                );
                Ok(PluginOutput::error(&e.to_string()))
            }
        }
    }

    fn cleanup_wasm(&mut self) -> Result<()> {
        // Drop the WASM state
        self.wasm_state = None;
        tracing::info!("WASM plugin '{}' unloaded", self.manifest.id);
        Ok(())
    }

    // ==========================================
    // Native Plugin Implementation
    // ==========================================

    fn init_native(&mut self) -> Result<()> {
        // Native plugins are loaded as shared libraries
        // This requires careful security consideration
        
        let lib_path = self.plugin_dir.join(&self.manifest.entry_point);
        
        if !lib_path.exists() {
            return Err(anyhow!("Native library not found: {:?}", lib_path));
        }

        // TODO: Load shared library
        // This is a security risk and should be carefully sandboxed
        // unsafe {
        //     let lib = libloading::Library::new(&lib_path)?;
        //     // Get function pointers
        // }

        tracing::warn!("Native plugins are experimental and may be unsafe");
        
        Ok(())
    }

    fn execute_native(&self, input: &PluginInput) -> Result<PluginOutput> {
        // TODO: Call native function
        Err(anyhow!("Native plugin execution not yet implemented"))
    }

    fn cleanup_native(&mut self) -> Result<()> {
        // TODO: Unload shared library
        Ok(())
    }

    // ==========================================
    // Script Plugin Implementation
    // ==========================================

    fn init_script(&mut self) -> Result<()> {
        let script_path = self.plugin_dir.join(&self.manifest.entry_point);
        
        if !script_path.exists() {
            return Err(anyhow!("Script file not found: {:?}", script_path));
        }

        // Validate script exists and is readable
        std::fs::read_to_string(&script_path)?;
        
        Ok(())
    }

    fn execute_script(&self, input: &PluginInput) -> Result<PluginOutput> {
        // TODO: Execute script via subprocess
        // This could use deno, node, python, etc.
        
        Ok(PluginOutput::success(serde_json::json!({
            "message": format!("Script execution placeholder for action: {}", input.action),
            "plugin": self.manifest.id
        })))
    }

    fn cleanup_script(&mut self) -> Result<()> {
        Ok(())
    }

    // ==========================================
    // Permission Checking
    // ==========================================

    fn check_permissions(&self, action: &str) -> Result<()> {
        // TODO: Implement permission checking based on action requirements
        // For now, just log
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
    pub fn execute_skill(&self, plugin_id: &str, skill_name: &str, params: serde_json::Value) -> Result<PluginOutput> {
        let instance = self.loader.get_instance(plugin_id)
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
