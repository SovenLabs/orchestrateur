//! Workflow desk B212 — pipeline déterministe via les 6 agents domaine.

use serde::{Deserialize, Serialize};

use b212::{
    build_setup_analysis, ModuleContext, ModuleId, ModuleOutput, Timeframe, TradeProposal,
    B212SetupAnalysis,
};

use super::agents::B212_AGENTS;
use super::governance::B212GovernanceService;
use crate::deps::AppDependencies;

/// Requête d'analyse setup B212.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct B212AnalyzeRequest {
    /// Symbole (ex. `BTCUSDT`).
    pub symbol: String,
    /// Session de trading (`london`, `ny`, `asia`).
    pub session: String,
    /// Nombre de bougies par timeframe.
    pub lookback: usize,
}

impl Default for B212AnalyzeRequest {
    fn default() -> Self {
        Self {
            symbol: "BTCUSDT".into(),
            session: "london".into(),
            lookback: 24,
        }
    }
}

/// Rapport d'une étape agent dans le workflow.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct B212AgentStepReport {
    /// Identifiant agent persistant.
    pub agent_id: String,
    /// Nom affiché.
    pub agent_name: String,
    /// Rôle Bible.
    pub bible_role: String,
    /// Résumé desk.
    pub summary: String,
    /// Score confiance module (si applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<u8>,
}

/// Résultat complet du workflow B212.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct B212WorkflowResult {
    /// Symbole analysé.
    pub symbol: String,
    /// Session.
    pub session: String,
    /// Analyse agrégée.
    pub analysis: B212SetupAnalysis,
    /// Étapes agents dans l'ordre canonique.
    pub steps: Vec<B212AgentStepReport>,
    /// Proposition HITL créée (si éligible).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proposal: Option<TradeProposal>,
}

/// Service workflow B212 (fixtures + gouvernance).
pub struct B212WorkflowService {
    deps: AppDependencies,
}

impl B212WorkflowService {
    /// Construit le service si B212 est activé et les ports sont câblés.
    ///
    /// # Errors
    ///
    /// Retourne [`b212::B212Error::Config`] si B212 est désactivé ou incomplet.
    pub fn new(deps: AppDependencies) -> Result<Self, b212::B212Error> {
        if !deps.config.b212.enabled {
            return Err(b212::B212Error::Config("protocole B212 désactivé".into()));
        }
        if deps.market_data.is_none() {
            return Err(b212::B212Error::Config(
                "market_data B212 non câblé".into(),
            ));
        }
        Ok(Self { deps })
    }

    /// Accès aux dépendances.
    #[must_use]
    pub fn deps(&self) -> &AppDependencies {
        &self.deps
    }

    /// Service de gouvernance (journal + propositions) si disponible.
    #[must_use]
    pub fn governance(&self) -> Option<B212GovernanceService> {
        match (&self.deps.b212_journal, &self.deps.b212_proposals) {
            (Some(j), Some(p)) => {
                if self.deps.config.b212.events_enabled {
                    Some(B212GovernanceService::with_events(
                        j.clone(),
                        p.clone(),
                        self.deps.events.clone(),
                        true,
                    ))
                } else {
                    Some(B212GovernanceService::new(j.clone(), p.clone()))
                }
            }
            _ => None,
        }
    }

    /// Exécute le workflow complet : chargement fixtures → analyse → gouvernance.
    pub async fn run(&self, req: B212AnalyzeRequest) -> Result<B212WorkflowResult, b212::B212Error> {
        let series = self.load_series(&req).await?;
        let ctx = ModuleContext::new(&req.symbol, series);
        let analysis = build_setup_analysis(&ctx, &req.session);
        let steps = build_agent_steps(&analysis);

        let proposal = if let Some(gov) = self.governance() {
            gov.process_analysis(&analysis).await?
        } else {
            None
        };

        Ok(B212WorkflowResult {
            symbol: req.symbol,
            session: req.session,
            analysis,
            steps,
            proposal,
        })
    }

    async fn load_series(&self, req: &B212AnalyzeRequest) -> Result<Vec<b212::OhlcvSeries>, b212::B212Error> {
        let provider = self
            .deps
            .market_data
            .as_ref()
            .ok_or_else(|| b212::B212Error::MarketData("provider absent".into()))?;
        let symbol = req.symbol.to_uppercase();
        let lookback = req.lookback;
        let candidates = default_timeframes_for(&symbol);
        let mut loaded = Vec::new();
        for tf in candidates {
            let series = match provider.get_ohlcv(&symbol, tf, lookback).await {
                Ok(s) => Some(s),
                Err(b212::B212Error::InsufficientBars { .. }) if lookback > 0 => {
                    provider.get_ohlcv(&symbol, tf, 0).await.ok()
                }
                Err(_) => None,
            };
            if let Some(s) = series {
                loaded.push(s);
            }
        }
        if loaded.is_empty() {
            return Err(b212::B212Error::FixtureNotFound {
                path: format!("aucune fixture pour {symbol}"),
            });
        }
        Ok(loaded)
    }
}

fn default_timeframes_for(symbol: &str) -> Vec<Timeframe> {
    match symbol {
        "BTCUSDT" => vec![Timeframe::H4, Timeframe::H1],
        "ETHUSDT" => vec![Timeframe::M15, Timeframe::H1],
        _ => vec![Timeframe::H1, Timeframe::H4, Timeframe::M15],
    }
}

fn build_agent_steps(analysis: &B212SetupAnalysis) -> Vec<B212AgentStepReport> {
    let module = |id: ModuleId| {
        analysis
            .modules
            .iter()
            .find(|m| m.module == id)
            .cloned()
    };

    B212_AGENTS
        .iter()
        .map(|def| step_for_agent(def, &module, analysis))
        .collect()
}

fn step_for_agent(
    def: &super::agents::B212AgentDef,
    module: &dyn Fn(ModuleId) -> Option<ModuleOutput>,
    analysis: &B212SetupAnalysis,
) -> B212AgentStepReport {
    let (summary, confidence) = match def.id {
        "b212-research-analyst" => module_output_step(module(ModuleId::B1)),
        "b212-market-regime" => module_output_step(module(ModuleId::B1_5)),
        "b212-structure" => {
            let b2 = module(ModuleId::B2);
            let b25 = module(ModuleId::B2_5);
            let summary = format!(
                "{} | {}",
                b2.as_ref().map(|m| m.summary.as_str()).unwrap_or("n/a"),
                b25.as_ref().map(|m| m.summary.as_str()).unwrap_or("n/a")
            );
            let confidence = b2
                .as_ref()
                .map(|m| m.confidence)
                .or_else(|| b25.as_ref().map(|m| m.confidence));
            (summary, confidence)
        }
        "b212-order-flow" => module_output_step(module(ModuleId::B12)),
        "b212-risk-manager" => {
            let scores = analysis.scores.as_ref();
            let cardinal = analysis.cardinal.as_ref();
            let summary = match (scores, cardinal) {
                (Some(s), Some(c)) => format!(
                    "TLS {}/10, alignement {}/10, sizing {} — cardinal {}.",
                    s.trade_location.total,
                    s.alignment.total,
                    s.recommended_sizing,
                    if c.passed { "OK" } else { "BLOQUÉ" }
                ),
                _ => "Scores ou cardinal manquants.".into(),
            };
            let confidence = scores.map(|s| s.alignment.total);
            (summary, confidence)
        }
        "b212-execution" => {
            let summary = analysis
                .scores
                .as_ref()
                .map(|s| {
                    format!(
                        "Quick Check {} — recommandation {}.",
                        if s.quick_check.passed {
                            "complet"
                        } else {
                            "incomplet"
                        },
                        s.recommended_sizing
                    )
                })
                .unwrap_or_else(|| "Exécution en attente de scores.".into());
            (summary, None)
        }
        _ => ("Étape non mappée.".into(), None),
    };

    B212AgentStepReport {
        agent_id: def.id.to_string(),
        agent_name: def.name.to_string(),
        bible_role: def.bible_role.to_string(),
        summary,
        confidence,
    }
}

fn module_output_step(output: Option<ModuleOutput>) -> (String, Option<u8>) {
    match output {
        Some(m) => (m.summary, Some(m.confidence)),
        None => ("Module non disponible.".into(), None),
    }
}