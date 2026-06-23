use serde::{Deserialize, Serialize};

/// Identifiant d'une règle cardinale Bible B212.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardinalRuleId {
    /// Le contexte contextualise, il ne déclenche pas (B1).
    ContextNeverTriggers,
    /// La structure décide si un trade existe (B2).
    StructureDecidesTrade,
    /// L'exécution valide, elle ne sauve jamais une mauvaise idée (B12).
    ExecutionNeverSaves,
    /// Le flow ne peut pas sauver une structure invalide.
    FlowCannotRescueStructure,
    /// Compression ≠ accumulation haussière garantie.
    CompressionNotAccumulation,
    /// Quick Check complet avant taille normale.
    QuickCheckComplete,
    /// Trade Location Score minimal pour entrer.
    TradeLocationMinimum,
    /// Narratif auditable en 6 phrases.
    NarrativeAuditable,
}

/// Violation d'une règle cardinale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardinalViolation {
    /// Règle violée.
    pub rule: CardinalRuleId,
    /// Message explicite pour l'opérateur.
    pub message: String,
}

/// Résultat d'évaluation des règles cardinales.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardinalRulesResult {
    /// Toutes les règles applicables respectées.
    pub passed: bool,
    /// Violations détectées.
    pub violations: Vec<CardinalViolation>,
    /// Synthèse desk.
    pub rationale: String,
}