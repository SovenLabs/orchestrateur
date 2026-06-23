//! Définition clap — commandes modernes + alias legacy (cachés).

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::commands::{
    AgentCommands, BackupCommands, B212Commands, ConfigCommands, DaemonCommands, MemoryCommands,
    OnboardArgs, SessionCommands, SkillCommands, UpdateArgs,
};

/// Orchestrateur — second cerveau local souverain.
#[derive(Parser)]
#[command(
    name = "orchestrateur",
    alias = "orchestre",
    alias = "orch",
    version,
    about = "Orchestrateur v0.28.0 — Cortex + Esprit souverain",
    after_help = "Alias : orchestrateur | orchestre | orch\nExemples :\n  orch onboard\n  orch update\n  orch memory list\n  orch daemon start"
)]
pub struct Cli {
    /// Racine du workspace (défaut: ./workspace).
    #[arg(long, global = true, default_value = "workspace")]
    pub workspace: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

/// Arborescence des commandes CLI.
#[derive(Subcommand)]
pub enum Commands {
    /// Première installation et configuration.
    Onboard(OnboardArgs),

    /// Diagnostic complet du système.
    Doctor,

    /// Met à jour Orchestrateur (release ou dev, sans interaction).
    Update(UpdateArgs),

    /// Gère le daemon WebSocket local.
    Daemon {
        #[command(subcommand)]
        command: DaemonCommands,
    },

    /// Opérations sur les mémoires Cortex.
    Memory {
        #[command(subcommand)]
        command: MemoryCommands,
    },

    /// Sessions de conversation agent.
    Session {
        #[command(subcommand)]
        command: SessionCommands,
    },

    /// Agents persistants (identité, heartbeat, messagerie).
    Agent {
        #[command(subcommand)]
        command: AgentCommands,
    },

    /// Protocole B212 — analyse desk, agents domaine, HITL.
    B212 {
        #[command(subcommand)]
        command: B212Commands,
    },

    /// Configuration orchestrator.toml.
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// Skills opérationnelles et hub.
    Skill {
        #[command(subcommand)]
        command: SkillCommands,
    },

    /// Sauvegarde et restauration du workspace.
    Backup {
        #[command(subcommand)]
        command: BackupCommands,
    },

    /// Santé rapide du bridge.
    Health,

    /// Désinstallation complète.
    Uninstall,

    /// Chat libre avec le LLM.
    Chat {
        message: String,
    },

    /// Journal d'audit récent.
    Audit {
        #[arg(long, default_value = "50")]
        limit: usize,
    },

    /// Harness — validation intégrée.
    Harness {
        #[command(subcommand)]
        command: HarnessCommands,
    },

    /// Serveur MCP stdio.
    Mcp {
        #[command(subcommand)]
        command: McpCommands,
    },

    /// Surveillance sessions Markdown → brouillons.
    Watch,

    /// Contrôle du watcher de sessions.
    Watcher {
        #[command(subcommand)]
        command: WatcherCommands,
    },

    /// File de brouillons Cortex.
    Draft {
        #[command(subcommand)]
        command: DraftCommands,
    },

    /// Catalogue providers LLM / embeddings.
    Providers {
        #[command(subcommand)]
        command: ProviderCommands,
    },

    /// Profils de capacités agent.
    #[command(name = "capability-profiles")]
    CapabilityProfiles {
        #[command(subcommand)]
        command: CapabilityProfileCommands,
    },

    /// Hub skills (alias avancé — préférez `orch skill`).
    #[command(hide = true)]
    SkillsHub {
        #[command(subcommand)]
        command: SkillsHubCommands,
    },

    /// Daemon HTTP (feature `http`).
    #[cfg(feature = "http")]
    Serve {
        #[arg(long, default_value = "17489")]
        port: u16,
        #[arg(long, default_value = "127.0.0.1")]
        bind: String,
    },

    /// Gateway messaging (feature `gateway`).
    #[cfg(feature = "gateway")]
    Gateway {
        #[command(subcommand)]
        command: GatewayCommands,
    },

    /// Canaux gateway.
    #[cfg(feature = "gateway")]
    Channels {
        #[command(subcommand)]
        command: ChannelCommands,
    },

    // --- Legacy (migration progressive, cachées) ---

    /// [obsolète] Utilisez `orch onboard`.
    #[command(hide = true)]
    Setup,

    /// [obsolète] Utilisez `orch config edit`.
    #[command(hide = true)]
    Settings,

    /// [obsolète] Utilisez `orch config set`.
    #[command(hide = true)]
    Configure {
        #[arg(long)]
        profile: Option<String>,
        #[arg(long)]
        llm: Option<String>,
        #[arg(long)]
        local_only: bool,
    },

    /// [obsolète] Utilisez `orch memory list`.
    #[command(hide = true)]
    List {
        #[arg(long)]
        filter: Option<String>,
        #[arg(long, default_value = "0")]
        offset: usize,
        #[arg(long, default_value = "100")]
        limit: usize,
    },

    /// [obsolète] Utilisez `orch memory show`.
    #[command(hide = true)]
    Get {
        id: String,
    },

    /// [obsolète] Utilisez `orch memory search`.
    #[command(hide = true)]
    Search {
        query: String,
        #[arg(long, default_value = "10")]
        limit: usize,
    },

    /// [obsolète] Utilisez `orch memory assimilate`.
    #[command(hide = true)]
    Assimilate {
        text: String,
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
    },

    /// [obsolète] Utilisez `orch memory graph`.
    #[command(hide = true)]
    Graph,

    /// [obsolète] Utilisez `orch memory import`.
    #[command(hide = true)]
    Import {
        #[arg(long)]
        source: PathBuf,
    },

    /// [obsolète] Utilisez `orch memory reindex`.
    #[command(hide = true)]
    Reindex,
}

#[derive(Subcommand)]
pub enum HarnessCommands {
    /// Enchaîne health, graph, drafts, watcher, chat.
    Smoke {
        #[arg(long)]
        skip_gateway: bool,
        #[arg(long)]
        skip_chat: bool,
    },
    /// Démarre daemon + gateway si absents.
    Run,
}

#[derive(Subcommand)]
pub enum McpCommands {
    /// Serveur MCP JSON-RPC sur stdin/stdout.
    Serve,
}

#[derive(Subcommand)]
pub enum WatcherCommands {
    Status,
    Start,
    Stop,
}

#[derive(Subcommand)]
pub enum DraftCommands {
    List,
    Get {
        id: String,
    },
    Publish {
        id: String,
    },
    Discard {
        id: String,
    },
}

#[derive(Subcommand)]
pub enum ProviderCommands {
    List {
        #[arg(long)]
        kind: Option<String>,
    },
    Test {
        #[arg(long)]
        kind: Option<String>,
    },
    Set {
        provider: String,
    },
}

#[derive(Subcommand)]
pub enum CapabilityProfileCommands {
    List,
}

#[derive(Subcommand)]
pub enum SkillsHubCommands {
    List,
    Path,
    Marketplace,
    Sync,
    Verify,
}

#[cfg(feature = "gateway")]
#[derive(Subcommand)]
pub enum GatewayCommands {
    Run {
        #[arg(long)]
        port: Option<u16>,
        #[arg(long)]
        bind: Option<String>,
    },
    Status,
}

#[cfg(feature = "gateway")]
#[derive(Subcommand)]
pub enum ChannelCommands {
    List,
    Enable {
        channel: String,
    },
    Disable {
        channel: String,
    },
    Status,
}