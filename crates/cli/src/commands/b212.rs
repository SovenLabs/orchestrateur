//! `orch b212` — protocole B212 Phase 3.

use std::path::Path;

use anyhow::Result;
use clap::Subcommand;
use orchestrator::{B212AnalyzeRequest, Command};

use crate::context::{bootstrap_facade, run_bridge_command};

/// Sous-commandes protocole B212.
#[derive(Debug, Clone, Subcommand)]
pub enum B212Commands {
    /// Initialise les 6 agents domaine B212.
    InitAgents,
    /// Analyse un setup via fixtures OHLCV.
    Analyze {
        /// Symbole (ex. BTCUSDT).
        symbol: String,
        /// Session de trading.
        #[arg(long, default_value = "london")]
        session: String,
        /// Lookback bougies par timeframe.
        #[arg(long, default_value = "24")]
        lookback: usize,
    },
    /// Liste les propositions en attente validation humaine.
    Proposals,
    /// Approuve une proposition trade.
    Approve {
        /// Identifiant proposition.
        id: String,
    },
    /// Rejette une proposition trade.
    Reject {
        /// Identifiant proposition.
        id: String,
        /// Motif de rejet.
        #[arg(long, default_value = "rejet opérateur")]
        reason: String,
    },
    /// Exécute une proposition approuvée en simulation paper.
    SimExecute {
        /// Identifiant proposition.
        id: String,
    },
}

pub async fn run(cmd: B212Commands, workspace: &Path) -> Result<()> {
    let facade = bootstrap_facade(workspace).await?;

    match cmd {
        B212Commands::InitAgents => {
            let ids = facade.b212_init_agents().await?;
            println!("Agents B212 prêts ({}):", ids.len());
            for id in ids {
                println!("  - {id}");
            }
        }
        B212Commands::Analyze {
            symbol,
            session,
            lookback,
        } => {
            let result = facade
                .b212_analyze(B212AnalyzeRequest {
                    symbol: symbol.clone(),
                    session,
                    lookback,
                })
                .await?;
            println!("B212 analyse {symbol} — {} étapes", result.steps.len());
            for step in &result.steps {
                println!("  [{}] {}", step.agent_name, step.summary);
            }
            if let Some(cardinal) = &result.analysis.cardinal {
                println!(
                    "Cardinal: {} ({})",
                    if cardinal.passed { "OK" } else { "BLOQUÉ" },
                    cardinal.rationale
                );
            }
            if let Some(scores) = &result.analysis.scores {
                println!(
                    "Scores: TLS {}/10, alignement {}/10, sizing {}",
                    scores.trade_location.total,
                    scores.alignment.total,
                    scores.recommended_sizing
                );
            }
            if let Some(p) = &result.proposal {
                println!("Proposition créée: {} ({})", p.id, p.side);
            } else {
                println!("Aucune proposition créée (observation ou blocage).");
            }
        }
        B212Commands::Proposals => {
            let resp = run_bridge_command(&facade, Command::B212ListProposals).await?;
            println!("{resp}");
        }
        B212Commands::Approve { id } => {
            let resp = run_bridge_command(&facade, Command::B212ApproveProposal { id }).await?;
            println!("{resp}");
        }
        B212Commands::Reject { id, reason } => {
            let resp = run_bridge_command(
                &facade,
                Command::B212RejectProposal { id, reason },
            )
            .await?;
            println!("{resp}");
        }
        B212Commands::SimExecute { id } => {
            let resp = run_bridge_command(&facade, Command::B212SimExecute { id }).await?;
            println!("{resp}");
        }
    }

    Ok(())
}