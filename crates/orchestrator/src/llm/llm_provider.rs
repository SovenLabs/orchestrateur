use async_trait::async_trait;
use thiserror::Error;

use crate::memory_draft::MemoryDraft;

/// Message d'une conversation LLM.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatMessage {
    /// Rôle (`system`, `user`, `assistant`, `tool`…).
    pub role: String,
    /// Contenu textuel du message.
    pub content: String,
}

/// Capacités déclarées d'un provider LLM.
///
/// Prépare l'extension future (`cost_per_1k_tokens`, fenêtre de contexte dynamique, etc.).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LlmCapabilities {
    /// Supporte les Structured Outputs (JSON Schema strict).
    pub supports_structured_output: bool,
    /// Supporte le tool calling / function calling.
    pub supports_tools: bool,
    /// Taille maximale du contexte en tokens.
    pub max_context_tokens: Option<u32>,
    /// Supporte le streaming de réponses.
    pub supports_streaming: bool,
}

/// Erreurs des providers LLM (couche orchestrator).
#[derive(Debug, Error, PartialEq, Eq)]
pub enum LlmError {
    /// Échec générique du provider.
    #[error("provider {provider} a échoué: {message}")]
    ProviderError {
        /// Nom du provider.
        provider: String,
        /// Détail lisible.
        message: String,
    },

    /// Structured Output invalide ou non conforme au schéma `MemoryDraft`.
    #[error("structured output invalide pour {provider}: {message}")]
    StructuredOutputInvalid {
        /// Nom du provider.
        provider: String,
        /// Détail de validation.
        message: String,
    },

    /// Limite de débit atteinte.
    #[error("rate limit {provider}")]
    RateLimited {
        /// Nom du provider.
        provider: String,
    },

    /// Authentification refusée.
    #[error("authentification refusée pour {provider}")]
    AuthenticationFailed {
        /// Nom du provider.
        provider: String,
    },

    /// Provider indisponible.
    #[error("provider {provider} indisponible: {message}")]
    Unavailable {
        /// Nom du provider.
        provider: String,
        /// Détail lisible.
        message: String,
    },

    /// Modèle ou service surchargé (HTTP 503/529…).
    #[error("modèle surchargé pour {provider}")]
    ModelOverloaded {
        /// Nom du provider.
        provider: String,
    },
}

/// Consommation de tokens LLM — traçabilité coût et usage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlmUsageRecorded {
    /// Nom du provider (`xai`, `ollama`…).
    pub provider: String,
    /// Opération (`generate_memory_draft`, `chat`…).
    pub operation: String,
    /// Tokens du prompt.
    pub prompt_tokens: Option<u32>,
    /// Tokens de complétion.
    pub completion_tokens: Option<u32>,
}

impl LlmError {
    /// Indique si la chaîne de fallback doit tenter le provider suivant.
    #[must_use]
    pub fn should_fallback(&self) -> bool {
        match self {
            Self::Unavailable { .. } | Self::RateLimited { .. } | Self::ModelOverloaded { .. } => {
                true
            }
            Self::ProviderError { message, .. } => {
                message.contains("HTTP 5") || message.contains("timeout")
            }
            Self::AuthenticationFailed { .. } | Self::StructuredOutputInvalid { .. } => false,
        }
    }
}

/// Port de génération LLM — provider-agnostic, manipule [`MemoryDraft`] uniquement.
///
/// # Ajouter un nouveau provider en 3 étapes
///
/// 1. Créer une struct dans `infrastructure` avec client HTTP partagé (`reqwest::Client`).
/// 2. Implémenter ce trait : `generate_memory_draft` produit du JSON désérialisé en [`MemoryDraft`].
/// 3. Enregistrer dans la factory TOML (`[providers]` + section provider).
///
/// # Validation
///
/// Le provider **ne valide pas** le domaine Cortex : il retourne un [`MemoryDraft`] brut.
/// Les validateurs Cortex (`into_memory()`, graphe, backlinks) s'exécutent dans le use case.
///
/// # Streaming (extension future)
///
/// Une méthode `stream_chat` pourra être ajoutée via trait séparé `LlmStreamingProvider`
/// ou méthode par défaut retournant `LlmError::Unavailable` — sans breaking change majeur.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Nom lisible du provider.
    fn name(&self) -> &'static str;

    /// Capacités déclarées (découverte dynamique).
    fn capabilities(&self) -> LlmCapabilities;

    /// Génère un [`MemoryDraft`] structuré — cœur du flux d'assimilation.
    ///
    /// # Errors
    ///
    /// Retourne [`LlmError`] si le provider ou la désérialisation JSON échoue.
    async fn generate_memory_draft(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<MemoryDraft, LlmError>;

    /// Chat libre (Thought Loop, CLI interactif — phases ultérieures).
    ///
    /// # Errors
    ///
    /// Retourne [`LlmError`] si le provider échoue.
    async fn chat(&self, messages: &[ChatMessage]) -> Result<String, LlmError>;

    /// Dernière consommation de tokens enregistrée par le provider (si disponible).
    fn last_usage(&self) -> Option<LlmUsageRecorded> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubLlm;

    #[async_trait]
    impl LlmProvider for StubLlm {
        fn name(&self) -> &'static str {
            "stub-llm"
        }

        fn capabilities(&self) -> LlmCapabilities {
            LlmCapabilities {
                supports_structured_output: true,
                ..Default::default()
            }
        }

        async fn generate_memory_draft(
            &self,
            _system: &str,
            user: &str,
        ) -> Result<MemoryDraft, LlmError> {
            Ok(MemoryDraft {
                title: "Stub".into(),
                content: user.into(),
                tags: vec![],
                backlinks: vec![],
            })
        }

        async fn chat(&self, messages: &[ChatMessage]) -> Result<String, LlmError> {
            Ok(messages
                .last()
                .map(|m| m.content.clone())
                .unwrap_or_default())
        }
    }

    #[test]
    fn should_fallback_on_transient_errors_only() {
        assert!(LlmError::RateLimited {
            provider: "xai".into()
        }
        .should_fallback());
        assert!(LlmError::ModelOverloaded {
            provider: "xai".into()
        }
        .should_fallback());
        assert!(!LlmError::AuthenticationFailed {
            provider: "xai".into()
        }
        .should_fallback());
        assert!(!LlmError::StructuredOutputInvalid {
            provider: "xai".into(),
            message: "bad json".into()
        }
        .should_fallback());
    }

    #[tokio::test]
    async fn stub_generates_memory_draft() {
        let llm = StubLlm;
        let draft = llm
            .generate_memory_draft("sys", "contenu utilisateur")
            .await
            .unwrap();
        assert_eq!(draft.content, "contenu utilisateur");
    }
}
