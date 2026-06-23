//! Identité et statut d'un agent persistant.

use serde::{Deserialize, Serialize};

/// Cycle de vie d'un agent persistant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    /// Agent en veille — état persisté, pas de tâches actives.
    Sleeping,
    /// Agent réveillé — prêt à traiter des messages et tours.
    Awake,
    /// Agent en tâches de fond (heartbeat, inbox, maintenance).
    Background,
}

impl AgentStatus {
    /// Libellé court pour affichage CLI / registre.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Sleeping => "sleeping",
            Self::Awake => "awake",
            Self::Background => "background",
        }
    }
}

/// Contrat d'identité partagé par les entités agent.
pub trait AgentIdentity {
    /// Identifiant stable (nom de dossier).
    fn id(&self) -> &str;
    /// Nom affiché.
    fn name(&self) -> &str;
    /// Rôle fonctionnel.
    fn role(&self) -> &str;
    /// Modèle LLM associé.
    fn model(&self) -> &str;
    /// Statut courant du cycle de vie.
    fn status(&self) -> AgentStatus;
}