//! CLI Orchestrateur — headless (sans TUI). Interface terminal : voir `orchestrateur-tui`.

#[cfg(feature = "http")]
use std::sync::Arc;
use std::path::{Path, PathBuf};

use anyhow::Result;
#[cfg(feature = "http")]
use anyhow::Context;
use clap::{Parser, Subcommand};
use infrastructure::bootstrap_workspace;
use orchestrator::{
    execute_command, BridgeSkillContext, Command, OrchestratorFacade, Response,
};

use tracing_subscriber::EnvFilter;

/// Orchestrateur — second cerveau local souverain (CLI headless).
#[derive(Parser)]
#[command(
    name = "orchestrateur",
    version,
    about = "Orchestrateur v0.6.0 — CLI headless (TUI : orchestrateur-tui)"
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
    /// Santé du service (équivalent `HealthCheck` bridge).
    Health,
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

    let deps = bootstrap_workspace(&cli.workspace)
        .await
        .map_err(|err| anyhow::anyhow!(err.with_context("CLI")))?;
    let facade = OrchestratorFacade::new(deps);

    match cli.command {
        Commands::Health => run_bridge_command(&facade, Command::HealthCheck).await?,
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
        #[cfg(feature = "http")]
        Commands::Serve { port, bind } => run_http_server(facade, &bind, port).await?,
    }
    Ok(())
}

async fn run_bridge_command(facade: &OrchestratorFacade, command: Command) -> Result<()> {
    let response = execute_command(facade, command).await;
    print_response(response)
}

fn print_response(response: Response) -> Result<()> {
    match response {
        Response::Health {
            status,
            version,
            llm_available,
            embedding_available,
        } => {
            println!(
                "status={status} version={version} llm={llm_available} embedding={embedding_available}"
            );
        }
        Response::MemoryList { items, total } => {
            if items.is_empty() {
                println!("Aucune mémoire (total={total}).");
                return Ok(());
            }
            println!("# total={total}");
            for item in items {
                let tags = item.tags.join(", ");
                println!("{} | {} | tags=[{tags}]", item.id, item.title);
            }
        }
        Response::MemoryDetail { memory } => {
            println!("# {}", memory.title);
            println!("id: {}", memory.id);
            if !memory.tags.is_empty() {
                let tags: Vec<_> = memory.tags.iter().map(|t| t.as_str()).collect();
                println!("tags: {}", tags.join(", "));
            }
            println!("---");
            println!("{}", memory.content);
        }
        Response::SearchResults { items } => {
            if items.is_empty() {
                println!("Aucun résultat.");
                return Ok(());
            }
            for hit in items {
                let preview: String = hit
                    .snippet
                    .as_deref()
                    .unwrap_or("")
                    .chars()
                    .take(120)
                    .collect();
                println!("{:.3} | {} | {}", hit.score, hit.memory_id, preview);
            }
        }
        Response::Assimilated { memory_id, title } => {
            println!("Assimilé : {title} ({memory_id})");
        }
        Response::GraphSummary {
            node_count,
            edge_count,
            hubs,
        } => {
            println!("Nœuds : {node_count}");
            println!("Arêtes : {edge_count}");
            for hub in hubs {
                println!(
                    "  hub ({}) : {} [{}]",
                    hub.inbound_links, hub.title, hub.memory_id
                );
            }
        }
        Response::AuditLog {
            entries,
            chain_intact,
        } => {
            let status = if chain_intact { "intacte" } else { "ROMPUE" };
            println!("Chaîne d'audit : {status}");
            for entry in entries {
                println!(
                    "{} | {} | {} | {}",
                    entry.timestamp, entry.event_type, entry.details, entry.hash
                );
            }
        }
        Response::Error(err) => {
            anyhow::bail!("[{}] {}", err.kind, err.message);
        }
        Response::Success { message } => {
            println!("{message}");
        }
        Response::ChatReply { reply } => {
            println!("{reply}");
        }
        Response::SkillList { skills } => {
            if skills.is_empty() {
                println!("Aucune skill enregistrée.");
                return Ok(());
            }
            for skill in skills {
                println!("{} — {}", skill.name, skill.description);
            }
        }
        Response::SkillResult { message } => {
            println!("{message}");
        }
        Response::Event(_) => {}
    }
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