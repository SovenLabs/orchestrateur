use std::str::FromStr;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use cortex::{KnowledgeGraph, MemoryId, SearchFilter, SessionKey};
use flume::{Receiver, Sender};
use tracing::error;

use crate::deps::AppDependencies;
use crate::error::SkillError;
use crate::facade::OrchestratorFacade;
use crate::health::probe_services;
use crate::security::{assert_llm_egress_allowed, assert_text_safe_for_llm};
use crate::skills::SkillsMarketplace;
use crate::skills::SkillContext;
use crate::VERSION;

use super::command::Command;
use super::error::BridgeError;
use super::events::FanoutEventPublisher;
use super::handle::ChannelHandle;
use super::response::Response;
use super::types::{
    AppError, HubIntegritySummary, HubSummary, MarketplaceEntrySummary, MemorySummary,
    SkillSummary,
};

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

/// Thread orchestrateur joignable proprement à la fermeture du client bridge.
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
        Command::SkillsMarketplaceList => execute_marketplace_list(facade).await,
        Command::SkillsHubVerify => execute_hub_verify(facade),
        Command::WatcherStatus => execute_watcher_status(facade).await,
        Command::WatcherStart => execute_watcher_start(facade),
        Command::WatcherStop => execute_watcher_stop(),
        Command::ListDrafts => execute_list_drafts(facade).await,
        Command::GetDraft { id } => execute_get_draft(facade, &id).await,
        Command::PublishDraft { id } => execute_publish_draft(facade, &id).await,
        Command::DiscardDraft { id } => execute_discard_draft(facade, &id).await,
        Command::ListAgents => execute_list_agents(facade).await,
        Command::GetAgent { id } => execute_get_agent(facade, &id).await,
        Command::CreateAgent {
            id,
            name,
            role,
            model,
        } => execute_create_agent(facade, &id, &name, &role, model.as_deref()).await,
        Command::AgentWake { id } => execute_agent_wake(facade, &id).await,
        Command::AgentSleep { id } => execute_agent_sleep(facade, &id).await,
        Command::AgentBackground { id } => execute_agent_background(facade, &id).await,
        Command::AgentTurn { id, message } => execute_agent_turn(facade, &id, &message).await,
        Command::AgentSendMessage { from, to, body } => {
            execute_agent_send_message(facade, &from, &to, &body).await
        }
        Command::AgentMessages { id, mark_read } => {
            execute_agent_messages(facade, &id, mark_read).await
        }
    }
}

fn agent_summary(agent: &crate::persistent::PersistentAgent) -> super::types::AgentSummary {
    use crate::persistent::AgentIdentity;
    super::types::AgentSummary {
        id: agent.id().to_string(),
        name: agent.name().to_string(),
        role: agent.role().to_string(),
        model: agent.model().to_string(),
        status: agent.status().label().to_string(),
        session_key: agent.config.session_key.clone(),
        last_heartbeat: agent.config.last_heartbeat.clone(),
    }
}

fn agent_error(err: crate::persistent::PersistentAgentError) -> Response {
    Response::Error(AppError {
        kind: "persistent_agent".to_string(),
        message: err.to_string(),
    })
}

async fn execute_list_agents(facade: &OrchestratorFacade) -> Response {
    match facade.agent_manager().await {
        Ok(manager) => match manager.list().await {
            Ok(agents) => Response::AgentList {
                items: agents.iter().map(agent_summary).collect(),
            },
            Err(err) => agent_error(err),
        },
        Err(err) => agent_error(err),
    }
}

async fn execute_get_agent(facade: &OrchestratorFacade, id: &str) -> Response {
    match facade.agent_manager().await {
        Ok(manager) => match manager.get(id).await {
            Ok(agent) => Response::AgentDetail {
                agent: agent_summary(&agent),
            },
            Err(err) => agent_error(err),
        },
        Err(err) => agent_error(err),
    }
}

async fn execute_create_agent(
    facade: &OrchestratorFacade,
    id: &str,
    name: &str,
    role: &str,
    model: Option<&str>,
) -> Response {
    match facade.agent_manager().await {
        Ok(manager) => match manager.create_agent(id, name, role, model).await {
            Ok(agent) => Response::AgentDetail {
                agent: agent_summary(&agent),
            },
            Err(err) => agent_error(err),
        },
        Err(err) => agent_error(err),
    }
}

async fn execute_agent_wake(facade: &OrchestratorFacade, id: &str) -> Response {
    match facade.agent_manager().await {
        Ok(manager) => match manager.wake(id).await {
            Ok(agent) => Response::AgentDetail {
                agent: agent_summary(&agent),
            },
            Err(err) => agent_error(err),
        },
        Err(err) => agent_error(err),
    }
}

async fn execute_agent_sleep(facade: &OrchestratorFacade, id: &str) -> Response {
    match facade.agent_manager().await {
        Ok(manager) => match manager.sleep(id).await {
            Ok(agent) => Response::AgentDetail {
                agent: agent_summary(&agent),
            },
            Err(err) => agent_error(err),
        },
        Err(err) => agent_error(err),
    }
}

async fn execute_agent_background(facade: &OrchestratorFacade, id: &str) -> Response {
    match facade.agent_manager().await {
        Ok(manager) => match manager.background(id).await {
            Ok(report) => Response::AgentBackgroundReport {
                inbox_count: report.inbox_count,
                pending_tasks: report.pending_tasks,
                executed: report.executed,
            },
            Err(err) => agent_error(err),
        },
        Err(err) => agent_error(err),
    }
}

async fn execute_agent_turn(facade: &OrchestratorFacade, id: &str, message: &str) -> Response {
    match facade.agent_turn_for(id, message).await {
        Ok(result) => Response::AgentTurnReply {
            reply: result.reply,
            tools_invoked: result.tools_invoked,
            auto_assimilated: result.auto_assimilated,
            auto_executed_skills: result.auto_executed_skills,
        },
        Err(err) => Response::Error(AppError {
            kind: "agent".to_string(),
            message: err.to_string(),
        }),
    }
}

async fn execute_agent_send_message(
    facade: &OrchestratorFacade,
    from: &str,
    to: &str,
    body: &str,
) -> Response {
    match facade.agent_manager().await {
        Ok(manager) => match manager.send_message(from, to, body).await {
            Ok(msg) => Response::AgentMessageSent {
                message_id: msg.id,
                from: msg.from,
                to: msg.to,
            },
            Err(err) => agent_error(err),
        },
        Err(err) => agent_error(err),
    }
}

async fn execute_agent_messages(facade: &OrchestratorFacade, id: &str, mark_read: bool) -> Response {
    match facade.agent_manager().await {
        Ok(manager) => match manager.receive_messages(id, mark_read).await {
            Ok(messages) => Response::AgentMessages {
                items: messages
                    .into_iter()
                    .map(|m| super::types::AgentMessageSummary {
                        id: m.id,
                        from: m.from,
                        to: m.to,
                        body: m.body,
                        sent_at: m.sent_at,
                        read: m.read,
                    })
                    .collect(),
            },
            Err(err) => agent_error(err),
        },
        Err(err) => agent_error(err),
    }
}

async fn execute_get_draft(facade: &OrchestratorFacade, id: &str) -> Response {
    match facade.get_draft(id).await {
        Ok(draft) => Response::DraftDetail { draft },
        Err(err) => Response::Error(AppError::from_orchestrator(&err)),
    }
}

async fn execute_watcher_status(facade: &OrchestratorFacade) -> Response {
    if let Some(handle) = crate::watcher::global_handle() {
        let status = handle.status().await;
        return Response::WatcherStatus { status };
    }
    use crate::draft::DraftStatus;

    let drafts_pending = facade
        .deps()
        .draft_repo
        .list(Some(DraftStatus::Pending))
        .await
        .map(|d| d.len())
        .unwrap_or(0);
    let cfg = &facade.deps().config.watcher;
    Response::WatcherStatus {
        status: super::types::WatcherStatus {
            enabled: cfg.enabled,
            running: false,
            watch_dirs: cfg
                .watch_dirs
                .iter()
                .map(|d| facade.deps().config.workspace_root.join(d).display().to_string())
                .collect(),
            sessions_processed: 0,
            drafts_created: 0,
            drafts_pending,
            last_activity_at: None,
            last_error: None,
        },
    }
}

fn execute_watcher_start(facade: &OrchestratorFacade) -> Response {
    if !facade.deps().config.watcher.enabled {
        return Response::Error(AppError {
            kind: "watcher".into(),
            message: "watcher désactivé — activez [watcher] enabled = true".into(),
        });
    }
    if crate::watcher::global_handle().is_some() {
        return Response::Success {
            message: "watcher déjà actif".into(),
        };
    }
    let handle = std::sync::Arc::new(crate::watcher::SessionWatcherHandle::new(
        std::sync::Arc::new(OrchestratorFacade::new(facade.deps().clone())),
        None::<crate::watcher::DraftReadyCallback>,
        &facade.deps().config,
    ));
    crate::watcher::install_global(std::sync::Arc::clone(&handle));
    handle.spawn();
    Response::Success {
        message: "watcher démarré".into(),
    }
}

fn execute_watcher_stop() -> Response {
    if let Some(handle) = crate::watcher::global_handle() {
        handle.stop();
        Response::Success {
            message: "watcher arrêté".into(),
        }
    } else {
        Response::Success {
            message: "watcher inactif".into(),
        }
    }
}

async fn execute_list_drafts(facade: &OrchestratorFacade) -> Response {
    match facade.list_drafts().await {
        Ok(items) => {
            let total = items.len();
            Response::DraftList { items, total }
        }
        Err(err) => Response::Error(AppError::from_orchestrator(&err)),
    }
}

async fn execute_publish_draft(facade: &OrchestratorFacade, id: &str) -> Response {
    match facade.publish_draft(id).await {
        Ok((draft_id, (memory, _events))) => Response::DraftPublished {
            draft_id,
            memory_id: memory.id,
            title: memory.title,
        },
        Err(err) => Response::Error(AppError::from_orchestrator(&err)),
    }
}

async fn execute_discard_draft(facade: &OrchestratorFacade, id: &str) -> Response {
    match facade.discard_draft(id).await {
        Ok(()) => Response::DraftDiscarded { id: id.to_string() },
        Err(err) => Response::Error(AppError::from_orchestrator(&err)),
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
    let config = &facade.deps().config;
    if let Err(err) = assert_llm_egress_allowed(config) {
        return Response::Error(AppError {
            kind: err.code,
            message: err.message,
        });
    }
    if let Err(err) = assert_text_safe_for_llm(&config.security, text) {
        return Response::Error(AppError {
            kind: err.code,
            message: err.message,
        });
    }
    match facade.assimilate(text, tags, None).await {
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
    let config = &facade.deps().config;
    if let Err(err) = assert_llm_egress_allowed(config) {
        return Response::Error(AppError {
            kind: err.code,
            message: err.message,
        });
    }
    if let Err(err) = assert_text_safe_for_llm(&config.security, message) {
        return Response::Error(AppError {
            kind: err.code,
            message: err.message,
        });
    }
    let probe = probe_services(facade.deps()).await;
    if !probe.llm_available {
        return Response::Error(AppError {
            kind: "degraded".to_string(),
            message: "chat indisponible — provider LLM hors ligne".to_string(),
        });
    }
    match facade
        .agent_turn(SessionKey::default_chat(), message)
        .await
    {
        Ok(result) => Response::ChatReply {
            reply: result.reply,
            tools_invoked: result.tools_invoked,
            auto_assimilated: result.auto_assimilated,
            auto_executed_skills: result.auto_executed_skills,
        },
        Err(err) => Response::Error(AppError {
            kind: "agent".to_string(),
            message: err.to_string(),
        }),
    }
}

fn execute_list_skills(facade: &OrchestratorFacade) -> Response {
    let skills = facade
        .list_skills()
        .into_iter()
        .map(|entry| SkillSummary {
            name: entry.name,
            description: entry.description,
            source: match entry.source {
                crate::skills::SkillSource::Builtin => "builtin".into(),
                crate::skills::SkillSource::Hub => "hub".into(),
                crate::skills::SkillSource::Native => "native".into(),
            },
            version: entry.version,
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

async fn execute_marketplace_list(facade: &OrchestratorFacade) -> Response {
    let config = &facade.deps().config;
    match SkillsMarketplace::load_catalog_auto(config).await {
        Ok(catalog) => {
            let entries = catalog
                .skills
                .iter()
                .map(|entry| MarketplaceEntrySummary {
                    id: entry.id.clone(),
                    name: entry.name.clone(),
                    description: entry.description.clone(),
                    version: entry.version.clone(),
                    enabled: entry.enabled,
                })
                .collect();
            Response::MarketplaceList {
                version: catalog.version,
                catalog_hash: catalog.catalog_hash,
                entries,
            }
        }
        Err(err) => Response::Error(AppError {
            kind: "marketplace".to_string(),
            message: err.to_string(),
        }),
    }
}

fn execute_hub_verify(facade: &OrchestratorFacade) -> Response {
    match SkillsMarketplace::verify_hub_integrity(&facade.deps().config) {
        Ok(report) => Response::HubIntegrityReport {
            report: HubIntegritySummary {
                valid_count: report.valid.len(),
                invalid: report
                    .invalid
                    .into_iter()
                    .map(|(path, message)| (path.display().to_string(), message))
                    .collect(),
            },
        },
        Err(err) => Response::Error(AppError {
            kind: "marketplace".to_string(),
            message: err.to_string(),
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
    async fn dispatch_marketplace_list_reads_catalog() {
        use crate::skills::{MarketplaceCatalog, MarketplaceEntry};

        let dir = tempfile::tempdir().unwrap();
        let catalog = MarketplaceCatalog {
            version: 1,
            catalog_hash: None,
            skills: vec![MarketplaceEntry {
                id: "bridge-demo".into(),
                name: "Bridge Demo".into(),
                description: "Test bridge".into(),
                version: "0.1.0".into(),
                enabled: true,
                manifest_toml: "[skill]\nid=\"bridge-demo\"".into(),
            }],
        };
        std::fs::write(
            dir.path().join("catalog.json"),
            serde_json::to_string(&catalog).unwrap(),
        )
        .unwrap();
        let mut bundle = MockBundle::new();
        bundle.config.workspace_root = dir.path().to_path_buf();
        bundle.config.skills_hub.marketplace_catalog = "catalog.json".into();
        let facade = OrchestratorFacade::new(bundle.into_deps());
        let response = execute_command(&facade, Command::SkillsMarketplaceList).await;
        match response {
            Response::MarketplaceList { version, entries, .. } => {
                assert_eq!(version, 1);
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].id, "bridge-demo");
            }
            other => panic!("réponse inattendue: {other:?}"),
        }
    }

    #[tokio::test]
    async fn dispatch_hub_verify_returns_report() {
        let facade = OrchestratorFacade::new(MockBundle::new().into_deps());
        let response = execute_command(&facade, Command::SkillsHubVerify).await;
        match response {
            Response::HubIntegrityReport { report } => {
                assert_eq!(report.invalid.len(), 0);
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
            Response::ChatReply { reply, .. } => assert!(!reply.is_empty()),
            other => panic!("réponse inattendue: {other:?}"),
        }
    }

    #[tokio::test]
    async fn dispatch_draft_lifecycle() {
        use cortex::MemoryDraft;
        use crate::draft::DraftStatus;

        let facade = OrchestratorFacade::new(MockBundle::new().into_deps());
        let stored = facade
            .store_draft(
                MemoryDraft::new("Brouillon bridge", "Contenu test."),
                Some("sessions/demo.md".into()),
            )
            .await
            .unwrap();

        let list = execute_command(&facade, Command::ListDrafts).await;
        match list {
            Response::DraftList { items, total } => {
                assert_eq!(total, 1);
                assert_eq!(items[0].id, stored.id);
                assert_eq!(items[0].status, DraftStatus::Pending);
            }
            other => panic!("réponse inattendue: {other:?}"),
        }

        let detail = execute_command(
            &facade,
            Command::GetDraft {
                id: stored.id.clone(),
            },
        )
        .await;
        match detail {
            Response::DraftDetail { draft } => {
                assert_eq!(draft.id, stored.id);
                assert_eq!(draft.draft.title, "Brouillon bridge");
            }
            other => panic!("réponse inattendue: {other:?}"),
        }

        let published = execute_command(
            &facade,
            Command::PublishDraft {
                id: stored.id.clone(),
            },
        )
        .await;
        match published {
            Response::DraftPublished {
                draft_id,
                memory_id: _,
                title,
            } => {
                assert_eq!(draft_id, stored.id);
                assert_eq!(title, "Brouillon bridge");
            }
            other => panic!("réponse inattendue: {other:?}"),
        }

        let after = facade.get_draft(&stored.id).await.unwrap();
        assert_eq!(after.status, DraftStatus::Published);

        let stored2 = facade
            .store_draft(MemoryDraft::new("À rejeter", "x"), None)
            .await
            .unwrap();
        let discarded = execute_command(
            &facade,
            Command::DiscardDraft {
                id: stored2.id.clone(),
            },
        )
        .await;
        match discarded {
            Response::DraftDiscarded { id } => assert_eq!(id, stored2.id),
            other => panic!("réponse inattendue: {other:?}"),
        }
        assert_eq!(
            facade.get_draft(&stored2.id).await.unwrap().status,
            DraftStatus::Discarded
        );
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
