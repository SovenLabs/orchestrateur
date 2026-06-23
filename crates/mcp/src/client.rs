use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::Mutex;
use tracing::{debug, warn};

use crate::error::McpClientError;

/// Client MCP stdio (un serveur enfant).
pub struct McpStdioClient {
    name: String,
    _child: Child,
    stdin: Arc<Mutex<ChildStdin>>,
    next_id: AtomicU64,
    reader_task: tokio::task::JoinHandle<()>,
    pending: Arc<Mutex<std::collections::HashMap<u64, tokio::sync::oneshot::Sender<JsonRpcResponse>>>>,
}

impl McpStdioClient {
    /// Lance un serveur MCP via stdio et effectue l'initialisation.
    ///
    /// # Errors
    ///
    /// Propage [`McpClientError`] si le spawn ou l'handshake échoue.
    pub async fn spawn(
        name: impl Into<String>,
        command: &str,
        args: &[String],
    ) -> Result<Self, McpClientError> {
        if command.is_empty() {
            return Err(McpClientError::Spawn {
                command: String::new(),
                message: "commande vide".into(),
            });
        }

        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);

        let mut child = cmd.spawn().map_err(|e| McpClientError::Spawn {
            command: command.into(),
            message: e.to_string(),
        })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| McpClientError::Io("stdin indisponible".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| McpClientError::Io("stdout indisponible".into()))?;

        let pending: Arc<
            Mutex<std::collections::HashMap<u64, tokio::sync::oneshot::Sender<JsonRpcResponse>>>,
        > = Arc::new(Mutex::new(std::collections::HashMap::new()));

        let pending_reader = Arc::clone(&pending);
        let reader_task = tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let parsed: JsonRpcResponse = match serde_json::from_str(&line) {
                    Ok(v) => v,
                    Err(err) => {
                        warn!(%err, "ligne MCP non JSON");
                        continue;
                    }
                };
                if let Some(id) = parsed.id {
                    let mut guard = pending_reader.lock().await;
                    if let Some(tx) = guard.remove(&id) {
                        let _ = tx.send(parsed);
                    }
                }
            }
        });

        let client = Self {
            name: name.into(),
            _child: child,
            stdin: Arc::new(Mutex::new(stdin)),
            next_id: AtomicU64::new(1),
            reader_task,
            pending,
        };

        client.initialize().await?;
        Ok(client)
    }

    /// Nom du serveur.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    async fn initialize(&self) -> Result<(), McpClientError> {
        let _: Value = self
            .request(
                "initialize",
                serde_json::json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": { "name": "orchestrateur", "version": orchestrator::VERSION }
                }),
            )
            .await?;
        self.notify("notifications/initialized", Value::Null).await?;
        Ok(())
    }

    async fn notify(&self, method: &str, params: Value) -> Result<(), McpClientError> {
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });
        let line =
            serde_json::to_string(&payload).map_err(|e| McpClientError::Json(e.to_string()))?;
        let mut stdin = self.stdin.lock().await;
        stdin
            .write_all(line.as_bytes())
            .await
            .map_err(|e| McpClientError::Io(e.to_string()))?;
        stdin
            .write_all(b"\n")
            .await
            .map_err(|e| McpClientError::Io(e.to_string()))?;
        stdin
            .flush()
            .await
            .map_err(|e| McpClientError::Io(e.to_string()))?;
        Ok(())
    }

    /// Liste les outils MCP du serveur.
    ///
    /// # Errors
    ///
    /// Propage [`McpClientError`].
    pub async fn list_tools(&self) -> Result<Vec<McpToolDescriptor>, McpClientError> {
        let result: ToolsListResult = self.request("tools/list", Value::Null).await?;
        Ok(result.tools)
    }

    /// Appelle un outil MCP.
    ///
    /// # Errors
    ///
    /// Propage [`McpClientError`].
    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<String, McpClientError> {
        let result: ToolCallResult = self
            .request(
                "tools/call",
                serde_json::json!({ "name": name, "arguments": arguments }),
            )
            .await?;
        Ok(result
            .content
            .into_iter()
            .map(|c| c.text)
            .collect::<Vec<_>>()
            .join("\n"))
    }

    async fn request<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: Value,
    ) -> Result<T, McpClientError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id,
            method,
            params,
        };
        let line = serde_json::to_string(&request).map_err(|e| McpClientError::Json(e.to_string()))?;
        debug!(server = %self.name, %method, "MCP request");

        let (tx, rx) = tokio::sync::oneshot::channel();
        {
            let mut guard = self.pending.lock().await;
            guard.insert(id, tx);
        }

        {
            let mut stdin = self.stdin.lock().await;
            stdin
                .write_all(line.as_bytes())
                .await
                .map_err(|e| McpClientError::Io(e.to_string()))?;
            stdin
                .write_all(b"\n")
                .await
                .map_err(|e| McpClientError::Io(e.to_string()))?;
            stdin
                .flush()
                .await
                .map_err(|e| McpClientError::Io(e.to_string()))?;
        }

        let response = tokio::time::timeout(std::time::Duration::from_secs(60), rx)
            .await
            .map_err(|_| McpClientError::Timeout)?
            .map_err(|_| McpClientError::Io("réponse MCP abandonnée".into()))?;

        if let Some(err) = response.error {
            return Err(McpClientError::Rpc {
                code: err.code,
                message: err.message,
            });
        }

        let result = response.result.ok_or_else(|| McpClientError::Rpc {
            code: -1,
            message: "résultat RPC absent".into(),
        })?;

        serde_json::from_value(result).map_err(|e| McpClientError::Json(e.to_string()))
    }
}

/// Descripteur outil MCP.
#[derive(Debug, Clone, Deserialize)]
pub struct McpToolDescriptor {
    /// Nom de l'outil.
    pub name: String,
    /// Description.
    #[serde(default)]
    pub description: String,
}

#[derive(Serialize)]
struct JsonRpcRequest<'a> {
    jsonrpc: &'a str,
    id: u64,
    method: &'a str,
    params: Value,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    id: Option<u64>,
    result: Option<Value>,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

#[derive(Debug, Deserialize)]
struct ToolsListResult {
    tools: Vec<McpToolDescriptor>,
}

#[derive(Debug, Deserialize)]
struct ToolCallResult {
    content: Vec<ToolContent>,
}

#[derive(Debug, Deserialize)]
struct ToolContent {
    text: String,
}

impl Drop for McpStdioClient {
    fn drop(&mut self) {
        self.reader_task.abort();
    }
}