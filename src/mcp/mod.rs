//! MCP (Model Context Protocol) Client and Server
//! 
//! - Server: Exposes Webrana skills as MCP tools
//! - Client: Connects to external MCP servers to discover additional tools

pub mod protocol;
pub mod server;
pub mod client;
pub mod registry;

pub use protocol::*;
pub use client::McpClient;
pub use registry::{McpRegistry, McpConfig, McpServerConfig, format_mcp_tools_for_llm};
