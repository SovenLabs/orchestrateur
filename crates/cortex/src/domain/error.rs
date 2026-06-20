use thiserror::Error;

use super::MemoryId;

/// Erreurs du domaine Cortex — explicites, jamais paniquantes.
#[derive(Debug, Error, PartialEq)]
pub enum CortexError {
    #[error("identifiant mémoire invalide: {0}")]
    InvalidMemoryId(String),

    #[error("tag invalide: {0}")]
    InvalidTag(String),

    #[error("tag trop long (> 64 caractères)")]
    TagTooLong,

    #[error("mémoire introuvable: {0}")]
    MemoryNotFound(MemoryId),

    #[error("format Markdown invalide: {0}")]
    InvalidMarkdown(String),

    #[error("frontmatter YAML invalide: {0}")]
    InvalidFrontmatter(String),

    #[error("titre mémoire vide")]
    EmptyTitle,

    #[error("contenu mémoire vide")]
    EmptyContent,

    #[error("backlink invalide: {0}")]
    InvalidBacklink(String),

    #[error("graphe: {0}")]
    GraphError(String),
}