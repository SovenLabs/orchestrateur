use serde::{Deserialize, Serialize};

/// Trade Location Score /10 (Bible B2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TradeLocationScore {
    /// Total /10.
    pub total: u8,
    /// Proximité liquidité (0–3).
    pub liquidity_proximity: u8,
    /// Confluence HTF/LTF (0–3).
    pub htf_ltf_confluence: u8,
    /// Extrémité ou retest (0–2).
    pub extreme_or_retest: u8,
    /// Validation B12 (0–2).
    pub b12_validation: u8,
    /// Taille suggérée (`none`, `reduced`, `normal`).
    pub sizing: String,
    /// Justification.
    pub rationale: String,
}

/// Item d'un bloc Quick Check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuickCheckItem {
    /// Libellé de la question.
    pub label: String,
    /// Réponse booléenne.
    pub passed: bool,
}

/// Bloc Stratos Quick Check (macro, structure, liquidité/OF, exécution).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuickCheckBlock {
    /// Nom du bloc.
    pub name: String,
    /// Bloc validé (majorité des checks).
    pub passed: bool,
    /// Checks individuels.
    pub checks: Vec<QuickCheckItem>,
}

/// Résultat Quick Check pré-trade.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuickCheckResult {
    /// Tous les blocs critiques présents.
    pub passed: bool,
    /// Quatre blocs Bible.
    pub blocks: Vec<QuickCheckBlock>,
    /// Synthèse.
    pub rationale: String,
}

/// Score d'alignement desk /10 (Bible section D).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlignmentScore {
    /// Total /10.
    pub total: u8,
    /// Macro 0–2.
    pub macro_score: u8,
    /// Structure 0–2.
    pub structure_score: u8,
    /// Liquidité 0–2.
    pub liquidity_score: u8,
    /// Dérivés & order flow 0–2.
    pub derivatives_of_score: u8,
    /// Exécution 0–2.
    pub execution_score: u8,
    /// Grade (`A_plus`, `good`, `medium`, `avoid`).
    pub grade: String,
    /// Synthèse.
    pub rationale: String,
}

/// Bundle scoring agrégé (PR-4).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoreBundle {
    /// Trade Location Score.
    pub trade_location: TradeLocationScore,
    /// Stratos Quick Check.
    pub quick_check: QuickCheckResult,
    /// Score d'alignement desk.
    pub alignment: AlignmentScore,
    /// Taille finale recommandée (`none`, `reduced`, `normal`).
    pub recommended_sizing: String,
}