use std::sync::Arc;

use async_trait::async_trait;
use cortex::{EmbeddingProvider, MemoryRepository, SessionRepository, VectorStore};
use crate::config::OrchestratorConfig;
use crate::deps::AppDependencies;
use crate::llm::LlmProvider;
use serde_json::Value;
use thiserror::Error;

/// Contexte d'exécution partagé par tous les outils.
#[derive(Clone)]
pub struct ToolContext {
    /// Dépendances applicatives complètes.
    pub deps: AppDependencies,
}

impl ToolContext {
    /// Construit depuis [`AppDependencies`].
    #[must_use]
    pub fn new(deps: AppDependencies) -> Self {
        Self { deps }
    }

    /// Accès rapide au dépôt mémoires.
    #[must_use]
    pub fn memory_repo(&self) -> Arc<dyn MemoryRepository> {
        self.deps.memory_repo.clone()
    }

    /// Accès rapide au vector store.
    #[must_use]
    pub fn vector_store(&self) -> Arc<dyn VectorStore> {
        self.deps.vector_store.clone()
    }

    /// Accès rapide aux embeddings.
    #[must_use]
    pub fn embedding(&self) -> Arc<dyn EmbeddingProvider> {
        self.deps.embedding.clone()
    }

    /// Accès rapide au LLM.
    #[must_use]
    pub fn llm(&self) -> Arc<dyn LlmProvider> {
        self.deps.llm.clone()
    }

    /// Accès rapide aux sessions.
    #[must_use]
    pub fn session_repo(&self) -> Arc<dyn SessionRepository> {
        self.deps.session_repo.clone()
    }

    /// Configuration applicative.
    #[must_use]
    pub fn config(&self) -> &OrchestratorConfig {
        &self.deps.config
    }
}

/// Description d'un outil pour le prompt LLM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolDefinition {
    /// Nom unique (`memory_search`, …).
    pub name: &'static str,
    /// Description pour le modèle.
    pub description: &'static str,
    /// Schéma JSON des paramètres attendus.
    pub parameters_schema: &'static str,
}

/// Résultat d'exécution d'un outil (texte injecté dans la boucle agent).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolResult {
    /// Contenu textuel retourné au modèle.
    pub content: String,
}

/// Erreurs d'exécution d'outils.
#[derive(Debug, Error, PartialEq)]
pub enum ToolError {
    /// Outil introuvable.
    #[error("outil introuvable: {0}")]
    NotFound(String),

    /// Arguments JSON invalides.
    #[error("arguments invalides pour {tool}: {message}")]
    InvalidArguments {
        /// Nom de l'outil.
        tool: String,
        /// Détail.
        message: String,
    },

    /// Échec d'exécution.
    #[error("exécution {tool} échouée: {message}")]
    ExecutionFailed {
        /// Nom de l'outil.
        tool: String,
        /// Détail.
        message: String,
    },
}

/// Contrat d'un outil agent exécutable.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Nom stable de l'outil.
    fn name(&self) -> &'static str;

    /// Description pour le modèle.
    fn description(&self) -> &'static str;

    /// Schéma JSON des paramètres.
    fn parameters_schema(&self) -> &'static str;

    /// Définition exportable pour le prompt.
    #[must_use]
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name(),
            description: self.description(),
            parameters_schema: self.parameters_schema(),
        }
    }

    /// Exécute l'outil avec arguments JSON.
    ///
    /// # Errors
    ///
    /// Retourne [`ToolError`] si les arguments ou l'exécution échouent.
    async fn execute(&self, ctx: &ToolContext, args: &Value) -> Result<ToolResult, ToolError>;
}