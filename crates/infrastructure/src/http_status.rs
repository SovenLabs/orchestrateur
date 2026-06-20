use cortex::EmbeddingError;
use orchestrator::LlmError;

/// Classifie une réponse HTTP provider en [`LlmError`].
#[must_use]
pub fn map_llm_http_status(provider: &str, status: reqwest::StatusCode) -> LlmError {
    match status.as_u16() {
        401 | 403 => LlmError::AuthenticationFailed {
            provider: provider.into(),
        },
        429 => LlmError::RateLimited {
            provider: provider.into(),
        },
        503 | 529 => LlmError::ModelOverloaded {
            provider: provider.into(),
        },
        _ if status.is_server_error() => LlmError::Unavailable {
            provider: provider.into(),
            message: format!("HTTP {status}"),
        },
        _ => LlmError::ProviderError {
            provider: provider.into(),
            message: format!("HTTP {status}"),
        },
    }
}

/// Classifie une réponse HTTP provider en [`EmbeddingError`].
#[must_use]
pub fn map_embedding_http_status(provider: &str, status: reqwest::StatusCode) -> EmbeddingError {
    match status.as_u16() {
        401 | 403 => EmbeddingError::AuthenticationFailed {
            provider: provider.into(),
        },
        429 => EmbeddingError::RateLimited {
            provider: provider.into(),
        },
        503 | 529 => EmbeddingError::ModelOverloaded {
            provider: provider.into(),
        },
        _ if status.is_server_error() => EmbeddingError::Unavailable {
            provider: provider.into(),
            message: format!("HTTP {status}"),
        },
        _ => EmbeddingError::InvalidResponse {
            provider: provider.into(),
            message: format!("HTTP {status}"),
        },
    }
}
