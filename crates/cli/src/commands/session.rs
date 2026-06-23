//! `orch session` — sessions agent (SQLite).

use std::path::Path;

use anyhow::{Context, Result};
use clap::Subcommand;
use cortex::{SessionKey, TurnRole};
use orchestrator::OrchestratorConfig;
use rusqlite::Connection;
use uuid::Uuid;

use crate::context::bootstrap_facade;

/// Sous-commandes session.
#[derive(Debug, Clone, Subcommand)]
pub enum SessionCommands {
    /// Crée une nouvelle session et affiche son identifiant.
    New,
    /// Liste les sessions enregistrées.
    List,
    /// Affiche l'historique d'une session.
    Show {
        /// Clé de session.
        id: String,
    },
    /// Supprime une session.
    Delete {
        id: String,
    },
}

pub async fn run(cmd: SessionCommands, workspace: &Path) -> Result<()> {
    match cmd {
        SessionCommands::New => {
            let facade = bootstrap_facade(workspace).await?;
            let key = SessionKey::new(format!("cli-{}", Uuid::new_v4()))
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            facade
                .deps()
                .session_repo
                .get_or_create(&key)
                .await
                .map_err(|e| anyhow::anyhow!("session: {e}"))?;
            println!("Session créée : {}", key.as_str());
            Ok(())
        }
        SessionCommands::List => list_sessions(workspace),
        SessionCommands::Show { id } => show_session(workspace, &id).await,
        SessionCommands::Delete { id } => delete_session(workspace, &id).await,
    }
}

fn list_sessions(workspace: &Path) -> Result<()> {
    let db = sessions_db(workspace)?;
    let conn = Connection::open(&db).context("ouverture sessions.db")?;
    let mut stmt = conn.prepare(
        "SELECT key, created_at, updated_at FROM sessions ORDER BY updated_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;
    let mut count = 0usize;
    for row in rows {
        let (key, created, updated) = row?;
        println!("{key}  created={created}  updated={updated}");
        count += 1;
    }
    if count == 0 {
        println!("Aucune session.");
    }
    Ok(())
}

async fn show_session(workspace: &Path, id: &str) -> Result<()> {
    let facade = bootstrap_facade(workspace).await?;
    let key = SessionKey::new(id).map_err(|e| anyhow::anyhow!("{e}"))?;
    let turns = facade
        .deps()
        .session_repo
        .list_turns(&key)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    if turns.is_empty() {
        println!("Session `{id}` — aucun tour.");
        return Ok(());
    }
    println!("# session {id} ({} tours)", turns.len());
    for turn in turns {
        let role = match turn.role {
            TurnRole::User => "user",
            TurnRole::Assistant => "assistant",
            TurnRole::Tool => "tool",
            TurnRole::System => "system",
        };
        let preview: String = turn.content.chars().take(120).collect();
        println!("[{role}] {preview}");
    }
    Ok(())
}

async fn delete_session(workspace: &Path, id: &str) -> Result<()> {
    let facade = bootstrap_facade(workspace).await?;
    let key = SessionKey::new(id).map_err(|e| anyhow::anyhow!("{e}"))?;
    facade
        .deps()
        .session_repo
        .delete(&key)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("Session supprimée : {id}");
    Ok(())
}

fn sessions_db(workspace: &Path) -> Result<std::path::PathBuf> {
    let config = OrchestratorConfig::load_workspace(workspace)
        .map_err(|e| anyhow::anyhow!("config: {e}"))?;
    Ok(config.sessions_db_path())
}