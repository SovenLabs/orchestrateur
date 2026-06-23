//! Client et serveur MCP JSON-RPC 2.0 (transport stdio).

mod client;
mod error;
mod manager;
mod server;

pub use client::McpStdioClient;
pub use error::McpClientError;
pub use manager::{build_mcp_gateway, McpManager};
pub use server::run_stdio_server;