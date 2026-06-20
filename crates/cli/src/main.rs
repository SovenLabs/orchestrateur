//! CLI Orchestrateur — Phase 5 : CLI + TUI intégré au cœur (feature `tui`).

#[cfg(feature = "tui")]
use std::io::IsTerminal;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use cortex::{KnowledgeGraph, SearchFilter};
use infrastructure::{build_app_dependencies, WiringError};
#[cfg(feature = "tui")]
use orchestrator::spawn_orchestrator_bridge;
#[cfg(feature = "tui")]
use orchestrator::TuiApp;
use orchestrator::{
    format_assimilate_user_prompt, ChatMessage, OrchestratorConfig, OrchestratorError,
    OrchestratorFacade, DEFAULT_ASSIMILATION_SYSTEM_PROMPT, VERSION,
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

    let config =
        OrchestratorConfig::load_workspace(&cli.workspace).context("chargement configuration")?;

    let facade = match build_facade(config).await {
        Ok(f) => f,
        Err(WiringError::MemoryMode) => {
            anyhow::bail!(
                "vector_store type=memory : configurez type=lancedb dans orchestrator.toml pour le CLI"
            );
        }
        Err(e) => return Err(e.into()),
    };

    match command {
        #[cfg(feature = "tui")]
        Commands::Tui => unreachable!("sous-commande `tui` interceptée par should_launch_tui"),
        Commands::Health => cmd_health()?,
        Commands::List {
            filter,
            offset,
            limit,
        } => cmd_list(&facade, filter.as_deref(), offset, limit).await?,
        Commands::Get { id } => cmd_get(&facade, &id).await?,
        Commands::Search { query, limit } => cmd_search(&facade, &query, limit).await?,
        Commands::Assimilate { text, tags } => cmd_assimilate(&facade, &text, &tags).await?,
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

    let config = OrchestratorConfig::load_workspace(&workspace)
        .with_context(|| format!("chargement config depuis {}", workspace.display()))?;

    let deps = match build_app_dependencies(config).await {
        Ok(deps) => deps,
        Err(WiringError::MemoryMode) => {
            anyhow::bail!(
                "vector_store type=memory : configurez type=lancedb dans orchestrator.toml pour le TUI"
            );
        }
        Err(err) => return Err(err.into()),
    };

    let (handle, thread) = spawn_orchestrator_bridge(deps)
        .map_err(|err| anyhow::anyhow!("démarrage bridge orchestrateur: {err}"))?;

    // NOTE(architecte): la boucle ratatui est synchrone — on l'isole dans spawn_blocking
    // pour ne pas bloquer le runtime Tokio du thread principal.
    let mut app = TuiApp::new(handle, thread);
    let run_result = tokio::task::spawn_blocking(move || app.run()).await;

    match run_result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(err)) => Err(anyhow::anyhow!("TUI: {err}")),
        Err(join_err) => Err(anyhow::anyhow!("TUI thread: {join_err}")),
    }
}

fn cmd_health() -> Result<()> {
    println!("status=ok version={VERSION}");
    Ok(())
}

async fn build_facade(config: OrchestratorConfig) -> Result<OrchestratorFacade, WiringError> {
    let deps = build_app_dependencies(config).await?;
    Ok(OrchestratorFacade::new(deps))
}

async fn cmd_list(
    facade: &OrchestratorFacade,
    filter: Option<&str>,
    offset: usize,
    limit: usize,
) -> Result<()> {
    let mut memories = facade.list_memories().await?;
    if let Some(needle) = filter {
        if !needle.is_empty() {
            let needle = needle.to_lowercase();
            memories.retain(|mem| {
                mem.title.to_lowercase().contains(&needle)
                    || mem
                        .tags
                        .iter()
                        .any(|tag| tag.as_str().to_lowercase().contains(&needle))
            });
        }
    }
    let total = memories.len();
    let page: Vec<_> = memories.into_iter().skip(offset).take(limit).collect();
    if page.is_empty() {
        println!("Aucune mémoire (total={total}).");
        return Ok(());
    }
    println!("# total={total} offset={offset} limit={limit}");
    for mem in page {
        let tags: Vec<_> = mem.tags.iter().map(|t| t.as_str()).collect();
        println!("{} | {} | tags=[{}]", mem.id, mem.title, tags.join(", "));
    }
    Ok(())
}

async fn cmd_get(facade: &OrchestratorFacade, id: &str) -> Result<()> {
    let memory_id = id
        .parse()
        .map_err(|e| anyhow::anyhow!("identifiant invalide: {e}"))?;
    let mem = facade.get_memory(memory_id).await?;
    println!("# {}", mem.title);
    println!("id: {}", mem.id);
    if !mem.tags.is_empty() {
        let tags: Vec<_> = mem.tags.iter().map(|t| t.as_str()).collect();
        println!("tags: {}", tags.join(", "));
    }
    println!("---");
    println!("{}", mem.content);
    Ok(())
}

async fn cmd_search(facade: &OrchestratorFacade, query: &str, limit: usize) -> Result<()> {
    let filter = SearchFilter {
        limit: Some(limit),
        ..Default::default()
    };
    let hits = facade.search_memories(query, &filter).await?;
    if hits.is_empty() {
        println!("Aucun résultat.");
        return Ok(());
    }
    for hit in hits {
        let mem = facade.get_memory(hit.memory_id).await?;
        let snippet = hit.snippet.as_deref().unwrap_or(mem.content.as_str());
        let preview: String = snippet.chars().take(120).collect();
        println!("{:.3} | {} | {}", hit.score, mem.id, mem.title);
        println!("    {preview}");
    }
    Ok(())
}

async fn cmd_assimilate(facade: &OrchestratorFacade, text: &str, tags: &[String]) -> Result<()> {
    let prompt = format_assimilate_user_prompt(text, tags);
    let (memory, events) = facade
        .assimilate(&prompt, Some(DEFAULT_ASSIMILATION_SYSTEM_PROMPT))
        .await
        .map_err(map_orch_err)?;
    println!("Assimilé : {} ({})", memory.title, memory.id);
    println!("Backlinks : {}", memory.backlink_count());
    println!("Événements : {}", events.len());
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

fn map_orch_err(err: OrchestratorError) -> anyhow::Error {
    err.into()
}
