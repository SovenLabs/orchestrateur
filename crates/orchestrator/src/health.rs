//! Sondes de disponibilité des providers (health bridge, sans dépendance infrastructure).

use std::time::Duration;

use cortex::EmbeddingProvider;
use tokio::time::timeout;

use crate::deps::AppDependencies;
use crate::llm::{ChatMessage, LlmProvider};

/// Résultat des sondes exposé au HUD / TUI via [`crate::bridge::Response::Health`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServiceProbe {
    /// Provider LLM joignable (ou mock in-memory en tests).
    pub llm_available: bool,
    /// Provider d'embeddings joignable.
    pub embedding_available: bool,
}

/// Sonde les providers injectés (timeout court, non bloquant pour l'UI).
pub async fn probe_services(deps: &AppDependencies) -> ServiceProbe {
    const PROBE_TIMEOUT: Duration = Duration::from_secs(3);

    let llm_available: bool = timeout(PROBE_TIMEOUT, probe_llm(deps.llm.as_ref()))
        .await
        .unwrap_or_default();
    let embedding_available: bool =
        timeout(PROBE_TIMEOUT, probe_embedding(deps.embedding.as_ref()))
            .await
            .unwrap_or_default();

    ServiceProbe {
        llm_available,
        embedding_available,
    }
}

async fn probe_llm(llm: &dyn LlmProvider) -> bool {
    match llm.name() {
        "unavailable" => false,
        "in-memory-llm" => true,
        _ => llm
            .chat(&[ChatMessage {
                role: "user".into(),
                content: ".".into(),
            }])
            .await
            .is_ok(),
    }
}

async fn probe_embedding(embedding: &dyn EmbeddingProvider) -> bool {
    match embedding.name() {
        "unavailable" => false,
        "in-memory" => true,
        _ => embedding.embed("health-check").await.is_ok(),
    }
}
