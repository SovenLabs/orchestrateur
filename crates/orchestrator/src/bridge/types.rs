use chrono::{DateTime, Utc};
use cortex::{BacklinkKind, Memory, MemoryId, MemoryKind, SearchHit};
use serde::{Deserialize, Serialize};

use crate::error::OrchestratorError;

/// Lien sortant exposé dans les listes mémoire (trous de ver UI).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BacklinkSummary {
    /// Mémoire cible.
    pub target: MemoryId,
    /// Score de pertinence ∈ [0.0, 1.0].
    pub score: f32,
    /// `semantic` ou `explicit_wikilink`.
    pub kind: String,
}

/// Vue légère d'une mémoire pour listes virtualisées (HUD / TUI).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemorySummary {
    /// Identifiant unique.
    pub id: MemoryId,
    /// Titre affiché.
    pub title: String,
    /// Tags normalisés (chaînes).
    pub tags: Vec<String>,
    /// Date de création UTC.
    pub created_at: DateTime<Utc>,
    /// Date de dernière modification UTC.
    pub updated_at: DateTime<Utc>,
    /// Nombre de backlinks sortants.
    pub backlink_count: usize,
    /// Cibles des backlinks sortants (pour graphe cosmique).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub backlinks: Vec<BacklinkSummary>,
    /// Type sémantique du souvenir.
    #[serde(default)]
    pub kind: MemoryKind,
}

impl MemorySummary {
    /// Construit un résumé depuis une entité [`Memory`] complète.
    #[must_use]
    pub fn from_memory(memory: &Memory) -> Self {
        Self {
            id: memory.id,
            title: memory.title.clone(),
            tags: memory
                .tags
                .iter()
                .map(|tag| tag.as_str().to_string())
                .collect(),
            created_at: memory.created_at,
            updated_at: memory.updated_at,
            backlink_count: memory.backlink_count(),
            kind: memory.kind,
            backlinks: memory
                .backlinks
                .iter()
                .map(|bl| BacklinkSummary {
                    target: bl.target,
                    score: bl.score,
                    kind: match bl.kind {
                        BacklinkKind::Semantic => "semantic".to_string(),
                        BacklinkKind::ExplicitWikilink => "explicit_wikilink".to_string(),
                    },
                })
                .collect(),
        }
    }

    /// Indique si le résumé correspond à un filtre textuel (titre ou tags).
    #[must_use]
    pub fn matches_filter(&self, needle: &str) -> bool {
        let needle = needle.to_lowercase();
        self.title.to_lowercase().contains(&needle)
            || self
                .tags
                .iter()
                .any(|tag| tag.to_lowercase().contains(&needle))
    }

    /// Filtre par kind sémantique (`None` = tous).
    #[must_use]
    pub fn matches_kind(&self, kind: Option<MemoryKind>) -> bool {
        kind.is_none_or(|k| self.kind == k)
    }
}

/// Résumé léger d'un brouillon en attente de publication.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DraftSummary {
    /// Identifiant du brouillon.
    pub id: String,
    /// Titre candidat.
    pub title: String,
    /// Type sémantique.
    pub kind: MemoryKind,
    /// Tags.
    pub tags: Vec<String>,
    /// Statut du cycle de vie (`pending`, `published`, `discarded`).
    pub status: crate::draft::DraftStatus,
    /// Date de création UTC.
    pub created_at: DateTime<Utc>,
    /// Fichier session source (watcher).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "source_session"
    )]
    pub watcher_session: Option<String>,
}

/// Statut du watcher de sessions (bridge / UI).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WatcherStatus {
    /// Activé dans la configuration.
    pub enabled: bool,
    /// Tâche de surveillance en cours.
    pub running: bool,
    /// Répertoires surveillés.
    pub watch_dirs: Vec<String>,
    /// Sessions traitées depuis le démarrage.
    pub sessions_processed: usize,
    /// Brouillons créés depuis le démarrage.
    pub drafts_created: usize,
    /// Brouillons en attente (file disque).
    pub drafts_pending: usize,
    /// Dernière activité UTC.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_activity_at: Option<DateTime<Utc>>,
    /// Dernière erreur lisible.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
}

/// Résultat de recherche exposé au bridge (réutilise le type Cortex).
pub type BridgeSearchHit = SearchHit;

/// Contexte sérialisable pour `Command::ExecuteSkill`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BridgeSkillContext {
    /// Requête de recherche (`search`).
    pub query: Option<String>,
    /// Texte à assimiler (`assimilate`).
    pub text: Option<String>,
    /// Tags suggérés ou filtre.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Limite de résultats (`search`).
    pub limit: Option<usize>,
}

/// Métadonnées d'une skill exposée au bridge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillSummary {
    /// Identifiant stable de la skill.
    pub name: String,
    /// Description lisible.
    pub description: String,
    /// Origine : `builtin` ou `hub`.
    pub source: String,
    /// Version optionnelle (plugins hub).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// Entrée du catalogue marketplace exposée au bridge (Phase 14).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketplaceEntrySummary {
    /// Identifiant stable.
    pub id: String,
    /// Nom affiché.
    pub name: String,
    /// Description.
    pub description: String,
    /// Version catalogue.
    pub version: String,
    /// Installable via sync.
    pub enabled: bool,
}

/// Rapport d'intégrité hub pour le bridge (Phase 14).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HubIntegritySummary {
    /// Manifestes valides.
    pub valid_count: usize,
    /// Chemins invalides avec message d'erreur.
    pub invalid: Vec<(String, String)>,
}

/// Hub du graphe de connaissances (backlinks entrants).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HubSummary {
    /// Identifiant de la mémoire hub.
    pub memory_id: MemoryId,
    /// Titre affiché.
    pub title: String,
    /// Nombre de backlinks entrants.
    pub inbound_links: usize,
}

/// Erreur applicative sérialisable pour les réponses bridge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppError {
    /// Catégorie stable (`cortex`, `validation`, `security`, `llm`, …).
    pub kind: String,
    /// Message lisible par l'utilisateur ou le HUD.
    pub message: String,
}

impl AppError {
    /// Construit une erreur applicative depuis une [`OrchestratorError`].
    #[must_use]
    pub fn from_orchestrator(err: &OrchestratorError) -> Self {
        let kind = match err {
            OrchestratorError::Cortex(_) => "cortex",
            OrchestratorError::Embedding(_) => "embedding",
            OrchestratorError::Llm(_) => "llm",
            OrchestratorError::Validation(_) => "validation",
            OrchestratorError::Security(_) => "security",
            OrchestratorError::InsightSkipped { .. } => "insight_skipped",
            OrchestratorError::Internal(_) => "internal",
            OrchestratorError::Draft(_) => "draft",
        };
        Self {
            kind: kind.to_string(),
            message: err.to_string(),
        }
    }
}
