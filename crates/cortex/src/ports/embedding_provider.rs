use async_trait::async_trait;
use thiserror::Error;

/// Vecteur d'embedding — value object du domaine.
#[derive(Clone, Debug, PartialEq)]
pub struct Embedding(pub Vec<f32>);

impl Embedding {
    /// Crée un embedding à partir d'un vecteur brut.
    #[must_use]
    pub fn new(vector: Vec<f32>) -> Self {
        Self(vector)
    }

    /// Nombre de dimensions du vecteur.
    #[must_use]
    pub fn dimensions(&self) -> usize {
        self.0.len()
    }

    /// Accès en lecture au vecteur sous-jacent.
    #[must_use]
    pub fn as_slice(&self) -> &[f32] {
        &self.0
    }

    /// Consomme l'embedding et retourne le vecteur brut.
    #[must_use]
    pub fn into_vec(self) -> Vec<f32> {
        self.0
    }
}

/// Capacités déclarées d'un provider d'embeddings.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EmbeddingCapabilities {
    /// Le provider supporte le batch natif (plus efficace).
    pub supports_batch: bool,
    /// Taille maximale recommandée pour un batch.
    pub max_batch_size: Option<usize>,
    /// Le provider accepte une instruction / task spécifique.
    pub supports_instruction: bool,
    /// Dimensions typiques produites (pour validation).
    pub typical_dimensions: Option<usize>,
}

/// Erreur spécifique aux providers d'embeddings.
#[derive(Debug, Error)]
pub enum EmbeddingError {
    /// Provider indisponible (processus arrêté, endpoint down).
    #[error("provider {provider} indisponible: {message}")]
    Unavailable {
        /// Nom du provider.
        provider: String,
        /// Détail lisible.
        message: String,
    },

    /// Erreur réseau ou transport.
    #[error("erreur réseau avec {provider}: {message}")]
    Network {
        /// Nom du provider.
        provider: String,
        /// Détail de l'erreur.
        message: String,
        /// Source optionnelle pour chaînage d'erreurs.
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Réponse du provider invalide ou non parseable.
    #[error("réponse invalide du provider {provider}: {message}")]
    InvalidResponse {
        /// Nom du provider.
        provider: String,
        /// Détail de l'erreur.
        message: String,
    },

    /// Texte d'entrée trop long pour le modèle.
    #[error("texte trop long pour le provider {provider} (max {max} tokens)")]
    TextTooLong {
        /// Nom du provider.
        provider: String,
        /// Limite en tokens.
        max: usize,
    },

    /// Erreur interne non classifiée côté provider.
    #[error("erreur interne du provider {provider}: {message}")]
    Internal {
        /// Nom du provider.
        provider: String,
        /// Détail de l'erreur.
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

    /// Modèle ou service surchargé (HTTP 503/529…).
    #[error("modèle surchargé pour {provider}")]
    ModelOverloaded {
        /// Nom du provider.
        provider: String,
    },
}

impl EmbeddingError {
    /// Indique si la chaîne de fallback doit tenter le provider suivant.
    #[must_use]
    pub fn should_fallback(&self) -> bool {
        matches!(
            self,
            Self::Unavailable { .. }
                | Self::Network { .. }
                | Self::RateLimited { .. }
                | Self::ModelOverloaded { .. }
        )
    }
}

/// Port de génération d'embeddings vectoriels — provider-agnostic.
///
/// # Ajouter un nouveau provider en 3 étapes
///
/// 1. Créer une struct dans `infrastructure` (ex. `FastEmbedProvider`) avec client partagé.
/// 2. Implémenter ce trait (`name`, `capabilities`, `embed`, surcharger `embed_batch` si natif).
/// 3. Enregistrer le provider dans la factory TOML (`[providers]` + section dédiée).
///
/// Implémentations prévues : `Ollama`, `fastembed`, `OpenAI`, `xAI` (futur).
///
/// # Streaming / extensions futures
///
/// Le streaming d'embeddings n'est pas requis en Phase 3. Une extension future pourra
/// ajouter un trait séparé `EmbeddingStreamProvider` sans breaking change sur ce port.
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Nom lisible du provider (logs, événements, debugging).
    fn name(&self) -> &'static str;

    /// Capacités déclarées du provider (découverte dynamique).
    fn capabilities(&self) -> EmbeddingCapabilities;

    /// Génère un embedding pour un texte unique.
    ///
    /// # Errors
    ///
    /// Retourne [`EmbeddingError`] si le provider échoue.
    async fn embed(&self, text: &str) -> Result<Embedding, EmbeddingError>;

    /// Génère plusieurs embeddings en une seule requête.
    ///
    /// Les providers sans batch natif peuvent surcharger cette méthode ;
    /// le défaut appelle `embed` en boucle.
    ///
    /// # Errors
    ///
    /// Retourne [`EmbeddingError`] si une entrée du lot échoue.
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Embedding>, EmbeddingError> {
        let mut out = Vec::with_capacity(texts.len());
        for text in texts {
            out.push(self.embed(text).await?);
        }
        Ok(out)
    }

    /// Embedding avec instruction (modèles type Qwen/Snowflake embedding).
    ///
    /// Par défaut, ignore l'instruction et délègue à [`Self::embed`].
    ///
    /// # Errors
    ///
    /// Retourne [`EmbeddingError`] si le provider échoue.
    async fn embed_with_instruction(
        &self,
        text: &str,
        instruction: &str,
    ) -> Result<Embedding, EmbeddingError> {
        let _ = instruction;
        self.embed(text).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubProvider;

    #[async_trait]
    impl EmbeddingProvider for StubProvider {
        fn name(&self) -> &'static str {
            "stub"
        }

        fn capabilities(&self) -> EmbeddingCapabilities {
            EmbeddingCapabilities {
                typical_dimensions: Some(3),
                ..Default::default()
            }
        }

        async fn embed(&self, text: &str) -> Result<Embedding, EmbeddingError> {
            let v = vec![text.len() as f32, 0.0, 1.0];
            Ok(Embedding::new(v))
        }
    }

    #[test]
    fn should_fallback_on_transient_errors_only() {
        assert!(EmbeddingError::RateLimited {
            provider: "ollama".into()
        }
        .should_fallback());
        assert!(!EmbeddingError::AuthenticationFailed {
            provider: "ollama".into()
        }
        .should_fallback());
    }

    #[tokio::test]
    async fn default_batch_delegates_to_embed() {
        let provider = StubProvider;
        let batch = provider.embed_batch(&["a", "bb"]).await.unwrap();
        assert_eq!(batch.len(), 2);
        assert_eq!(batch[0].dimensions(), 3);
    }

    #[tokio::test]
    async fn default_instruction_delegates_to_embed() {
        let provider = StubProvider;
        let e = provider
            .embed_with_instruction("hello", "task")
            .await
            .unwrap();
        assert_eq!(e.dimensions(), 3);
    }
}
