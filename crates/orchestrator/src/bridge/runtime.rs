use std::str::FromStr;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use cortex::{MemoryId, SearchFilter};
use flume::{Receiver, Sender};
use tracing::error;

use crate::deps::AppDependencies;
use crate::facade::OrchestratorFacade;
use crate::health::probe_services;
use crate::VERSION;

use super::command::Command;
use super::error::BridgeError;
use super::events::FanoutEventPublisher;
use super::handle::ChannelHandle;
use super::response::Response;
use super::types::{AppError, MemorySummary};

/// Capacité des canaux commandes/réponses (back-pressure léger).
const CMD_CHANNEL_CAPACITY: usize = 64;
const RESP_CHANNEL_CAPACITY: usize = 128;

/// Démarre le thread orchestrateur avec runtime Tokio dédié.
///
/// Retourne un [`ChannelHandle`] clonable pour le thread UI et le handle du thread
/// (joignable à l'arrêt de l'application).
///
/// # Errors
///
/// Retourne [`BridgeError::ThreadSpawn`] ou [`BridgeError::RuntimeBuild`] si le
/// démarrage du thread orchestrateur échoue.
pub fn spawn_orchestrator_bridge(
    deps: AppDependencies,
) -> Result<(ChannelHandle, OrchestratorThread), BridgeError> {
    let fanout = FanoutEventPublisher::new();
    let deps = deps_with_fanout(deps, fanout.clone());
    let facade = OrchestratorFacade::new(deps);

    let (cmd_tx, cmd_rx) = flume::bounded(CMD_CHANNEL_CAPACITY);
    let (resp_tx, resp_rx) = flume::bounded(RESP_CHANNEL_CAPACITY);

    let join = thread::Builder::new()
        .name("orchestrator-bridge".into())
        .spawn(move || orchestrator_thread_main(facade, cmd_rx, resp_tx))?;

    let handle = ChannelHandle::new(cmd_tx, resp_rx, fanout);
    Ok((handle, OrchestratorThread { join: Some(join) }))
}

/// Thread orchestrateur joignable proprement à la fermeture du HUD.
pub struct OrchestratorThread {
    join: Option<JoinHandle<()>>,
}

impl OrchestratorThread {
    /// Attend la fin du thread orchestrateur (fermeture du canal commandes).
    pub fn join(mut self) {
        if let Some(join) = self.join.take() {
            if let Err(err) = join.join() {
                error!(?err, "thread orchestrateur terminé avec panic");
            }
        }
    }
}

fn deps_with_fanout(mut deps: AppDependencies, fanout: FanoutEventPublisher) -> AppDependencies {
    deps.events = Arc::new(fanout);
    deps
}

fn orchestrator_thread_main(
    facade: OrchestratorFacade,
    cmd_rx: Receiver<Command>,
    resp_tx: Sender<Response>,
) {
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("orchestrator-tokio")
        .build()
    {
        Ok(runtime) => runtime,
        Err(err) => {
            error!(%err, "échec création runtime tokio");
            return;
        }
    };

    rt.block_on(async {
        orchestrator_loop(facade, cmd_rx, resp_tx).await;
    });
}

async fn orchestrator_loop(
    facade: OrchestratorFacade,
    cmd_rx: Receiver<Command>,
    resp_tx: Sender<Response>,
) {
    while let Ok(cmd) = cmd_rx.recv_async().await {
        let response = execute_command(&facade, cmd).await;
        if resp_tx.send_async(response).await.is_err() {
            break;
        }
    }
}

/// Exécute une [`Command`] bridge de façon synchrone (CLI headless, tests, thread bridge).
pub async fn execute_command(facade: &OrchestratorFacade, cmd: Command) -> Response {
    match cmd {
        Command::HealthCheck => {
            let probe = probe_services(facade.deps()).await;
            let status = if probe.llm_available && probe.embedding_available {
                "ok"
            } else {
                "degraded"
            };
            Response::Health {
                status: status.to_string(),
                version: VERSION.to_string(),
                llm_available: probe.llm_available,
                embedding_available: probe.embedding_available,
            }
        }
        Command::SubscribeToEvents => Response::Success {
            message: "abonnement événements actif".to_string(),
        },
        Command::List {
            filter,
            offset,
            limit,
        } => match facade.list_memories().await {
            Ok(memories) => {
                let mut summaries: Vec<MemorySummary> =
                    memories.iter().map(MemorySummary::from_memory).collect();
                if let Some(ref needle) = filter {
                    if !needle.is_empty() {
                        summaries.retain(|item| item.matches_filter(needle));
                    }
                }
                let total = summaries.len();
                let items = summaries.into_iter().skip(offset).take(limit).collect();
                Response::MemoryList { items, total }
            }
            Err(err) => Response::Error(AppError::from_orchestrator(&err)),
        },
        Command::GetMemory { id } => match MemoryId::from_str(&id) {
            Ok(memory_id) => match facade.get_memory(memory_id).await {
                Ok(memory) => Response::MemoryDetail { memory },
                Err(err) => Response::Error(AppError::from_orchestrator(&err)),
            },
            Err(err) => Response::Error(AppError {
                kind: "invalid_id".to_string(),
                message: err.to_string(),
            }),
        },
        Command::Search { query, limit } => {
            let filter = SearchFilter {
                limit: Some(limit),
                ..SearchFilter::default()
            };
            match facade.search_memories(&query, &filter).await {
                Ok(items) => Response::SearchResults { items },
                Err(err) => Response::Error(AppError::from_orchestrator(&err)),
            }
        }
        Command::Assimilate { text, tags } => {
            let prompt = format_assimilate_user_prompt(&text, &tags);
            match facade.assimilate(&prompt, None).await {
                Ok((memory, _events)) => Response::MemoryDetail { memory },
                Err(err) => Response::Error(AppError::from_orchestrator(&err)),
            }
        }
    }
}

/// Formate le prompt utilisateur d'assimilation (tags suggérés + texte).
#[must_use]
pub fn format_assimilate_user_prompt(text: &str, tags: &[String]) -> String {
    if tags.is_empty() {
        return text.to_string();
    }
    let tag_list = tags.join(", ");
    format!("Tags suggérés: {tag_list}\n\n{text}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MockBundle;

    #[test]
    fn format_assimilate_prompt_includes_tags() {
        let prompt = format_assimilate_user_prompt("contenu", &["rust".into(), "egui".into()]);
        assert!(prompt.contains("rust"));
        assert!(prompt.contains("contenu"));
    }

    #[tokio::test]
    async fn dispatch_health_check_returns_version() {
        let facade = OrchestratorFacade::new(MockBundle::new().into_deps());
        let response = execute_command(&facade, Command::HealthCheck).await;
        match response {
            Response::Health {
                status,
                version,
                llm_available,
                embedding_available,
            } => {
                assert_eq!(status, "ok");
                assert_eq!(version, VERSION);
                assert!(llm_available);
                assert!(embedding_available);
            }
            other => panic!("réponse inattendue: {other:?}"),
        }
    }

    #[tokio::test]
    async fn dispatch_list_returns_saved_memories() {
        let facade = OrchestratorFacade::new(MockBundle::new().into_deps());
        let memory = cortex::Memory::new("Titre HUD", "Corps").unwrap();
        facade.save_memory(&memory).await.unwrap();

        let response = execute_command(
            &facade,
            Command::List {
                filter: None,
                offset: 0,
                limit: 10,
            },
        )
        .await;

        match response {
            Response::MemoryList { items, total } => {
                assert_eq!(total, 1);
                assert_eq!(items[0].title, "Titre HUD");
            }
            other => panic!("réponse inattendue: {other:?}"),
        }
    }

    #[tokio::test]
    async fn dispatch_list_handles_5k_memories() {
        use std::time::Instant;

        use cortex::Memory;

        let facade = OrchestratorFacade::new(MockBundle::new().into_deps());
        for i in 0..5000_u32 {
            let mem = Memory::new(format!("Mémoire {i}"), format!("Contenu {i}")).unwrap();
            facade.save_memory(&mem).await.unwrap();
        }

        let start = Instant::now();
        let response = execute_command(
            &facade,
            Command::List {
                filter: None,
                offset: 0,
                limit: 10_000,
            },
        )
        .await;
        let elapsed = start.elapsed();

        match response {
            Response::MemoryList { items, total } => {
                assert_eq!(total, 5000);
                assert_eq!(items.len(), 5000);
            }
            other => panic!("réponse inattendue: {other:?}"),
        }
        assert!(elapsed.as_millis() < 2000, "list 5k trop lent: {elapsed:?}");
    }
}
