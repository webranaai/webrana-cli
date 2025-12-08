//! MCP Client implementation

use super::protocol::*;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// MCP Client for connecting to MCP servers
pub struct McpClient {
    name: String,
    transport: Transport,
    request_id: AtomicU64,
    server_info: Option<ServerInfo>,
    capabilities: Option<ServerCapabilities>,
    tools: Vec<McpTool>,
}

enum Transport {
    Stdio(StdioTransport),
    #[allow(dead_code)]
    Http(HttpTransport),
}

struct StdioTransport {
    process: Arc<Mutex<Child>>,
}

struct HttpTransport {
    #[allow(dead_code)]
    url: String,
}

impl McpClient {
    /// Create a new MCP client connecting to a server via stdio
    pub fn new_stdio(name: &str, command: &str, args: &[&str]) -> Result<Self> {
        let process = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| anyhow!("Failed to spawn MCP server: {}", e))?;

        Ok(Self {
            name: name.to_string(),
            transport: Transport::Stdio(StdioTransport {
                process: Arc::new(Mutex::new(process)),
            }),
            request_id: AtomicU64::new(1),
            server_info: None,
            capabilities: None,
            tools: Vec::new(),
        })
    }

    /// Create a new MCP client connecting via HTTP
    #[allow(dead_code)]
    pub fn new_http(name: &str, url: &str) -> Result<Self> {
        Ok(Self {
            name: name.to_string(),
            transport: Transport::Http(HttpTransport {
                url: url.to_string(),
            }),
            request_id: AtomicU64::new(1),
            server_info: None,
            capabilities: None,
            tools: Vec::new(),
        })
    }

    /// Get the client name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get server info
    pub fn server_info(&self) -> Option<&ServerInfo> {
        self.server_info.as_ref()
    }

    /// Get available tools
    pub fn tools(&self) -> &[McpTool] {
        &self.tools
    }

    /// Initialize the connection
    pub fn initialize(&mut self) -> Result<InitializeResult> {
        let params = InitializeParams {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ClientCapabilities {
                roots: Some(RootsCapability { list_changed: true }),
                sampling: None,
            },
            client_info: ClientInfo {
                name: "webrana-cli".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        let response: InitializeResult = self.send_request("initialize", Some(serde_json::to_value(params)?))?;
        
        self.server_info = Some(response.server_info.clone());
        self.capabilities = Some(response.capabilities.clone());

        // Send initialized notification
        self.send_notification("notifications/initialized", None)?;

        Ok(response)
    }

    /// List available tools from the server
    pub fn list_tools(&mut self) -> Result<Vec<McpTool>> {
        let response: ListToolsResult = self.send_request("tools/list", None)?;
        self.tools = response.tools.clone();
        Ok(response.tools)
    }

    /// Call a tool
    pub fn call_tool(&mut self, name: &str, arguments: HashMap<String, serde_json::Value>) -> Result<ToolCallResult> {
        let params = ToolCallRequest {
            name: name.to_string(),
            arguments,
        };

        self.send_request("tools/call", Some(serde_json::to_value(params)?))
    }

    /// Send a request and wait for response
    fn send_request<T: serde::de::DeserializeOwned>(&mut self, method: &str, params: Option<serde_json::Value>) -> Result<T> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        let request = McpRequest::new(id, method, params);

        match &self.transport {
            Transport::Stdio(stdio) => {
                let mut process = stdio.process.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
                
                // Send request
                let stdin = process.stdin.as_mut().ok_or_else(|| anyhow!("No stdin"))?;
                let request_json = serde_json::to_string(&request)?;
                writeln!(stdin, "{}", request_json)?;
                stdin.flush()?;

                // Read response
                let stdout = process.stdout.as_mut().ok_or_else(|| anyhow!("No stdout"))?;
                let mut reader = BufReader::new(stdout);
                let mut line = String::new();
                reader.read_line(&mut line)?;

                let response: McpResponse = serde_json::from_str(&line)?;
                
                if let Some(error) = response.error {
                    return Err(anyhow!("MCP error {}: {}", error.code, error.message));
                }

                let result = response.result.ok_or_else(|| anyhow!("No result in response"))?;
                Ok(serde_json::from_value(result)?)
            }
            Transport::Http(_http) => {
                // HTTP transport would use reqwest here
                Err(anyhow!("HTTP transport not yet implemented"))
            }
        }
    }

    /// Send a notification (no response expected)
    fn send_notification(&mut self, method: &str, params: Option<serde_json::Value>) -> Result<()> {
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });

        match &self.transport {
            Transport::Stdio(stdio) => {
                let mut process = stdio.process.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
                let stdin = process.stdin.as_mut().ok_or_else(|| anyhow!("No stdin"))?;
                let json = serde_json::to_string(&notification)?;
                writeln!(stdin, "{}", json)?;
                stdin.flush()?;
                Ok(())
            }
            Transport::Http(_) => {
                Err(anyhow!("HTTP transport not yet implemented"))
            }
        }
    }

    /// Shutdown the client
    pub fn shutdown(&mut self) -> Result<()> {
        if let Transport::Stdio(stdio) = &self.transport {
            if let Ok(mut process) = stdio.process.lock() {
                let _ = process.kill();
            }
        }
        Ok(())
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        // Just test that the struct can be created
        // Actual server tests would need a mock MCP server
        let result = McpClient::new_stdio("test", "nonexistent_binary", &[]);
        assert!(result.is_err()); // Expected to fail without the binary
    }
}
