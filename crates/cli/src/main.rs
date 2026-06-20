//! CLI Orchestrateur — Phase 5 : CLI + TUI intégré au cœur (feature `tui`).

#[cfg(feature = "tui")]
use std::io::IsTerminal;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use cortex::KnowledgeGraph;
use infrastructure::bootstrap_workspace;
#[cfg(feature = "tui")]
use orchestrator::spawn_orchestrator_bridge;
#[cfg(feature = "tui")]
use orchestrator::TuiApp;
use orchestrator::{
    execute_command, ChatMessage, Command, OrchestratorError, OrchestratorFacade, Response,
};

use tracing_subscriber::EnvFilter;

/// Orchestrateur — second cerveau local souverain.
#[derive(Parser)]
#[command(
    name = "orchestrateur",
    version,
    about = "Orchestrateur v0.3.0 — Cortex + Esprit + CLI/TUI"
)]
struct Cli {
    /// Racine du workspace (défaut: ./workspace).
    #[arg(long, global = true, default_value = "workspace")]
    workspace: PathBuf,

    /// Force le mode interface terminal (ratatui).
    #[arg(long, global = true)]
    tui: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Lance l'interface terminal (ratatui).
    #[cfg(feature = "tui")]
    Tui,
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
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("orchestrateur=info".parse()?))
        .init();

    let cli = Cli::parse();

    if should_launch_tui(&cli) {
        #[cfg(feature = "tui")]
        return run_tui(cli.workspace).await;
        #[cfg(not(feature = "tui"))]
        anyhow::bail!("TUI non compilé — recompilez avec `--features tui`");
    }

    let command = cli.command.context(
        "aucune commande — lancez sans argument dans un terminal interactif (TUI) ou utilisez --help",
    )?;

    let deps = bootstrap_workspace(&cli.workspace)
        .await
        .map_err(|err| anyhow::anyhow!(err.with_context("CLI")))?;
    let facade = OrchestratorFacade::new(deps);

    match command {
        #[cfg(feature = "tui")]
        Commands::Tui => unreachable!("sous-commande `tui` interceptée par should_launch_tui"),
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
        Commands::Graph => cmd_graph(&facade).await?,
        Commands::Chat { message } => cmd_chat(&facade, &message).await?,
    }
    Ok(())
}

fn should_launch_tui(cli: &Cli) -> bool {
    if cli.tui {
        return true;
    }
    #[cfg(feature = "tui")]
    if matches!(cli.command, Some(Commands::Tui)) {
        return true;
    }
    if cli.command.is_some() {
        return false;
    }
    #[cfg(feature = "tui")]
    {
        std::io::stdin().is_terminal()
    }
    #[cfg(not(feature = "tui"))]
    {
        false
    }
}

#[cfg(feature = "tui")]
async fn run_tui(workspace: PathBuf) -> Result<()> {
    if !std::io::stdin().is_terminal() {
        anyhow::bail!("TUI requiert un terminal interactif (stdin n'est pas un TTY)");
    }

    let deps = bootstrap_workspace(&workspace)
        .await
        .map_err(|err| anyhow::anyhow!(err.with_context("TUI")))?;

    let (handle, thread) = spawn_orchestrator_bridge(deps)
        .map_err(|err| anyhow::anyhow!("démarrage bridge orchestrateur: {err}"))?;

    let mut app = TuiApp::new(handle, thread);
    let run_result = tokio::task::spawn_blocking(move || app.run()).await;

    match run_result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(err)) => Err(anyhow::anyhow!("TUI: {err}")),
        Err(join_err) => Err(anyhow::anyhow!("TUI thread: {join_err}")),
    }
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
        Response::Error(err) => {
            anyhow::bail!("[{}] {}", err.kind, err.message);
        }
        Response::Success { message } => {
            println!("{message}");
        }
        Response::Event(_) => {}
    }
    Ok(())
}

async fn cmd_graph(facade: &OrchestratorFacade) -> Result<()> {
    let memories = facade.list_memories().await?;
    let graph = KnowledgeGraph::from_memories(&memories);
    println!("Nœuds : {}", graph.node_count());
    println!("Arêtes : {}", graph.edge_count());
    for (id, inbound) in graph.hub_ranking().into_iter().take(5) {
        if let Ok(mem) = facade.get_memory(id).await {
            println!("  hub ({inbound} liens) : {}", mem.title);
        }
    }
    Ok(())
}

async fn cmd_chat(facade: &OrchestratorFacade, message: &str) -> Result<()> {
    let deps = facade.deps();
    let reply = deps
        .llm
        .chat(&[ChatMessage {
            role: "user".into(),
            content: message.into(),
        }])
        .await
        .map_err(OrchestratorError::Llm)?;
    println!("{reply}");
    Ok(())
}