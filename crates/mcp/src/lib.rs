//! Client MCP JSON-RPC 2.0 (transport stdio) — Phase 9.

mod client;
mod error;
mod manager;

pub use client::McpStdioClient;
pub use error::McpClientError;
pub use manager::{build_mcp_gateway, McpManager};