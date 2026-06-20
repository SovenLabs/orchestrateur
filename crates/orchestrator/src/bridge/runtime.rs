use std::str::FromStr;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use cortex::{KnowledgeGraph, MemoryId, SearchFilter};
use flume::{Receiver, Sender};
use tracing::error;

use crate::deps::AppDependencies;
use crate::error::SkillError;
use crate::facade::OrchestratorFacade;
use crate::health::probe_services;
use crate::skills::SkillContext;
use crate::VERSION;

use super::command::Command;
use super::error::BridgeError;
use super::events::FanoutEventPublisher;
use super::handle::ChannelHandle;
use super::response::Response;
use super::types::{AppError, HubSummary, MemorySummary, SkillSummary};

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
        Command::Search { query, limit } => execute_search(facade, &query, limit).await,
        Command::Assimilate { text, tags } => execute_assimilate(facade, &text, &tags).await,
        Command::Graph => execute_graph(facade).await,
        Command::Audit { limit } => execute_audit(facade, limit),
        Command::Chat { message } => execute_chat(facade, &message).await,
        Command::ListSkills => execute_list_skills(facade),
        Command::ExecuteSkill { name, context } => {
            execute_skill(facade, &name, context.into()).await
        }
    }
}

async fn execute_search(facade: &OrchestratorFacade, query: &str, limit: usize) -> Response {
    let probe = probe_services(facade.deps()).await;
    if !probe.embedding_available {
        return Response::Error(AppError {
            kind: "degraded".to_string(),
            message: "recherche indisponible — provider embeddings hors ligne".to_string(),
        });
    }
    let filter = SearchFilter {
        limit: Some(limit),
        ..SearchFilter::default()
    };
    match facade.search_memories(query, &filter).await {
        Ok(items) => Response::SearchResults { items },
        Err(err) => Response::Error(AppError::from_orchestrator(&err)),
    }
}

async fn execute_assimilate(
    facade: &OrchestratorFacade,
    text: &str,
    tags: &[String],
) -> Response {
    let prompt = format_assimilate_user_prompt(text, tags);
    match facade.assimilate(&prompt, None).await {
        Ok((memory, _events)) => Response::Assimilated {
            memory_id: memory.id,
            title: memory.title,
        },
        Err(err) => Response::Error(AppError::from_orchestrator(&err)),
    }
}

async fn execute_graph(facade: &OrchestratorFacade) -> Response {
    match facade.list_memories().await {
        Ok(memories) => {
            let graph = KnowledgeGraph::from_memories(&memories);
            let title_by_id: std::collections::HashMap<_, _> =
                memories.iter().map(|m| (m.id, m.title.as_str())).collect();
            let hubs = graph
                .hub_ranking()
                .into_iter()
                .take(10)
                .map(|(memory_id, inbound_links)| HubSummary {
                    memory_id,
                    title: title_by_id
                        .get(&memory_id)
                        .map_or_else(|| memory_id.to_string(), |t| (*t).to_string()),
                    inbound_links,
                })
                .collect();
            Response::GraphSummary {
                node_count: graph.node_count(),
                edge_count: graph.edge_count(),
                hubs,
            }
        }
        Err(err) => Response::Error(AppError::from_orchestrator(&err)),
    }
}

async fn execute_chat(facade: &OrchestratorFacade, message: &str) -> Response {
    let probe = probe_services(facade.deps()).await;
    if !probe.llm_available {
        return Response::Error(AppError {
            kind: "degraded".to_string(),
            message: "chat indisponible — provider LLM hors ligne".to_string(),
        });
    }
    match facade.chat(message).await {
        Ok(reply) => Response::ChatReply { reply },
        Err(err) => Response::Error(AppError::from_orchestrator(&err)),
    }
}

fn execute_list_skills(facade: &OrchestratorFacade) -> Response {
    let skills = facade
        .list_skills()
        .into_iter()
        .map(|(name, description)| SkillSummary {
            name: name.to_string(),
            description: description.to_string(),
        })
        .collect();
    Response::SkillList { skills }
}

async fn execute_skill(facade: &OrchestratorFacade, name: &str, ctx: SkillContext) -> Response {
    match facade.execute_skill(name, &ctx).await {
        Ok(output) => Response::SkillResult {
            message: output.message,
        },
        Err(SkillError::NotFound(skill)) => Response::Error(AppError {
            kind: "skill".to_string(),
            message: format!("skill introuvable: {skill}"),
        }),
        Err(SkillError::ExecutionFailed(message)) => Response::Error(AppError {
            kind: "skill".to_string(),
            message,
        }),
    }
}

fn execute_audit(facade: &OrchestratorFacade, limit: usize) -> Response {
    let security = facade.deps().security.as_ref();
    match security.read_audit_recent(limit) {
        Ok(entries) => {
            let chain_intact = security.verify_audit_chain();
            Response::AuditLog {
                entries,
                chain_intact,
            }
        }
        Err(err) => Response::Error(AppError {
            kind: "security".to_string(),
            message: err.to_string(),
        }),
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
    async fn dispatch_list_skills_includes_operational_skills() {
        let facade = OrchestratorFacade::new(MockBundle::new().into_deps());
        let response = execute_command(&facade, Command::ListSkills).await;
        match response {
            Response::SkillList { skills } => {
                assert!(skills.iter().any(|s| s.name == "search"));
            }
            other => panic!("réponse inattendue: {other:?}"),
        }
    }

    #[tokio::test]
    async fn dispatch_chat_returns_reply() {
        let facade = OrchestratorFacade::new(MockBundle::new().into_deps());
        let response = execute_command(
            &facade,
            Command::Chat {
                message: "ping".into(),
            },
        )
        .await;
        match response {
            Response::ChatReply { reply } => assert!(!reply.is_empty()),
            other => panic!("réponse inattendue: {other:?}"),
        }
    }

    #[tokio::test]
    #[ignore = "charge: dispatch list sur 5000 mémoires"]
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
