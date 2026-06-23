//! Ports et types pour la communication entre l'Agent Loop et le Cortex.
//!
//! Ce module définit l'interface minimale que l'Agent Loop utilise pour
//! interagir avec le Cortex, sans dépendre des adapters d'infrastructure.
//!
//! ## Séparation des responsabilités
//!
//! - **Cortex** : persistance session ([`SessionRepository`]), mémoires, recherche.
//! - **Agent Loop** : cycle de vie de la session courante, orchestration LLM.
//!
//! Les implémentations concrètes vivent dans `orchestrator` (composition des
//! ports existants : [`MemoryRepository`], [`VectorStore`], [`EmbeddingProvider`]).

use async_trait::async_trait;
use thiserror::Error;

use crate::domain::{
    ConversationTurn, CortexError, Memory, MemoryDraft, MemoryId, SessionKey,
};
use crate::ports::EmbeddingError;
use crate::services::ValidationWarning;

/// Contexte enrichi envoyé à l'Agent / LLM.
#[derive(Debug, Clone, PartialEq)]
pub struct AgentContext {
    /// Mémoires pertinentes trouvées.
    pub memories: Vec<Memory>,
    /// Résumé du graphe de connaissances (optionnel).
    pub graph_context: Option<String>,
    /// Historique récent de la session.
    pub session_turns: Vec<ConversationTurn>,
}

/// Résultat d'une assimilation de tour.
#[derive(Debug, Clone, PartialEq)]
pub struct AssimilationResult {
    /// Mémoires créées.
    pub created: Vec<MemoryId>,
    /// Mémoires mises à jour.
    pub updated: Vec<MemoryId>,
    /// Brouillons en attente d'approbation utilisateur.
    pub pending_drafts: Vec<MemoryDraft>,
}

impl AssimilationResult {
    /// Aucune modification persistée.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            created: Vec::new(),
            updated: Vec::new(),
            pending_drafts: Vec::new(),
        }
    }

    /// Indique si des brouillons attendent une approbation humaine.
    #[must_use]
    pub fn has_pending_approval(&self) -> bool {
        !self.pending_drafts.is_empty()
    }
}

/// Politique d'assimilation appliquée à un tour de conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssimilationPolicy {
    /// Assimile automatiquement seulement s'il y a un changement significatif.
    AutoIfChange,
    /// Demande toujours l'approbation de l'utilisateur (draft sans persistance).
    RequireUserApproval,
    /// Assimile toujours automatiquement (équivalent « always approve »).
    AlwaysAuto,
}

/// Hit de recherche enrichi pour l'agent (mémoire complète + score).
///
/// Distinct de [`VectorSearchHit`] qui ne transporte qu'un `memory_id` et un extrait.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextSearchHit {
    /// Mémoire correspondante.
    pub memory: Memory,
    /// Score de similarité (plus élevé = plus pertinent).
    pub score: f32,
}

// =============================================================================
// Ports exposés à l'Agent Loop
// =============================================================================

/// Fournit du contexte pertinent à l'agent avant l'appel LLM.
#[async_trait]
pub trait ContextProvider: Send + Sync {
    /// Construit le contexte à partir de la requête utilisateur.
    ///
    /// `session_id` permet d'inclure l'historique récent ; `limit` borne la
    /// recherche sémantique et le nombre de tours session injectés.
    async fn build_context(
        &self,
        query: &str,
        session_id: Option<SessionKey>,
        limit: usize,
    ) -> Result<AgentContext, RetrievalError>;
}

/// Gère l'assimilation intelligente des tours de conversation.
#[async_trait]
pub trait AssimilationService: Send + Sync {
    /// Tente d'assimiler un tour selon la politique demandée.
    async fn assimilate_turn(
        &self,
        turn: &ConversationTurn,
        policy: AssimilationPolicy,
    ) -> Result<AssimilationResult, AssimilationError>;
}

/// Recherche sémantique de haut niveau pour l'agent.
#[async_trait]
pub trait SemanticSearch: Send + Sync {
    /// Recherche les mémoires les plus pertinentes pour `query`.
    async fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<ContextSearchHit>, RetrievalError>;
}

// =============================================================================
// Erreurs fines
// =============================================================================

/// Erreurs de récupération de contexte ou de recherche.
#[derive(Debug, Error)]
pub enum RetrievalError {
    /// Vector store indisponible ou en erreur.
    #[error("vector store indisponible")]
    VectorStoreUnavailable,

    /// Aucune mémoire au-dessus du seuil de pertinence.
    #[error("aucune mémoire pertinente trouvée")]
    NoRelevantMemories,

    /// Échec de génération d'embedding pour la requête.
    #[error("échec de génération d'embedding: {0}")]
    EmbeddingFailed(#[from] EmbeddingError),

    /// Erreur domaine Cortex propagée.
    #[error("erreur Cortex: {0}")]
    Cortex(#[from] CortexError),
}

/// Erreurs d'assimilation d'un tour agent.
#[derive(Debug, Error)]
pub enum AssimilationError {
    /// Validation du brouillon échouée (avertissements bloquants agrégés).
    #[error("validation échouée: {0:?}")]
    ValidationFailed(Vec<ValidationWarning>),

    /// Brouillon produit mais persistance différée (approbation requise).
    #[error("approbation utilisateur requise")]
    UserApprovalRequired(Vec<MemoryDraft>),

    /// Échec lors de la persistance mémoire ou vectorielle.
    #[error("échec de persistance: {0}")]
    PersistenceFailed(#[from] CortexError),

    /// Impossible de déterminer si le tour apporte un changement significatif.
    #[error("détection de changement échouée")]
    ChangeDetectionFailed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assimilation_result_empty_has_no_pending() {
        let r = AssimilationResult::empty();
        assert!(!r.has_pending_approval());
    }

    #[test]
    fn assimilation_policy_variants_distinct() {
        assert_ne!(
            AssimilationPolicy::AutoIfChange,
            AssimilationPolicy::RequireUserApproval
        );
    }
}