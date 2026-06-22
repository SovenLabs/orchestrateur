//! Port MCP — connexion serveurs Model Context Protocol (Phase 9).

use async_trait::async_trait;
use serde_json::Value;
use thiserror::Error;

/// Métadonnées d'un outil MCP exposé.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpToolInfo {
    /// Nom du serveur MCP source.
    pub server: String,
    /// Nom de l'outil.
    pub name: String,
    /// Description.
    pub description: String,
}

/// Erreur port MCP.
#[derive(Debug, Error)]
pub enum McpError {
    /// Serveur introuvable.
    #[error("serveur MCP introuvable: {0}")]
    ServerNotFound(String),
    /// Outil introuvable.
    #[error("outil MCP introuvable: {server}/{tool}")]
    ToolNotFound {
        /// Serveur.
        server: String,
        /// Outil.
        tool: String,
    },
    /// Échec protocole / transport.
    #[error("MCP {server}: {message}")]
    Protocol {
        /// Serveur.
        server: String,
        /// Détail.
        message: String,
    },
    /// MCP désactivé.
    #[error("MCP désactivé dans la configuration")]
    Disabled,
}

/// Port d'accès aux serveurs MCP (implémenté par le crate `mcp`).
#[async_trait]
pub trait McpGateway: Send + Sync {
    /// Liste les outils de tous les serveurs connectés.
    async fn list_tools(&self) -> Result<Vec<McpToolInfo>, McpError>;

    /// Appelle un outil MCP.
    async fn call_tool(
        &self,
        server: &str,
        tool: &str,
        arguments: Value,
    ) -> Result<String, McpError>;
}