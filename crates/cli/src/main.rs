//! CLI Orchestrateur — headless. Daemon WS pour clients visuels Territoire Graphique.

mod harness_ops;
mod output;
mod tui;
mod update;
mod windows_ops;

#[cfg(feature = "http")]
use std::sync::Arc;
use std::path::{Path, PathBuf};

use anyhow::Result;
#[cfg(feature = "http")]
use anyhow::Context;
use clap::{Parser, Subcommand};
use infrastructure::bootstrap_workspace;
use orchestrator::{
    execute_command, BridgeSkillContext, Command, OrchestratorFacade, ProviderKind,
    CapabilityProfileRegistry, ProviderRegistry, Response, SkillsHub, SkillsMarketplace,
};
#[cfg(feature = "gateway")]
use orchestrator::ChannelCatalog;

use harness_ops::{
    channels_disable, channels_enable, channels_status, cmd_configure, cmd_doctor, cmd_harness_run,
    cmd_harness_smoke, cmd_onboard, cmd_uninstall, daemon_install, daemon_status, daemon_stop,
    gateway_status, providers_set, providers_test, ConfigureOptions, OnboardOptions,
};
use update::{cmd_update, UpdateOptions};
use output::print_response;
use tracing_subscriber::EnvFilter;

/// Orchestrateur — second cerveau local souverain (CLI headless).
#[derive(Parser)]
#[command(
    name = "orchestrateur",
    alias = "orchestre",
    alias = "orch",
    version,
    about = "Orchestrateur v0.28.0 — harness intégral Esprit + Cortex",
    after_help = "Alias acceptés : orchestrateur, orchestre, orch"
)]
struct Cli {
    /// Racine du workspace (défaut: ./workspace).
    #[arg(long, global = true, default_value = "workspace")]
    workspace: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Assistant de première installation harness.
    Onboard {
        /// Profil sécurité (`local_only`, `ai_assisted`, …).
        #[arg(long)]
        profile: Option<String>,
        /// Provider LLM primaire.
        #[arg(long)]
        llm: Option<String>,
        /// Raccourci profil local souverain (ollama + zéro egress cloud).
        #[arg(long)]
        local_only: bool,
        /// Installe la tâche planifiée daemon Windows après onboard.
        #[arg(long)]
        install_daemon: bool,
    },
    /// Centre de commande harness (menu interactif).
    Setup,
    /// Configuration harness (menu interactif).
    Settings,
    /// Met à jour des champs harness dans orchestrator.toml.
    Configure {
        #[arg(long)]
        profile: Option<String>,
        #[arg(long)]
        llm: Option<String>,
        #[arg(long)]
        local_only: bool,
    },
    /// Met à jour le binaire (release GitHub ou dev local).
    Update {
        /// Compare version locale vs GitHub sans installer.
        #[arg(long)]
        check: bool,
        /// Force recompilation depuis le dépôt local.
        #[arg(long)]
        dev: bool,
        /// Force installateur release GitHub.
        #[arg(long)]
        release: bool,
    },
    /// Stop sécurité + instructions de désinstallation complète.
    Uninstall,
    /// Santé du service (équivalent `HealthCheck` bridge).
    Health,
    /// Diagnostic harness intégral (Cortex + Esprit + watcher + drafts).
    Doctor,
    /// Liste les mémoires persistées (pagination / filtre).
    List {
        /// Filtre titre ou tags (sous-chaîne).
        #[arg(long)]
        filter: Option<String>,
        /// Décalage pagination.
        #[arg(long, default_value = "0")]
        offset: usize,
        /// Nombre maximal d'éléments.
        #[arg(long, default_value = "100")]
        limit: usize,
    },
    /// Affiche une mémoire par identifiant UUID.
    Get {
        /// Identifiant mémoire.
        id: String,
    },
    /// Recherche sémantique.
    Search {
        /// Requête textuelle.
        query: String,
        /// Nombre maximal de résultats.
        #[arg(long, default_value = "10")]
        limit: usize,
    },
    /// Assimile du texte via le provider LLM configuré.
    Assimilate {
        /// Contenu à assimiler.
        text: String,
        /// Tags suggérés (séparés par virgules).
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
    },
    /// Affiche les statistiques du graphe de connaissances.
    Graph,
    /// Chat libre avec le provider LLM.
    Chat {
        /// Message utilisateur.
        message: String,
    },
    /// Importe des mémoires Markdown depuis un répertoire.
    Import {
        /// Répertoire source (`*.md` récursif).
        #[arg(long)]
        source: PathBuf,
    },
    /// Ré-indexe les embeddings LanceDB pour toutes les mémoires persistées.
    Reindex,
    /// Surveille les fichiers session Markdown et génère des brouillons insight.
    Watch,
    /// Contrôle du watcher de sessions.
    Watcher {
        #[command(subcommand)]
        command: WatcherCommands,
    },
    /// File de brouillons Cortex (gouvernance harness).
    Draft {
        #[command(subcommand)]
        command: DraftCommands,
    },
    /// Validation rapide du harness intégré.
    Harness {
        #[command(subcommand)]
        command: HarnessCommands,
    },
    /// Serveur MCP stdio (expose Cortex + Esprit aux clients externes).
    Mcp {
        #[command(subcommand)]
        command: McpCommands,
    },
    /// Skills opérationnelles (liste et exécution via bridge).
    Skill {
        #[command(subcommand)]
        command: SkillCommands,
    },
    /// Journal d'audit récent (chaîne BLAKE3).
    Audit {
        /// Nombre maximal d'entrées.
        #[arg(long, default_value = "50")]
        limit: usize,
    },
    /// Démarre le daemon HTTP (feature `http`).
    #[cfg(feature = "http")]
    Serve {
        /// Port d'écoute.
        #[arg(long, default_value = "17489")]
        port: u16,
        /// Adresse de liaison.
        #[arg(long, default_value = "127.0.0.1")]
        bind: String,
    },
    /// Daemon WebSocket local pour clients visuels (Territoire Graphique).
    #[cfg(feature = "websocket-server")]
    Daemon {
        #[command(subcommand)]
        command: DaemonCommands,
    },
    /// Gateway WebSocket + canaux messaging (feature `gateway`).
    #[cfg(feature = "gateway")]
    Gateway {
        #[command(subcommand)]
        command: GatewayCommands,
    },
    /// Catalogue des providers LLM / embeddings (Phase 9).
    Providers {
        #[command(subcommand)]
        command: ProviderCommands,
    },
    /// Canaux gateway (Phase 10).
    #[cfg(feature = "gateway")]
    Channels {
        #[command(subcommand)]
        command: ChannelCommands,
    },
    /// Profils de capacités agent — outils Cortex exposés au LLM (Phase 10).
    #[command(name = "capability-profiles")]
    CapabilityProfiles {
        #[command(subcommand)]
        command: CapabilityProfileCommands,
    },
    /// Hub skills — plugins dynamiques (Phase 11).
    SkillsHub {
        #[command(subcommand)]
        command: SkillsHubCommands,
    },
}

#[cfg(feature = "gateway")]
#[derive(Subcommand)]
enum ChannelCommands {
    /// Liste les canaux messaging enregistrés (≥ 15).
    List,
    /// Active un canal dans orchestrator.toml.
    Enable {
        /// Identifiant canal (`telegram`, `discord`, …).
        channel: String,
    },
    /// Désactive un canal.
    Disable {
        /// Identifiant canal.
        channel: String,
    },
    /// Statut enabled + variables d'environnement token.
    Status,
}

#[derive(Subcommand)]
enum CapabilityProfileCommands {
    /// Liste les profils de capacités agent.
    List,
}

#[derive(Subcommand)]
enum SkillsHubCommands {
    /// Liste les skills découvertes (filesystem + inline TOML).
    List,
    /// Affiche le répertoire hub configuré.
    Path,
    /// Liste le catalogue marketplace (local ou distant).
    Marketplace,
    /// Installe les skills du catalogue dans le hub local.
    Sync,
    /// Vérifie les empreintes BLAKE3 des manifestes hub.
    Verify,
}

#[derive(Subcommand)]
enum ProviderCommands {
    /// Liste les providers enregistrés (`llm`, `embedding`, ou tous).
    List {
        /// Filtre : `llm`, `embedding`, ou absent pour tout.
        #[arg(long)]
        kind: Option<String>,
    },
    /// Sonde joignabilité LLM / embedding.
    Test {
        /// Filtre : `llm`, `embedding`, ou absent pour tout.
        #[arg(long)]
        kind: Option<String>,
    },
    /// Définit le provider LLM primaire dans orchestrator.toml.
    Set {
        /// Identifiant provider (`ollama`, `xai`, …).
        provider: String,
    },
}

#[cfg(feature = "websocket-server")]
#[derive(Subcommand)]
enum DaemonCommands {
    /// Démarre le daemon bridge WS (port 28790 par défaut).
    Run {
        /// Port d'écoute (surcharge `orchestrator.toml`).
        #[arg(long)]
        port: Option<u16>,
        /// Adresse de liaison (surcharge `orchestrator.toml`).
        #[arg(long)]
        bind: Option<String>,
    },
    /// Installe la tâche planifiée Windows (démarrage à la connexion).
    Install,
    /// Statut tâche planifiée + sonde HTTP /health.
    Status,
    /// Arrête le daemon (processus + tâche en cours).
    Stop,
}

#[cfg(feature = "gateway")]
#[derive(Subcommand)]
enum GatewayCommands {
    /// Démarre le gateway (WS :28789, webhook, canaux).
    Run {
        /// Port d'écoute (surcharge `orchestrator.toml`).
        #[arg(long)]
        port: Option<u16>,
        /// Adresse de liaison (surcharge `orchestrator.toml`).
        #[arg(long)]
        bind: Option<String>,
    },
    /// Sonde HTTP /health du gateway.
    Status,
}

#[derive(Subcommand)]
enum WatcherCommands {
    /// Affiche le statut du watcher.
    Status,
    /// Démarre le watcher (daemon background si déjà actif).
    Start,
    /// Arrête le watcher global.
    Stop,
}

#[derive(Subcommand)]
enum DraftCommands {
    /// Liste les brouillons en attente.
    List,
    /// Affiche un brouillon par identifiant.
    Get {
        /// Identifiant du brouillon.
        id: String,
    },
    /// Publie un brouillon en mémoire Cortex.
    Publish {
        /// Identifiant du brouillon.
        id: String,
    },
    /// Supprime un brouillon sans publier.
    Discard {
        /// Identifiant du brouillon.
        id: String,
    },
}

#[derive(Subcommand)]
enum HarnessCommands {
    /// Enchaîne health, graph, drafts, watcher (sans LLM obligatoire).
    Smoke,
    /// Démarre daemon + gateway si absents, attend Ctrl+C.
    Run,
}

#[derive(Subcommand)]
enum McpCommands {
    /// Serveur MCP JSON-RPC sur stdin/stdout.
    Serve,
}

#[derive(Subcommand)]
enum SkillCommands {
    /// Liste les skills enregistrées.
    List,
    /// Exécute une skill par son nom.
    Run {
        /// Identifiant (`list_memories`, `search`, `assimilate`, …).
        name: String,
        /// Requête (`search`).
        #[arg(long)]
        query: Option<String>,
        /// Texte (`assimilate`).
        #[arg(long)]
        text: Option<String>,
        /// Tags (filtre ou contexte).
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
        /// Limite de résultats (`search`).
        #[arg(long)]
        limit: Option<usize>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("orchestrateur=info".parse()?))
        .init();

    let cli = Cli::parse();

    if let Some(()) = dispatch_lightweight(&cli).await? {
        return Ok(());
    }

    let facade = bootstrap_facade(&cli.workspace).await?;

    match cli.command {
        Commands::Onboard { .. }
        | Commands::Configure { .. }
        | Commands::Setup
        | Commands::Settings
        | Commands::Update { .. }
        | Commands::Uninstall
        | Commands::Harness {
            command: HarnessCommands::Run,
        } => unreachable!("géré par dispatch_lightweight"),
        Commands::Health => run_bridge_command(&facade, Command::HealthCheck).await?,
        Commands::Doctor => cmd_doctor(&facade, &cli.workspace).await?,
        Commands::List {
            filter,
            offset,
            limit,
        } => {
            run_bridge_command(
                &facade,
                Command::List {
                    filter,
                    offset,
                    limit,
                },
            )
            .await?;
        }
        Commands::Get { id } => {
            run_bridge_command(&facade, Command::GetMemory { id }).await?;
        }
        Commands::Search { query, limit } => {
            run_bridge_command(&facade, Command::Search { query, limit }).await?;
        }
        Commands::Assimilate { text, tags } => {
            let response =
                execute_command(&facade, Command::Assimilate { text, tags }).await;
            match response {
                Response::Assimilated { memory_id, title } => {
                    println!("Assimilé : {title} ({memory_id})");
                }
                Response::MemoryDetail { memory } => {
                    println!("Assimilé : {} ({})", memory.title, memory.id);
                    println!("Backlinks : {}", memory.backlink_count());
                }
                Response::Error(err) => {
                    anyhow::bail!("[{}] {}", err.kind, err.message);
                }
                other => print_response(other)?,
            }
        }
        Commands::Graph => run_bridge_command(&facade, Command::Graph).await?,
        Commands::Chat { message } => {
            run_bridge_command(&facade, Command::Chat { message }).await?;
        }
        Commands::Audit { limit } => {
            run_bridge_command(&facade, Command::Audit { limit }).await?;
        }
        Commands::Skill { command } => match command {
            SkillCommands::List => run_bridge_command(&facade, Command::ListSkills).await?,
            SkillCommands::Run {
                name,
                query,
                text,
                tags,
                limit,
            } => {
                run_bridge_command(
                    &facade,
                    Command::ExecuteSkill {
                        name,
                        context: BridgeSkillContext {
                            query,
                            text,
                            tags,
                            limit,
                        },
                    },
                )
                .await?;
            }
        },
        Commands::Import { source } => cmd_import(&facade, &source).await?,
        Commands::Reindex => cmd_reindex(&facade).await?,
        Commands::Watch => cmd_watch(&facade).await?,
        Commands::Watcher { command } => match command {
            WatcherCommands::Status => {
                run_bridge_command(&facade, Command::WatcherStatus).await?
            }
            WatcherCommands::Start => {
                run_bridge_command(&facade, Command::WatcherStart).await?
            }
            WatcherCommands::Stop => run_bridge_command(&facade, Command::WatcherStop).await?,
        },
        Commands::Draft { command } => match command {
            DraftCommands::List => run_bridge_command(&facade, Command::ListDrafts).await?,
            DraftCommands::Get { id } => {
                run_bridge_command(&facade, Command::GetDraft { id }).await?
            }
            DraftCommands::Publish { id } => {
                run_bridge_command(&facade, Command::PublishDraft { id }).await?
            }
            DraftCommands::Discard { id } => {
                run_bridge_command(&facade, Command::DiscardDraft { id }).await?
            }
        },
        Commands::Harness { command } => match command {
            HarnessCommands::Smoke => cmd_harness_smoke(&facade).await?,
            HarnessCommands::Run => unreachable!("géré par dispatch_lightweight"),
        },
        Commands::Mcp { command } => match command {
            McpCommands::Serve => {
                use std::sync::Arc;
                mcp::run_stdio_server(Arc::new(facade))
                    .await
                    .map_err(|e| anyhow::anyhow!("{e}"))?
            }
        },
        #[cfg(feature = "http")]
        Commands::Serve { port, bind } => run_http_server(facade, &bind, port).await?,
        #[cfg(feature = "websocket-server")]
        Commands::Daemon { command } => match command {
            DaemonCommands::Run { port, bind } => {
                run_daemon_server(&cli.workspace, port, bind).await?
            }
            DaemonCommands::Install | DaemonCommands::Status | DaemonCommands::Stop => {
                unreachable!("géré par dispatch_lightweight")
            }
        },
        #[cfg(feature = "gateway")]
        Commands::Gateway { command } => match command {
            GatewayCommands::Run { port, bind } => {
                run_gateway_server(facade, &cli.workspace, port, bind).await?
            }
            GatewayCommands::Status => unreachable!("géré par dispatch_lightweight"),
        },
        Commands::Providers { command } => match command {
            ProviderCommands::List { kind: _ } => unreachable!("géré par dispatch_lightweight"),
            ProviderCommands::Test { kind } => providers_test(&facade, kind.as_deref()).await?,
            ProviderCommands::Set { provider: _ } => unreachable!("géré par dispatch_lightweight"),
        },
        #[cfg(feature = "gateway")]
        Commands::Channels { command } => match command {
            ChannelCommands::List
            | ChannelCommands::Enable { .. }
            | ChannelCommands::Disable { .. }
            | ChannelCommands::Status => unreachable!("géré par dispatch_lightweight"),
        },
        Commands::CapabilityProfiles { command } => match command {
            CapabilityProfileCommands::List => cmd_capability_profiles_list()?,
        },
        Commands::SkillsHub { command } => match command {
            SkillsHubCommands::List => cmd_skills_hub_list(&facade)?,
            SkillsHubCommands::Path => cmd_skills_hub_path(&facade)?,
            SkillsHubCommands::Marketplace => cmd_skills_hub_marketplace(&facade).await?,
            SkillsHubCommands::Sync => cmd_skills_hub_sync(&facade).await?,
            SkillsHubCommands::Verify => cmd_skills_hub_verify(&facade)?,
        },
    }
    Ok(())
}

/// Commandes harness sans bootstrap Cortex (config / OS / HTTP probes).
async fn dispatch_lightweight(cli: &Cli) -> Result<Option<()>> {
    match &cli.command {
        Commands::Setup => {
            tui::run_setup(&cli.workspace)?;
            return Ok(Some(()));
        }
        Commands::Settings => {
            tui::run_settings(&cli.workspace)?;
            return Ok(Some(()));
        }
        Commands::Update { check, dev, release } => {
            cmd_update(UpdateOptions {
                check: *check,
                dev: *dev,
                release: *release,
            })
            .await?;
            return Ok(Some(()));
        }
        Commands::Uninstall => {
            cmd_uninstall()?;
            return Ok(Some(()));
        }
        Commands::Onboard {
            profile,
            llm,
            local_only,
            install_daemon,
        } => {
            let opts = OnboardOptions {
                profile: profile.clone(),
                llm: llm.clone(),
                local_only: *local_only,
                install_daemon: *install_daemon,
            };
            if opts.profile.is_none()
                && opts.llm.is_none()
                && !opts.local_only
                && !opts.install_daemon
            {
                tui::run_onboard_wizard(&cli.workspace)?;
            } else {
                cmd_onboard(&cli.workspace, &opts)?;
            }
            return Ok(Some(()));
        }
        Commands::Configure {
            profile,
            llm,
            local_only,
        } => {
            cmd_configure(
                &cli.workspace,
                &ConfigureOptions {
                    profile: profile.clone(),
                    llm: llm.clone(),
                    local_only: *local_only,
                },
            )?;
            return Ok(Some(()));
        }
        Commands::Harness {
            command: HarnessCommands::Run,
        } => {
            cmd_harness_run(&cli.workspace).await?;
            return Ok(Some(()));
        }
        Commands::Providers { command } => match command {
            ProviderCommands::List { kind } => {
                cmd_providers_list(kind.as_deref())?;
                return Ok(Some(()));
            }
            ProviderCommands::Set { provider } => {
                providers_set(&cli.workspace, provider)?;
                return Ok(Some(()));
            }
            ProviderCommands::Test { .. } => {}
        },
        Commands::CapabilityProfiles {
            command: CapabilityProfileCommands::List,
        } => {
            cmd_capability_profiles_list()?;
            return Ok(Some(()));
        }
        #[cfg(feature = "gateway")]
        Commands::Channels { command } => match command {
            ChannelCommands::List => {
                cmd_channels_list()?;
                return Ok(Some(()));
            }
            ChannelCommands::Enable { channel } => {
                channels_enable(&cli.workspace, channel)?;
                return Ok(Some(()));
            }
            ChannelCommands::Disable { channel } => {
                channels_disable(&cli.workspace, channel)?;
                return Ok(Some(()));
            }
            ChannelCommands::Status => {
                channels_status(&cli.workspace)?;
                return Ok(Some(()));
            }
        },
        #[cfg(feature = "gateway")]
        Commands::Gateway {
            command: GatewayCommands::Status,
        } => {
            gateway_status(&cli.workspace).await?;
            return Ok(Some(()));
        },
        #[cfg(feature = "websocket-server")]
        Commands::Daemon { command } => match command {
            DaemonCommands::Install => {
                daemon_install(&cli.workspace)?;
                return Ok(Some(()));
            }
            DaemonCommands::Status => {
                daemon_status(&cli.workspace).await?;
                return Ok(Some(()));
            }
            DaemonCommands::Stop => {
                daemon_stop()?;
                return Ok(Some(()));
            }
            DaemonCommands::Run { .. } => {}
        },
        _ => {}
    }
    Ok(None)
}

async fn bootstrap_facade(workspace: &Path) -> Result<OrchestratorFacade> {
    let deps = bootstrap_workspace(workspace)
        .await
        .map_err(|err| anyhow::anyhow!(err.with_context("CLI")))?;
    Ok(OrchestratorFacade::new(deps))
}

async fn run_bridge_command(facade: &OrchestratorFacade, command: Command) -> Result<()> {
    let response = execute_command(facade, command).await;
    print_response(response)
}

async fn cmd_watch(facade: &OrchestratorFacade) -> Result<()> {
    use std::sync::Arc;

    use orchestrator::watcher::{install_global, SessionWatcherHandle};

    let config = &facade.deps().config;
    if !config.watcher.enabled {
        anyhow::bail!("watcher désactivé — activez [watcher] enabled = true dans orchestrator.toml");
    }

    let handle = Arc::new(SessionWatcherHandle::new(
        Arc::new(OrchestratorFacade::new(facade.deps().clone())),
        None::<orchestrator::watcher::DraftReadyCallback>,
        config,
    ));
    let watch_dirs = handle.status().await.watch_dirs;
    install_global(Arc::clone(&handle));
    Arc::clone(&handle).spawn();

    println!(
        "Watcher actif — Ctrl+C pour arrêter. Répertoires : {watch_dirs:?}"
    );

    tokio::signal::ctrl_c()
        .await
        .map_err(|e| anyhow::anyhow!("signal ctrl+c: {e}"))?;
    handle.stop();
    println!("Watcher arrêté.");
    Ok(())
}

async fn cmd_import(facade: &OrchestratorFacade, source: &Path) -> Result<()> {
    let result = facade.import_from_directory(source).await?;
    println!(
        "Import terminé : {} importée(s), {} ignorée(s), {} erreur(s)",
        result.imported,
        result.skipped,
        result.errors.len()
    );
    for err in &result.errors {
        eprintln!("  erreur: {err}");
    }
    Ok(())
}

async fn cmd_reindex(facade: &OrchestratorFacade) -> Result<()> {
    let memories = facade.list_memories().await?;
    let total = memories.len();
    if total == 0 {
        println!("Reindex terminé : 0 mémoire.");
        return Ok(());
    }

    let mut ok = 0usize;
    let mut errors = Vec::new();
    for (i, memory) in memories.iter().enumerate() {
        match facade.save_memory(memory).await {
            Ok(_) => {
                ok += 1;
                println!(
                    "[{}/{}] indexé : {} ({})",
                    i + 1,
                    total,
                    memory.title,
                    memory.id
                );
            }
            Err(err) => {
                errors.push(format!("{}: {err}", memory.id));
                eprintln!("[{}/{}] erreur : {} — {err}", i + 1, total, memory.id);
            }
        }
    }

    println!(
        "Reindex terminé : {ok}/{total} indexée(s), {} erreur(s)",
        errors.len()
    );
    Ok(())
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
        .context("variable ORCHESTRATEUR_DAEMON_TOKEN requise pour le daemon HTTP")?;

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
            .is_some_and(|provided| {
                constant_time_eq(provided.as_bytes(), state.token.as_bytes())
            });

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

fn cmd_skills_hub_list(facade: &OrchestratorFacade) -> Result<()> {
    let config = &facade.deps().config;
    let entries = SkillsHub::discover(config).map_err(anyhow::Error::msg)?;
    if entries.is_empty() {
        println!("Aucune skill découverte dans le hub.");
        return Ok(());
    }
    println!("# Skills hub ({})", entries.len());
    for entry in entries {
        let state = if entry.enabled { "on" } else { "off" };
        println!(
            "{:<16} {:<10} {:<10} {:<6} {} — {}",
            entry.id,
            entry.kind,
            entry.origin,
            state,
            entry.version,
            entry.description
        );
        println!("  path: {}", entry.path.display());
    }
    Ok(())
}

fn cmd_skills_hub_path(facade: &OrchestratorFacade) -> Result<()> {
    let path = facade.deps().config.skills_hub_dir();
    println!("{}", path.display());
    Ok(())
}

async fn cmd_skills_hub_marketplace(facade: &OrchestratorFacade) -> Result<()> {
    let config = &facade.deps().config;
    let catalog = SkillsMarketplace::load_catalog_auto(config)
        .await
        .map_err(anyhow::Error::msg)?;
    println!("# Marketplace v{} ({} skills)", catalog.version, catalog.skills.len());
    for entry in &catalog.skills {
        let state = if entry.enabled { "on" } else { "off" };
        println!(
            "{:<16} {:<6} {} — {}",
            entry.id, state, entry.version, entry.description
        );
    }
    Ok(())
}

async fn cmd_skills_hub_sync(facade: &OrchestratorFacade) -> Result<()> {
    let config = &facade.deps().config;
    let catalog = SkillsMarketplace::load_catalog_auto(config)
        .await
        .map_err(anyhow::Error::msg)?;
    let result = SkillsMarketplace::sync_to_hub(config, &catalog).map_err(anyhow::Error::msg)?;
    println!(
        "Sync terminé : {} installée(s), {} ignorée(s)",
        result.installed.len(),
        result.skipped.len()
    );
    for id in &result.installed {
        println!("  + {id}");
    }
    Ok(())
}

fn cmd_skills_hub_verify(facade: &OrchestratorFacade) -> Result<()> {
    let report = SkillsMarketplace::verify_hub_integrity(&facade.deps().config)
        .map_err(anyhow::Error::msg)?;
    println!(
        "Vérification intégrité : {} valide(s), {} invalide(s)",
        report.valid.len(),
        report.invalid.len()
    );
    for path in &report.valid {
        println!("  ok  {}", path.display());
    }
    for (path, err) in &report.invalid {
        println!("  ERR {} — {err}", path.display());
    }
    if !report.invalid.is_empty() {
        anyhow::bail!("manifestes invalides détectés");
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
        Some(other) => anyhow::bail!("kind inconnu: {other} (utiliser llm ou embedding)"),
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

#[cfg(feature = "websocket-server")]
async fn run_daemon_server(
    workspace: &Path,
    port: Option<u16>,
    bind: Option<String>,
) -> Result<()> {
    use std::sync::Arc;

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
        anyhow::bail!("daemon désactivé dans orchestrator.toml ([daemon] enabled = false)");
    }

    tracing::info!(
        bind = %config.daemon.bind,
        port = config.daemon.port,
        "démarrage daemon — définir {} pour l'authentification WS",
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
    use std::sync::Arc;

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
        anyhow::bail!("gateway désactivé dans orchestrator.toml ([gateway] enabled = false)");
    }

    tracing::info!(
        bind = %config.gateway.bind,
        port = config.gateway.port,
        "démarrage gateway — définir {} pour l'authentification WS",
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