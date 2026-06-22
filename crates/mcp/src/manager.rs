use std::sync::Arc;

use async_trait::async_trait;
use orchestrator::mcp::{McpError, McpGateway, McpToolInfo};
use orchestrator::McpConfig;
use serde_json::Value;
use tracing::warn;

use crate::client::McpStdioClient;
use crate::error::McpClientError;

/// Gestionnaire multi-serveurs MCP.
pub struct McpManager {
    clients: Vec<Arc<McpStdioClient>>,
}

impl McpManager {
    /// Connecte tous les serveurs configurés (ignore les échecs individuels).
    ///
    /// # Errors
    ///
    /// Retourne une erreur uniquement si aucun serveur n'a pu démarrer alors que la liste est non vide.
    pub async fn connect(config: &McpConfig) -> Result<Self, McpClientError> {
        let mut clients = Vec::new();
        for server in &config.servers {
            match McpStdioClient::spawn(&server.name, &server.command, &server.args).await {
                Ok(client) => clients.push(Arc::new(client)),
                Err(err) => {
                    warn!(server = %server.name, %err, "serveur MCP ignoré au démarrage");
                }
            }
        }
        if clients.is_empty() && !config.servers.is_empty() {
            return Err(McpClientError::Spawn {
                command: "mcp".into(),
                message: "aucun serveur MCP n'a démarré".into(),
            });
        }
        Ok(Self { clients })
    }

    /// Nombre de serveurs connectés.
    #[must_use]
    pub fn server_count(&self) -> usize {
        self.clients.len()
    }
}

#[async_trait]
impl McpGateway for McpManager {
    async fn list_tools(&self) -> Result<Vec<McpToolInfo>, McpError> {
        let mut all = Vec::new();
        for client in &self.clients {
            match client.list_tools().await {
                Ok(tools) => {
                    for tool in tools {
                        all.push(McpToolInfo {
                            server: client.name().to_string(),
                            name: tool.name,
                            description: tool.description,
                        });
                    }
                }
                Err(err) => {
                    warn!(server = %client.name(), %err, "list_tools MCP échoué");
                }
            }
        }
        Ok(all)
    }

    async fn call_tool(
        &self,
        server: &str,
        tool: &str,
        arguments: Value,
    ) -> Result<String, McpError> {
        let client = self
            .clients
            .iter()
            .find(|c| c.name() == server)
            .ok_or_else(|| McpError::ServerNotFound(server.into()))?;
        client
            .call_tool(tool, arguments)
            .await
            .map_err(|e| McpError::Protocol {
                server: server.into(),
                message: e.to_string(),
            })
    }
}

/// Construit un gateway MCP depuis la configuration (None si désactivé).
pub async fn build_mcp_gateway(
    config: &McpConfig,
) -> Result<Option<Arc<dyn McpGateway>>, McpClientError> {
    if !config.enabled {
        return Ok(None);
    }
    let manager = McpManager::connect(config).await?;
    Ok(Some(Arc::new(manager)))
}