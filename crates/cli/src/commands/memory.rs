//! `orch memory` — opérations Cortex (mémoires).

use std::path::Path;

use anyhow::Result;
use clap::Subcommand;
use cortex::MemoryId;
use orchestrator::{execute_command, Command, OrchestratorFacade, Response};

use crate::context::run_bridge_command;
use crate::output::print_response;

/// Sous-commandes mémoire.
#[derive(Debug, Clone, Subcommand)]
pub enum MemoryCommands {
    /// Liste les mémoires persistées.
    List {
        /// Filtre titre ou tags.
        #[arg(long)]
        filter: Option<String>,
        #[arg(long, default_value = "0")]
        offset: usize,
        #[arg(long, default_value = "100")]
        limit: usize,
    },
    /// Recherche sémantique.
    Search {
        /// Requête textuelle.
        query: String,
        #[arg(long, default_value = "10")]
        limit: usize,
    },
    /// Affiche une mémoire par identifiant.
    Show {
        /// UUID mémoire.
        id: String,
    },
    /// Supprime une mémoire (fichier + index vectoriel).
    Delete {
        /// UUID mémoire.
        id: String,
    },
    /// Assimile du texte via le LLM configuré.
    Assimilate {
        text: String,
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
    },
    /// Importe des mémoires Markdown depuis un répertoire.
    Import {
        #[arg(long)]
        source: std::path::PathBuf,
    },
    /// Ré-indexe les embeddings LanceDB.
    Reindex,
    /// Statistiques du graphe de connaissances.
    Graph,
}

pub async fn run(cmd: MemoryCommands, facade: &OrchestratorFacade) -> Result<()> {
    match cmd {
        MemoryCommands::List {
            filter,
            offset,
            limit,
        } => {
            run_bridge_command(
                facade,
                Command::List {
                    filter,
                    offset,
                    limit,
                },
            )
            .await
        }
        MemoryCommands::Search { query, limit } => {
            run_bridge_command(facade, Command::Search { query, limit }).await
        }
        MemoryCommands::Show { id } => {
            run_bridge_command(facade, Command::GetMemory { id }).await
        }
        MemoryCommands::Delete { id } => delete_memory(facade, &id).await,
        MemoryCommands::Assimilate { text, tags } => {
            let response = execute_command(facade, Command::Assimilate { text, tags }).await;
            match response {
                Response::Assimilated { memory_id, title } => {
                    println!("Assimilé : {title} ({memory_id})");
                    Ok(())
                }
                Response::MemoryDetail { memory } => {
                    println!("Assimilé : {} ({})", memory.title, memory.id);
                    println!("Backlinks : {}", memory.backlink_count());
                    Ok(())
                }
                Response::Error(err) => anyhow::bail!("[{}] {}", err.kind, err.message),
                other => print_response(other),
            }
        }
        MemoryCommands::Import { source } => import_memories(facade, &source).await,
        MemoryCommands::Reindex => reindex(facade).await,
        MemoryCommands::Graph => run_bridge_command(facade, Command::Graph).await,
    }
}

async fn delete_memory(facade: &OrchestratorFacade, id: &str) -> Result<()> {
    let memory_id: MemoryId = id
        .parse()
        .map_err(|e| anyhow::anyhow!("id invalide: {e}"))?;
    let deps = facade.deps();
    deps.memory_repo
        .delete(memory_id)
        .await
        .map_err(|e| anyhow::anyhow!("suppression mémoire: {e}"))?;
    let _ = deps.vector_store.delete(memory_id).await;
    println!("Mémoire supprimée : {id}");
    Ok(())
}

async fn import_memories(facade: &OrchestratorFacade, source: &Path) -> Result<()> {
    let result = facade.import_from_directory(source).await?;
    println!(
        "Import : {} importée(s), {} ignorée(s), {} erreur(s)",
        result.imported,
        result.skipped,
        result.errors.len()
    );
    for err in &result.errors {
        eprintln!("  erreur: {err}");
    }
    Ok(())
}

async fn reindex(facade: &OrchestratorFacade) -> Result<()> {
    let memories = facade.list_memories().await?;
    let total = memories.len();
    if total == 0 {
        println!("Reindex : 0 mémoire.");
        return Ok(());
    }
    let mut ok = 0usize;
    for (i, memory) in memories.iter().enumerate() {
        match facade.save_memory(memory).await {
            Ok(_) => {
                ok += 1;
                println!("[{}/{}] {}", i + 1, total, memory.title);
            }
            Err(err) => eprintln!("[{}/{}] {} — {err}", i + 1, total, memory.id),
        }
    }
    println!("Reindex terminé : {ok}/{total}");
    Ok(())
}