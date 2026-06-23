//! Routage des commandes CLI.

use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use orchestrator::{
    Command, OrchestratorFacade, ProviderKind, CapabilityProfileRegistry, ProviderRegistry,
};
#[cfg(feature = "gateway")]
use orchestrator::ChannelCatalog;

use crate::cli::{
    CapabilityProfileCommands, ChannelCommands, Commands, DraftCommands, GatewayCommands,
    HarnessCommands, McpCommands, ProviderCommands, SkillsHubCommands, WatcherCommands,
};
use crate::cli::Cli;
use crate::commands::{
    agent, b212, config, daemon, doctor, health, memory, onboard, session, skill, uninstall,
    update, DaemonCommands, MemoryCommands, SkillCommands,
};
use crate::context::{bootstrap_facade, run_bridge_command};
use orchestrator::{ConfigureOptions, HarnessSmokeOptions};
use crate::present::{
    self, channels_disable, channels_enable, channels_status, gateway_status, harness_run,
    harness_smoke, providers_set, providers_test,
};
use crate::tui;

/// Point d'entrée du dispatch (après parse clap).
pub async fn run(cli: Cli) -> Result<()> {
    if let Some(()) = dispatch_lightweight(&cli).await? {
        return Ok(());
    }

    let facade = bootstrap_facade(&cli.workspace).await?;
    dispatch_facade(&cli, &facade).await
}

async fn dispatch_lightweight(cli: &Cli) -> Result<Option<()>> {
    match &cli.command {
        Commands::Setup => {
            tui::run_setup(&cli.workspace)?;
            Ok(Some(()))
        }
        Commands::Settings => {
            tui::run_settings(&cli.workspace)?;
            Ok(Some(()))
        }
        Commands::Onboard(args) => {
            onboard::run(args.clone(), &cli.workspace)?;
            Ok(Some(()))
        }
        Commands::Update(args) => {
            update::run(args.clone()).await?;
            Ok(Some(()))
        }
        Commands::Uninstall => {
            uninstall::run()?;
            Ok(Some(()))
        }
        Commands::Configure {
            profile,
            llm,
            local_only,
        } => {
            present::configure(
                &cli.workspace,
                &ConfigureOptions {
                    profile: profile.clone(),
                    llm: llm.clone(),
                    local_only: *local_only,
                },
            )?;
            Ok(Some(()))
        }
        Commands::Config { command } => {
            config::run(command.clone(), &cli.workspace)?;
            Ok(Some(()))
        }
        Commands::Doctor => Ok(None),
        Commands::Health => Ok(None),
        Commands::Harness {
            command: HarnessCommands::Run,
        } => {
            harness_run(&cli.workspace).await?;
            Ok(Some(()))
        }
        Commands::Daemon { command } => daemon::run(command.clone(), &cli.workspace).await,
        Commands::Session { command } => {
            session::run(command.clone(), &cli.workspace).await?;
            Ok(Some(()))
        }
        Commands::Agent { command } => {
            agent::run(command.clone(), &cli.workspace).await?;
            Ok(Some(()))
        }
        Commands::B212 { command } => {
            b212::run(command.clone(), &cli.workspace).await?;
            Ok(Some(()))
        }
        Commands::Providers { command } => match command {
            ProviderCommands::List { kind } => {
                cmd_providers_list(kind.as_deref())?;
                Ok(Some(()))
            }
            ProviderCommands::Set { provider } => {
                providers_set(&cli.workspace, provider)?;
                Ok(Some(()))
            }
            ProviderCommands::Test { .. } => Ok(None),
        },
        Commands::CapabilityProfiles {
            command: CapabilityProfileCommands::List,
        } => {
            cmd_capability_profiles_list()?;
            Ok(Some(()))
        }
        #[cfg(feature = "gateway")]
        Commands::Channels { command } => match command {
            ChannelCommands::List => {
                cmd_channels_list()?;
                Ok(Some(()))
            }
            ChannelCommands::Enable { channel } => {
                channels_enable(&cli.workspace, channel)?;
                Ok(Some(()))
            }
            ChannelCommands::Disable { channel } => {
                channels_disable(&cli.workspace, channel)?;
                Ok(Some(()))
            }
            ChannelCommands::Status => {
                channels_status(&cli.workspace)?;
                Ok(Some(()))
            }
        },
        #[cfg(feature = "gateway")]
        Commands::Gateway {
            command: GatewayCommands::Status,
        } => {
            gateway_status(&cli.workspace).await?;
            Ok(Some(()))
        }
        _ => Ok(None),
    }
}

async fn dispatch_facade(cli: &Cli, facade: &OrchestratorFacade) -> Result<()> {
    match &cli.command {
        Commands::Doctor => doctor::run(facade, &cli.workspace).await?,
        Commands::Health => health::run(facade).await?,
        Commands::Memory { command } => memory::run(command.clone(), facade).await?,
        Commands::Skill { command } => skill::run(command.clone(), facade).await?,
        Commands::Chat { message } => {
            run_bridge_command(facade, Command::Chat { message: message.clone() }).await?
        }
        Commands::Audit { limit } => {
            run_bridge_command(facade, Command::Audit { limit: *limit }).await?
        }
        Commands::List {
            filter,
            offset,
            limit,
        } => {
            memory::run(
                MemoryCommands::List {
                    filter: filter.clone(),
                    offset: *offset,
                    limit: *limit,
                },
                facade,
            )
            .await?
        }
        Commands::Get { id } => {
            memory::run(MemoryCommands::Show { id: id.clone() }, facade).await?
        }
        Commands::Search { query, limit } => {
            memory::run(
                MemoryCommands::Search {
                    query: query.clone(),
                    limit: *limit,
                },
                facade,
            )
            .await?
        }
        Commands::Assimilate { text, tags } => {
            memory::run(
                MemoryCommands::Assimilate {
                    text: text.clone(),
                    tags: tags.clone(),
                },
                facade,
            )
            .await?
        }
        Commands::Graph => memory::run(MemoryCommands::Graph, facade).await?,
        Commands::Import { source } => {
            memory::run(MemoryCommands::Import { source: source.clone() }, facade).await?
        }
        Commands::Reindex => memory::run(MemoryCommands::Reindex, facade).await?,
        Commands::Watch => cmd_watch(facade).await?,
        Commands::Watcher { command } => match command {
            WatcherCommands::Status => {
                run_bridge_command(facade, Command::WatcherStatus).await?
            }
            WatcherCommands::Start => {
                run_bridge_command(facade, Command::WatcherStart).await?
            }
            WatcherCommands::Stop => run_bridge_command(facade, Command::WatcherStop).await?,
        },
        Commands::Draft { command } => match command {
            DraftCommands::List => run_bridge_command(facade, Command::ListDrafts).await?,
            DraftCommands::Get { id } => {
                run_bridge_command(facade, Command::GetDraft { id: id.clone() }).await?
            }
            DraftCommands::Publish { id } => {
                run_bridge_command(facade, Command::PublishDraft { id: id.clone() }).await?
            }
            DraftCommands::Discard { id } => {
                run_bridge_command(facade, Command::DiscardDraft { id: id.clone() }).await?
            }
        },
        Commands::Harness { command } => match command {
            HarnessCommands::Smoke {
                skip_gateway,
                skip_chat,
            } => {
                harness_smoke(
                    facade,
                    &cli.workspace,
                    &HarnessSmokeOptions {
                        skip_gateway: *skip_gateway,
                        skip_chat: *skip_chat,
                    },
                )
                .await?
            }
            HarnessCommands::Run => unreachable!("lightweight"),
        },
        Commands::Mcp { command } => match command {
            McpCommands::Serve => {
                let owned = Arc::new(OrchestratorFacade::new(facade.deps().clone()));
                mcp::run_stdio_server(owned)
                    .await
                    .map_err(|e| anyhow::anyhow!("{e}"))?
            }
        },
        Commands::Providers { command } => match command {
            ProviderCommands::Test { kind } => {
                providers_test(facade, kind.as_deref()).await?
            }
            ProviderCommands::List { .. } | ProviderCommands::Set { .. } => {
                unreachable!("lightweight")
            }
        },
        Commands::SkillsHub { command } => match command {
            SkillsHubCommands::List => skill::list_hub(facade)?,
            SkillsHubCommands::Path => {
                println!("{}", facade.deps().config.skills_hub_dir().display());
            }
            SkillsHubCommands::Marketplace => {
                skill::run(SkillCommands::Marketplace, facade).await?
            }
            SkillsHubCommands::Sync => skill::run(SkillCommands::Update, facade).await?,
            SkillsHubCommands::Verify => skill::run(SkillCommands::Verify, facade).await?,
        },
        #[cfg(feature = "http")]
        Commands::Serve { port, bind } => {
            run_http_server(OrchestratorFacade::new(facade.deps().clone()), bind, *port).await?
        }
        #[cfg(feature = "websocket-server")]
        Commands::Daemon { command } => match command {
            DaemonCommands::Start { port, bind } | DaemonCommands::Run { port, bind } => {
                run_daemon_server(&cli.workspace, *port, bind.clone()).await?
            }
            DaemonCommands::Stop
            | DaemonCommands::Status
            | DaemonCommands::Install
            | DaemonCommands::Restart => unreachable!("lightweight"),
        },
        #[cfg(feature = "gateway")]
        Commands::Gateway { command } => match command {
            GatewayCommands::Run { port, bind } => {
                run_gateway_server(
                    OrchestratorFacade::new(facade.deps().clone()),
                    &cli.workspace,
                    *port,
                    bind.clone(),
                )
                .await?
            }
            GatewayCommands::Status => unreachable!("lightweight"),
        },
        #[cfg(feature = "gateway")]
        Commands::Channels { .. } => unreachable!("lightweight"),
        Commands::CapabilityProfiles { .. } => unreachable!("lightweight"),
        Commands::Onboard(_)
        | Commands::Update(_)
        | Commands::Uninstall
        | Commands::Setup
        | Commands::Settings
        | Commands::Configure { .. }
        | Commands::Config { .. }
        | Commands::Session { .. }
        | Commands::Agent { .. }
        | Commands::B212 { .. } => unreachable!("lightweight"),
    }
    Ok(())
}

async fn cmd_watch(facade: &OrchestratorFacade) -> Result<()> {
    use orchestrator::watcher::{install_global, SessionWatcherHandle};

    let config = &facade.deps().config;
    if !config.watcher.enabled {
        anyhow::bail!("watcher désactivé — [watcher] enabled = true dans orchestrator.toml");
    }

    let handle = Arc::new(SessionWatcherHandle::new(
        Arc::new(OrchestratorFacade::new(facade.deps().clone())),
        None::<orchestrator::watcher::DraftReadyCallback>,
        config,
    ));
    let watch_dirs = handle.status().await.watch_dirs;
    install_global(Arc::clone(&handle));
    Arc::clone(&handle).spawn();

    println!("Watcher actif — Ctrl+C pour arrêter. Répertoires : {watch_dirs:?}");

    tokio::signal::ctrl_c()
        .await
        .map_err(|e| anyhow::anyhow!("signal ctrl+c: {e}"))?;
    handle.stop();
    println!("Watcher arrêté.");
    Ok(())
}

#[cfg(feature = "gateway")]
fn cmd_channels_list() -> Result<()> {
    let catalog = ChannelCatalog::new();
    println!("# Canaux gateway ({})", catalog.count());
    for channel in catalog.descriptors() {
        let kind = if channel.dedicated { "dedicated" } else { "stub" };
        println!(
            "{:<14} {:<10} {:<28} env={}",
            channel.id, kind, channel.display_name, channel.default_token_env
        );
    }
    Ok(())
}

fn cmd_capability_profiles_list() -> Result<()> {
    let registry = CapabilityProfileRegistry::new();
    for profile in registry.descriptors() {
        let tools = if profile.tools.is_empty() {
            "(tous)".to_string()
        } else {
            profile.tools.join(", ")
        };
        println!("{:<10} {:<22} [{tools}]", profile.id, profile.display_name);
    }
    Ok(())
}

fn cmd_providers_list(kind: Option<&str>) -> Result<()> {
    let registry = ProviderRegistry::new();
    match kind {
        Some("llm") => print_provider_table(registry.llm_descriptors()),
        Some("embedding") => print_provider_table(registry.embedding_descriptors()),
        Some(other) => anyhow::bail!("kind inconnu: {other} (llm ou embedding)"),
        None => {
            println!("# LLM ({} providers)", registry.llm_descriptors().len());
            print_provider_table(registry.llm_descriptors());
            println!();
            println!(
                "# Embeddings ({} providers)",
                registry.embedding_descriptors().len()
            );
            print_provider_table(registry.embedding_descriptors());
        }
    }
    Ok(())
}

fn print_provider_table(descriptors: &[orchestrator::ProviderDescriptor]) {
    for d in descriptors {
        let kind = match d.kind {
            ProviderKind::Llm => "llm",
            ProviderKind::Embedding => "embedding",
        };
        println!(
            "{:<14} {:<12} {:<24} model={} env={}",
            d.id, kind, d.display_name, d.default_model, d.default_api_key_env
        );
    }
}

#[cfg(feature = "http")]
async fn run_http_server(facade: OrchestratorFacade, bind: &str, port: u16) -> Result<()> {
    use axum::{
        extract::State,
        http::{header::AUTHORIZATION, StatusCode},
        response::IntoResponse,
        routing::post,
        Json, Router,
    };
    use tower_http::trace::TraceLayer;

    struct HttpState {
        facade: OrchestratorFacade,
        token: String,
    }

    let token = std::env::var("ORCHESTRATEUR_DAEMON_TOKEN")
        .context("variable ORCHESTRATEUR_DAEMON_TOKEN requise")?;

    let state = Arc::new(HttpState { facade, token });

    async fn execute_handler(
        State(state): State<Arc<HttpState>>,
        headers: axum::http::HeaderMap,
        Json(cmd): Json<Command>,
    ) -> impl IntoResponse {
        let authorized = headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .is_some_and(|provided| constant_time_eq(provided.as_bytes(), state.token.as_bytes()));

        if !authorized {
            return (
                StatusCode::UNAUTHORIZED,
                Json(Response::Error(orchestrator::AppError {
                    kind: "auth".into(),
                    message: "Bearer token invalide ou absent".into(),
                })),
            );
        }

        let response = execute_command(&state.facade, cmd).await;
        (StatusCode::OK, Json(response))
    }

    let app = Router::new()
        .route("/v1/execute", post(execute_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("{bind}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("écoute sur {addr}"))?;
    tracing::info!(%addr, "daemon HTTP démarré");
    axum::serve(listener, app)
        .await
        .context("serveur HTTP interrompu")?;
    Ok(())
}

#[cfg(feature = "websocket-server")]
async fn run_daemon_server(
    workspace: &Path,
    port: Option<u16>,
    bind: Option<String>,
) -> Result<()> {
    use infrastructure::bootstrap_workspace;
    use orchestrator::{
        run_daemon_with_domain_events, EventPublisher, FanoutEventPublisher, OrchestratorConfig,
    };

    let mut deps = bootstrap_workspace(workspace)
        .await
        .map_err(|e| anyhow::anyhow!("bootstrap: {e}"))?;
    let fanout = FanoutEventPublisher::new();
    let domain_rx = fanout.subscribe();
    let events: Arc<dyn EventPublisher> = Arc::new(fanout);
    deps.events = events;
    let facade = Arc::new(OrchestratorFacade::new(deps));

    let mut config = OrchestratorConfig::load_workspace(workspace)
        .map_err(|e| anyhow::anyhow!("config: {e}"))?;
    if let Some(p) = port {
        config.daemon.port = p;
    }
    if let Some(b) = bind {
        config.daemon.bind = b;
    }
    if !config.daemon.enabled {
        anyhow::bail!("daemon désactivé ([daemon] enabled = false)");
    }

    tracing::info!(
        bind = %config.daemon.bind,
        port = config.daemon.port,
        "daemon WS — définir {} pour l'auth",
        config.daemon.token_env
    );

    run_daemon_with_domain_events(facade, &config.daemon, Some(domain_rx))
        .await
        .map_err(|e| anyhow::anyhow!("daemon: {e}"))?;
    Ok(())
}

#[cfg(feature = "gateway")]
async fn run_gateway_server(
    facade: OrchestratorFacade,
    workspace: &Path,
    port: Option<u16>,
    bind: Option<String>,
) -> Result<()> {
    use orchestrator::{run_gateway, OrchestratorConfig};

    let mut config = OrchestratorConfig::load_workspace(workspace)
        .map_err(|e| anyhow::anyhow!("config: {e}"))?;
    if let Some(p) = port {
        config.gateway.port = p;
    }
    if let Some(b) = bind {
        config.gateway.bind = b;
    }
    if !config.gateway.enabled {
        anyhow::bail!("gateway désactivé ([gateway] enabled = false)");
    }

    tracing::info!(
        bind = %config.gateway.bind,
        port = config.gateway.port,
        "gateway — définir {} pour l'auth",
        config.gateway.token_env
    );

    run_gateway(Arc::new(facade), &config)
        .await
        .map_err(|e| anyhow::anyhow!("gateway: {e}"))?;
    Ok(())
}

#[cfg(feature = "http")]
fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut diff = 0u8;
    for (a, b) in left.iter().zip(right.iter()) {
        diff |= a ^ b;
    }
    diff == 0
}