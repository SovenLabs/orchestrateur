use serde::{Deserialize, Serialize};

/// Commande envoyée par la couche présentation (HUD, CLI, TUI) vers l'orchestrateur.
///
/// Le HUD n'accède jamais aux ports Cortex : tout transite par ce contrat sérialisable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    /// Assimilation depuis texte brut — le brouillon LLM est produit côté orchestrateur.
    Assimilate {
        /// Contenu à assimiler.
        text: String,
        /// Tags suggérés (transmis au provider LLM comme contexte).
        tags: Vec<String>,
    },
    /// Recherche vectorielle hybride.
    Search {
        /// Requête textuelle.
        query: String,
        /// Nombre maximal de résultats.
        limit: usize,
    },
    /// Liste paginée avec filtre optionnel sur le titre ou les tags.
    List {
        /// Sous-chaîne à rechercher dans titre/tags (insensible à la casse).
        filter: Option<String>,
        /// Décalage pour pagination.
        offset: usize,
        /// Nombre d'éléments par page.
        limit: usize,
    },
    /// Récupération complète d'une mémoire par identifiant UUID.
    GetMemory {
        /// Identifiant canonique (`MemoryId` en string).
        id: String,
    },
    /// Demande explicite d'abonnement aux événements (accusé de réception).
    SubscribeToEvents,
    /// Ping de santé du bridge et de l'orchestrateur.
    HealthCheck,
}
