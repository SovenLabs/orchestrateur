use thiserror::Error;

use super::MemoryId;

/// Erreurs du domaine Cortex — explicites, jamais paniquantes.
#[derive(Debug, Error, PartialEq)]
pub enum CortexError {
    /// UUID mémoire mal formé ou non parseable.
    #[error("identifiant mémoire invalide: {0}")]
    InvalidMemoryId(String),

    /// Tag vide, avec espaces ou caractères interdits.
    #[error("tag invalide: {0}")]
    InvalidTag(String),

    /// Tag dépassant [`crate::Tag`] max 64 caractères.
    #[error("tag trop long (> 64 caractères)")]
    TagTooLong,

    /// Aucune mémoire pour cet identifiant.
    #[error("mémoire introuvable: {0}")]
    MemoryNotFound(MemoryId),

    /// Structure Markdown non conforme au format canonique.
    #[error("format Markdown invalide: {0}")]
    InvalidMarkdown(String),

    /// YAML du frontmatter invalide ou champs manquants.
    #[error("frontmatter YAML invalide: {0}")]
    InvalidFrontmatter(String),

    /// Titre absent ou blanc uniquement.
    #[error("titre mémoire vide")]
    EmptyTitle,

    /// Corps absent ou blanc uniquement.
    #[error("contenu mémoire vide")]
    EmptyContent,

    /// Score hors bornes ou cible invalide.
    #[error("backlink invalide: {0}")]
    InvalidBacklink(String),

    /// Intégrité du graphe de connaissances compromise.
    #[error("graphe: {0}")]
    GraphError(String),

    /// Clé de session agent invalide.
    #[error("clé de session invalide: {0}")]
    InvalidSessionKey(String),

    /// Session introuvable.
    #[error("session introuvable: {0}")]
    SessionNotFound(String),
}
