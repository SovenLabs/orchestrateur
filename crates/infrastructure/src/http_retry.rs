use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use reqwest::Response;
use tokio::sync::Mutex;
use tracing::warn;

const BREAKER_THRESHOLD: u32 = 5;
const BREAKER_COOLDOWN: Duration = Duration::from_secs(30);
const BASE_DELAY_MS: u64 = 200;

/// Circuit ouvert — le provider est temporairement isolé.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CircuitOpen {
    /// Nom du provider concerné.
    pub provider: String,
    /// Secondes restantes avant réouverture.
    pub retry_after_secs: u64,
}

/// Circuit breaker léger par provider (compteur d'échecs + cooldown).
#[derive(Debug)]
pub struct CircuitBreaker {
    consecutive_failures: AtomicU32,
    open_until: Mutex<Option<Instant>>,
    failure_threshold: u32,
    cooldown: Duration,
}

impl CircuitBreaker {
    /// Crée un breaker avec seuil et cooldown configurables.
    #[must_use]
    pub fn new(failure_threshold: u32, cooldown: Duration) -> Self {
        Self {
            consecutive_failures: AtomicU32::new(0),
            open_until: Mutex::new(None),
            failure_threshold,
            cooldown,
        }
    }

    /// Vérifie si le circuit autorise une requête.
    ///
    /// # Errors
    ///
    /// Retourne [`CircuitOpen`] si le cooldown n'est pas expiré.
    pub async fn check(&self, provider: &str) -> Result<(), CircuitOpen> {
        let mut guard = self.open_until.lock().await;
        if let Some(until) = *guard {
            let now = Instant::now();
            if now < until {
                let secs = until.saturating_duration_since(now).as_secs().max(1);
                return Err(CircuitOpen {
                    provider: provider.into(),
                    retry_after_secs: secs,
                });
            }
            *guard = None;
            self.consecutive_failures.store(0, Ordering::SeqCst);
        }
        Ok(())
    }

    /// Réinitialise le compteur après un succès HTTP.
    pub async fn record_success(&self) {
        self.consecutive_failures.store(0, Ordering::SeqCst);
        *self.open_until.lock().await = None;
    }

    /// Incrémente les échecs et ouvre le circuit si le seuil est atteint.
    pub async fn record_failure(&self) {
        let failures = self.consecutive_failures.fetch_add(1, Ordering::SeqCst) + 1;
        if failures >= self.failure_threshold {
            *self.open_until.lock().await = Some(Instant::now() + self.cooldown);
            warn!(
                failures,
                cooldown_secs = self.cooldown.as_secs(),
                "circuit breaker ouvert"
            );
        }
    }
}

fn breakers() -> &'static Mutex<HashMap<String, Arc<CircuitBreaker>>> {
    static STORE: OnceLock<Mutex<HashMap<String, Arc<CircuitBreaker>>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

async fn breaker_for(provider: &str) -> Arc<CircuitBreaker> {
    let mut map = breakers().lock().await;
    map.entry(provider.to_string())
        .or_insert_with(|| Arc::new(CircuitBreaker::new(BREAKER_THRESHOLD, BREAKER_COOLDOWN)))
        .clone()
}

fn jittered_delay(attempt: u32) -> Duration {
    let base = BASE_DELAY_MS.saturating_mul(1u64 << attempt.min(6));
    let jitter = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| u64::from(d.subsec_millis())))
        % BASE_DELAY_MS;
    Duration::from_millis(base.saturating_add(jitter))
}

fn is_retriable_status(status: reqwest::StatusCode) -> bool {
    status.as_u16() == 429 || status.is_server_error()
}

/// Erreur HTTP résiliente (réseau ou circuit ouvert).
#[derive(Debug)]
pub enum HttpRetryError {
    /// Circuit breaker ouvert pour ce provider.
    CircuitOpen(CircuitOpen),
    /// Erreur réseau `reqwest`.
    Request(reqwest::Error),
}

impl std::fmt::Display for HttpRetryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CircuitOpen(c) => {
                write!(
                    f,
                    "circuit ouvert pour {} (retry dans {}s)",
                    c.provider, c.retry_after_secs
                )
            }
            Self::Request(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for HttpRetryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CircuitOpen(_) => None,
            Self::Request(e) => Some(e),
        }
    }
}

/// Exécute une requête HTTP avec retry exponentiel, jitter et circuit breaker.
///
/// # Errors
///
/// Retourne [`HttpRetryError`] si le circuit est ouvert ou si toutes les tentatives échouent.
pub async fn with_retry<F, Fut>(
    provider: &str,
    max_retries: u32,
    mut operation: F,
) -> Result<Response, HttpRetryError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<Response, reqwest::Error>>,
{
    let breaker = breaker_for(provider).await;
    breaker
        .check(provider)
        .await
        .map_err(HttpRetryError::CircuitOpen)?;

    let mut attempt = 0u32;
    loop {
        match operation().await {
            Ok(resp) if is_retriable_status(resp.status()) && attempt < max_retries => {
                attempt += 1;
                breaker.record_failure().await;
                let delay = jittered_delay(attempt);
                warn!(
                    provider,
                    attempt,
                    status = %resp.status(),
                    delay_ms = delay.as_millis(),
                    "retry après erreur HTTP retriable"
                );
                tokio::time::sleep(delay).await;
            }
            Ok(resp) => {
                if resp.status().is_success() {
                    breaker.record_success().await;
                } else if resp.status().is_client_error() && resp.status().as_u16() != 429 {
                    breaker.record_failure().await;
                }
                return Ok(resp);
            }
            Err(e) if attempt < max_retries => {
                attempt += 1;
                breaker.record_failure().await;
                let delay = jittered_delay(attempt);
                warn!(
                    provider,
                    attempt,
                    error = %e,
                    delay_ms = delay.as_millis(),
                    "retry après erreur réseau"
                );
                tokio::time::sleep(delay).await;
            }
            Err(e) => {
                breaker.record_failure().await;
                return Err(HttpRetryError::Request(e));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn breaker_opens_after_threshold() {
        let breaker = CircuitBreaker::new(2, Duration::from_millis(50));
        breaker.record_failure().await;
        breaker.record_failure().await;
        assert!(breaker.check("test").await.is_err());
        tokio::time::sleep(Duration::from_millis(60)).await;
        assert!(breaker.check("test").await.is_ok());
    }
}