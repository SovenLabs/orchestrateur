//! `orch agent` — agents persistants (Phase 2).

use std::path::Path;

use anyhow::{Context, Result};
use clap::Subcommand;
use orchestrator::AgentIdentity;

use crate::context::bootstrap_facade;

/// Sous-commandes agent persistant.
#[derive(Debug, Clone, Subcommand)]
pub enum AgentCommands {
    /// Liste tous les agents persistants.
    List,
    /// Crée un nouvel agent avec dossier et identité.
    Create {
        /// Identifiant stable (nom de dossier).
        id: String,
        /// Nom affiché.
        #[arg(long)]
        name: Option<String>,
        /// Rôle fonctionnel.
        #[arg(long, default_value = "assistant")]
        role: String,
        /// Modèle LLM (défaut : config xAI).
        #[arg(long)]
        model: Option<String>,
    },
    /// Affiche le détail d'un agent.
    Show {
        id: String,
    },
    /// Réveille un agent (statut awake).
    Wake {
        id: String,
    },
    /// Met un agent en veille (statut sleeping).
    Sleep {
        id: String,
    },
    /// Exécute les tâches de fond (heartbeat).
    Background {
        id: String,
    },
    /// Envoie un message inter-agent.
    Send {
        /// Agent émetteur.
        #[arg(long)]
        from: String,
        /// Agent destinataire.
        #[arg(long, alias = "to")]
        recipient: String,
        /// Corps du message.
        message: String,
    },
    /// Lit l'inbox d'un agent.
    Messages {
        id: String,
        /// Marque les messages comme lus.
        #[arg(long)]
        mark_read: bool,
    },
    /// Exécute un cycle worker (délégations, cron, background).
    Tick,
}

pub async fn run(cmd: AgentCommands, workspace: &Path) -> Result<()> {
    let facade = bootstrap_facade(workspace).await?;
    let manager = facade
        .agent_manager()
        .await
        .map_err(|e| anyhow::anyhow!("agents: {e}"))?;

    match cmd {
        AgentCommands::List => {
            let agents = manager.list().await.map_err(|e| anyhow::anyhow!("{e}"))?;
            if agents.is_empty() {
                println!("Aucun agent persistant.");
                return Ok(());
            }
            for agent in agents {
                println!(
                    "{}  name={}  role={}  status={}  model={}",
                    agent.id(),
                    agent.name(),
                    agent.role(),
                    agent.status().label(),
                    agent.model(),
                );
            }
            Ok(())
        }
        AgentCommands::Create { id, name, role, model } => {
            let display_name = name.unwrap_or_else(|| id.clone());
            let agent = manager
                .create_agent(&id, &display_name, &role, model.as_deref())
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!(
                "Agent créé : {} ({}) — dossier {}",
                agent.id(),
                agent.name(),
                agent.root.display()
            );
            Ok(())
        }
        AgentCommands::Show { id } => {
            let agent = manager.get(&id).await.map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("# agent {}", agent.id());
            println!("name: {}", agent.name());
            println!("role: {}", agent.role());
            println!("model: {}", agent.model());
            println!("status: {}", agent.status().label());
            println!("session: {}", agent.config.session_key);
            println!("root: {}", agent.root.display());
            if let Some(hb) = &agent.config.last_heartbeat {
                println!("last_heartbeat: {hb}");
            }
            Ok(())
        }
        AgentCommands::Wake { id } => {
            let agent = manager.wake(&id).await.context("réveil agent")?;
            println!("Agent {} réveillé ({}).", agent.id(), agent.status().label());
            Ok(())
        }
        AgentCommands::Sleep { id } => {
            let agent = manager.sleep(&id).await.context("veille agent")?;
            println!("Agent {} en veille ({}).", agent.id(), agent.status().label());
            Ok(())
        }
        AgentCommands::Background { id } => {
            let report = manager.background(&id).await.context("tâches de fond")?;
            println!(
                "Background {} — inbox={} tasks={} executed={:?}",
                id, report.inbox_count, report.pending_tasks, report.executed
            );
            Ok(())
        }
        AgentCommands::Send {
            from,
            recipient,
            message,
        } => {
            let msg = manager
                .send_message(&from, &recipient, &message)
                .await
                .context("envoi message")?;
            println!("Message {} → {} (id={})", msg.from, msg.to, msg.id);
            Ok(())
        }
        AgentCommands::Messages { id, mark_read } => {
            let messages = manager
                .receive_messages(&id, mark_read)
                .await
                .context("lecture inbox")?;
            if messages.is_empty() {
                println!("Inbox `{id}` vide.");
                return Ok(());
            }
            for msg in messages {
                let flag = if msg.read { "read" } else { "unread" };
                println!("[{flag}] {} → {} : {}", msg.from, msg.to, msg.body);
            }
            Ok(())
        }
        AgentCommands::Tick => {
            let report = orchestrator::run_agent_tick(&facade)
                .await
                .map_err(|e| anyhow::anyhow!("tick: {e}"))?;
            println!(
                "Tick — background={} inbox_turns={} delegations={} cron={}",
                report.agents_background,
                report.inbox_turns,
                report.delegations_completed,
                report.cron_ran
            );
            Ok(())
        }
    }
}