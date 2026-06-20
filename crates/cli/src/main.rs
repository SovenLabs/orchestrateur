//! CLI Orchestrateur — Phase 3 : assimilate, search, list, graph, chat.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use cortex::{KnowledgeGraph, SearchFilter};
use infrastructure::{build_app_dependencies, WiringError};
use orchestrator::{
    ChatMessage, OrchestratorConfig, OrchestratorFacade, OrchestratorError,
    DEFAULT_ASSIMILATION_SYSTEM_PROMPT,
};
use tracing_subscriber::EnvFilter;

/// Orchestrateur — second cerveau local souverain.
#[derive(Parser)]
#[command(name = "orchestrateur", version, about = "Orchestrateur v0.1.0 — Cortex + Esprit")]
struct Cli {
    /// Racine du workspace (défaut: ./workspace).
    #[arg(long, global = true, default_value = "workspace")]
    workspace: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Liste les mémoires persistées.
    List,
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
    let config = OrchestratorConfig::load_workspace(&cli.workspace)
        .context("chargement configuration")?;

    let facade = match build_facade(config).await {
        Ok(f) => f,
        Err(WiringError::MemoryMode) => {
            anyhow::bail!(
                "vector_store type=memory : configurez type=lancedb dans orchestrator.toml pour le CLI"
            );
        }
        Err(e) => return Err(e.into()),
    };

    match cli.command {
        Commands::List => cmd_list(&facade).await?,
        Commands::Search { query, limit } => cmd_search(&facade, &query, limit).await?,
        Commands::Assimilate { text } => cmd_assimilate(&facade, &text).await?,
        Commands::Graph => cmd_graph(&facade).await?,
        Commands::Chat { message } => cmd_chat(&facade, &message).await?,
    }
    Ok(())
}

async fn build_facade(config: OrchestratorConfig) -> Result<OrchestratorFacade, WiringError> {
    let deps = build_app_dependencies(config).await?;
    Ok(OrchestratorFacade::new(deps))
}

async fn cmd_list(facade: &OrchestratorFacade) -> Result<()> {
    let memories = facade.list_memories().await?;
    if memories.is_empty() {
        println!("Aucune mémoire.");
        return Ok(());
    }
    for mem in memories {
        println!("{} | {} | {} tags", mem.id, mem.title, mem.tags.len());
    }
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
        println!(
            "{:.3} | {} | {}",
            hit.score, mem.id, mem.title
        );
    }
    Ok(())
}

async fn cmd_assimilate(facade: &OrchestratorFacade, text: &str) -> Result<()> {
    let (memory, events) = facade
        .assimilate(text, Some(DEFAULT_ASSIMILATION_SYSTEM_PROMPT))
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