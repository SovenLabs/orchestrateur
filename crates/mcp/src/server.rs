//! Serveur MCP stdio — expose Cortex + Esprit + harness health.

use std::sync::Arc;

use orchestrator::bridge::{execute_command, Command, Response};
use orchestrator::OrchestratorFacade;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::io::{stdin, stdout};
use tracing::{debug, warn};

/// Lance le serveur MCP sur stdin/stdout jusqu'à EOF.
///
/// # Errors
///
/// Propage les erreurs d'E/S stdio.
pub async fn run_stdio_server(
    facade: Arc<OrchestratorFacade>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut reader = BufReader::new(stdin());
    let mut stdout = stdout();
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let request: IncomingRequest = match serde_json::from_str(trimmed) {
            Ok(r) => r,
            Err(err) => {
                warn!(%err, "MCP ligne invalide");
                continue;
            }
        };

        if request.id.is_none() {
            debug!(method = %request.method, "MCP notification ignorée");
            continue;
        }

        let id = request.id.unwrap_or(Value::Null);
        let response = handle_request(Arc::clone(&facade), &request.method, request.params).await;
        let out = json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": response.result,
            "error": response.error,
        });
        let mut payload = serde_json::to_string(&out)?;
        payload.push('\n');
        stdout.write_all(payload.as_bytes()).await?;
        stdout.flush().await?;
    }

    Ok(())
}

struct HandlerResult {
    result: Option<Value>,
    error: Option<Value>,
}

async fn handle_request(
    facade: Arc<OrchestratorFacade>,
    method: &str,
    params: Option<Value>,
) -> HandlerResult {
    match method {
        "initialize" => HandlerResult {
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": "orchestrateur",
                    "version": orchestrator::VERSION
                }
            })),
            error: None,
        },
        "notifications/initialized" | "initialized" => HandlerResult {
            result: Some(Value::Null),
            error: None,
        },
        "tools/list" => HandlerResult {
            result: Some(json!({ "tools": tool_catalog() })),
            error: None,
        },
        "tools/call" => match params {
            Some(p) => call_tool(facade, p).await,
            None => rpc_error(-32602, "params requis pour tools/call"),
        },
        _ => rpc_error(-32601, &format!("méthode inconnue: {method}")),
    }
}

fn rpc_error(code: i64, message: &str) -> HandlerResult {
    HandlerResult {
        result: None,
        error: Some(json!({ "code": code, "message": message })),
    }
}

fn tool_catalog() -> Vec<Value> {
    vec![
        tool_desc(
            "cortex_search",
            "Recherche sémantique dans le Cortex (LanceDB + graphe).",
            json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" },
                    "limit": { "type": "integer", "default": 10 }
                },
                "required": ["query"]
            }),
        ),
        tool_desc(
            "cortex_get",
            "Récupère une mémoire Cortex par UUID.",
            json!({
                "type": "object",
                "properties": { "id": { "type": "string" } },
                "required": ["id"]
            }),
        ),
        tool_desc(
            "cortex_graph",
            "Statistiques du graphe de connaissances Cortex.",
            json!({ "type": "object", "properties": {} }),
        ),
        tool_desc(
            "cortex_assimilate",
            "Assimile du texte dans le Cortex via l'Esprit (LLM).",
            json!({
                "type": "object",
                "properties": {
                    "text": { "type": "string" },
                    "tags": { "type": "array", "items": { "type": "string" } }
                },
                "required": ["text"]
            }),
        ),
        tool_desc(
            "draft_list",
            "Liste les brouillons en attente de publication.",
            json!({ "type": "object", "properties": {} }),
        ),
        tool_desc(
            "draft_publish",
            "Publie un brouillon en mémoire Cortex.",
            json!({
                "type": "object",
                "properties": { "id": { "type": "string" } },
                "required": ["id"]
            }),
        ),
        tool_desc(
            "esprit_chat",
            "Un tour de chat avec l'Esprit (agent loop).",
            json!({
                "type": "object",
                "properties": { "message": { "type": "string" } },
                "required": ["message"]
            }),
        ),
        tool_desc(
            "harness_health",
            "Santé du harness intégré (Cortex + Esprit + providers).",
            json!({ "type": "object", "properties": {} }),
        ),
    ]
}

fn tool_desc(name: &str, description: &str, schema: Value) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": schema
    })
}

async fn call_tool(facade: Arc<OrchestratorFacade>, params: Value) -> HandlerResult {
    let call: ToolCallParams = match serde_json::from_value(params) {
        Ok(c) => c,
        Err(err) => return rpc_error(-32602, &format!("params invalides: {err}")),
    };

    let response = match call.name.as_str() {
        "cortex_search" => {
            let query = call
                .arguments
                .get("query")
                .and_then(Value::as_str)
                .unwrap_or("");
            let limit = call
                .arguments
                .get("limit")
                .and_then(Value::as_u64)
                .unwrap_or(10) as usize;
            execute_command(&facade, Command::Search {
                query: query.into(),
                limit,
            })
            .await
        }
        "cortex_get" => {
            let id = call
                .arguments
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("");
            execute_command(&facade, Command::GetMemory { id: id.into() }).await
        }
        "cortex_graph" => execute_command(&facade, Command::Graph).await,
        "cortex_assimilate" => {
            let text = call
                .arguments
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or("");
            let tags: Vec<String> = call
                .arguments
                .get("tags")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            execute_command(
                &facade,
                Command::Assimilate {
                    text: text.into(),
                    tags,
                },
            )
            .await
        }
        "draft_list" => execute_command(&facade, Command::ListDrafts).await,
        "draft_publish" => {
            let id = call
                .arguments
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("");
            execute_command(&facade, Command::PublishDraft { id: id.into() }).await
        }
        "esprit_chat" => {
            let message = call
                .arguments
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("");
            execute_command(&facade, Command::Chat {
                message: message.into(),
            })
            .await
        }
        "harness_health" => execute_command(&facade, Command::HealthCheck).await,
        other => {
            return rpc_error(-32602, &format!("outil inconnu: {other}"));
        }
    };

    let is_error = matches!(&response, Response::Error(_));
    HandlerResult {
        result: Some(tool_result(response, is_error)),
        error: None,
    }
}

fn tool_result(response: Response, is_error: bool) -> Value {
    let text = match response {
        Response::Error(err) => format!("[{}] {}", err.kind, err.message),
        other => serde_json::to_string_pretty(&other).unwrap_or_else(|_| format!("{other:?}")),
    };
    json!({
        "content": [{ "type": "text", "text": text }],
        "isError": is_error
    })
}

#[derive(Debug, Deserialize)]
struct IncomingRequest {
    #[serde(default)]
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct ToolCallParams {
    name: String,
    #[serde(default)]
    arguments: Value,
}

