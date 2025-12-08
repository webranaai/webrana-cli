//! MCP Server Registry
//! 
//! Manages multiple MCP server connections and provides unified tool access.

use super::{McpClient, McpTool, ToolCallResult};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub enabled: bool,
}

/// MCP Registry configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpConfig {
    #[serde(default)]
    pub servers: HashMap<String, McpServerConfig>,
}

/// Registry for managing MCP server connections
pub struct McpRegistry {
    clients: HashMap<String, McpClient>,
    tool_map: HashMap<String, String>, // tool_name -> server_name
}

impl McpRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            tool_map: HashMap::new(),
        }
    }

    /// Load configuration from file
    pub fn load_config(path: &Path) -> Result<McpConfig> {
        let content = std::fs::read_to_string(path)?;
        let config: McpConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Initialize from configuration
    pub fn from_config(config: &McpConfig) -> Result<Self> {
        let mut registry = Self::new();

        for (name, server_config) in &config.servers {
            if !server_config.enabled {
                continue;
            }

            match registry.add_server(name, server_config) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Warning: Failed to connect to MCP server '{}': {}", name, e);
                }
            }
        }

        Ok(registry)
    }

    /// Add a server to the registry
    pub fn add_server(&mut self, name: &str, config: &McpServerConfig) -> Result<()> {
        let args: Vec<&str> = config.args.iter().map(|s| s.as_str()).collect();
        let mut client = McpClient::new_stdio(name, &config.command, &args)?;

        // Initialize and get tools
        client.initialize()?;
        let tools = client.list_tools()?;

        // Map tools to this server
        for tool in &tools {
            self.tool_map.insert(tool.name.clone(), name.to_string());
        }

        self.clients.insert(name.to_string(), client);
        Ok(())
    }

    /// Remove a server from the registry
    pub fn remove_server(&mut self, name: &str) -> Result<()> {
        if let Some(mut client) = self.clients.remove(name) {
            // Remove tool mappings
            self.tool_map.retain(|_, server| server != name);
            client.shutdown()?;
        }
        Ok(())
    }

    /// Get all available tools from all servers
    pub fn list_all_tools(&self) -> Vec<(String, McpTool)> {
        let mut tools = Vec::new();
        for (name, client) in &self.clients {
            for tool in client.tools() {
                tools.push((name.clone(), tool.clone()));
            }
        }
        tools
    }

    /// Get tools from a specific server
    pub fn list_server_tools(&self, server_name: &str) -> Option<&[McpTool]> {
        self.clients.get(server_name).map(|c| c.tools())
    }

    /// Find which server provides a tool
    pub fn find_tool_server(&self, tool_name: &str) -> Option<&str> {
        self.tool_map.get(tool_name).map(|s| s.as_str())
    }

    /// Call a tool (automatically routes to correct server)
    pub fn call_tool(&mut self, tool_name: &str, arguments: HashMap<String, serde_json::Value>) -> Result<ToolCallResult> {
        let server_name = self.tool_map.get(tool_name)
            .ok_or_else(|| anyhow!("Tool '{}' not found", tool_name))?
            .clone();

        let client = self.clients.get_mut(&server_name)
            .ok_or_else(|| anyhow!("Server '{}' not connected", server_name))?;

        client.call_tool(tool_name, arguments)
    }

    /// Get connected server names
    pub fn connected_servers(&self) -> Vec<&str> {
        self.clients.keys().map(|s| s.as_str()).collect()
    }

    /// Get server info
    pub fn server_info(&self, name: &str) -> Option<String> {
        self.clients.get(name).and_then(|c| {
            c.server_info().map(|info| format!("{} v{}", info.name, info.version))
        })
    }

    /// Shutdown all connections
    pub fn shutdown(&mut self) -> Result<()> {
        for (_, mut client) in self.clients.drain() {
            let _ = client.shutdown();
        }
        self.tool_map.clear();
        Ok(())
    }
}

impl Default for McpRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for McpRegistry {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

/// Generate tool descriptions for LLM context
pub fn format_mcp_tools_for_llm(tools: &[(String, McpTool)]) -> String {
    let mut output = String::new();
    output.push_str("## Available MCP Tools\n\n");

    for (server, tool) in tools {
        output.push_str(&format!("### {} (from {})\n", tool.name, server));
        if let Some(desc) = &tool.description {
            output.push_str(&format!("{}\n", desc));
        }
        if let Some(schema) = &tool.input_schema {
            if let Some(props) = schema.get("properties") {
                output.push_str("Parameters:\n");
                if let Some(obj) = props.as_object() {
                    for (name, _) in obj {
                        output.push_str(&format!("  - {}\n", name));
                    }
                }
            }
        }
        output.push('\n');
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new() {
        let registry = McpRegistry::new();
        assert!(registry.connected_servers().is_empty());
    }

    #[test]
    fn test_config_parse() {
        let toml = r#"
[servers.filesystem]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
enabled = true

[servers.github]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-github"]
enabled = false
"#;
        let config: McpConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.servers.len(), 2);
        assert!(config.servers.get("filesystem").unwrap().enabled);
        assert!(!config.servers.get("github").unwrap().enabled);
    }

    #[test]
    fn test_format_tools() {
        let tools = vec![
            ("server1".to_string(), McpTool {
                name: "read_file".to_string(),
                description: Some("Read a file".to_string()),
                input_schema: None,
            }),
        ];
        let output = format_mcp_tools_for_llm(&tools);
        assert!(output.contains("read_file"));
        assert!(output.contains("server1"));
    }
}
