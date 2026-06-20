use std::time::Duration;

use reqwest::Response;
use tracing::warn;

/// Exécute une requête HTTP avec retry exponentiel simple.
///
/// # Errors
///
/// Retourne la dernière erreur si toutes les tentatives échouent.
pub async fn with_retry<F, Fut>(
    provider: &str,
    max_retries: u32,
    mut operation: F,
) -> Result<Response, reqwest::Error>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<Response, reqwest::Error>>,
{
    let mut attempt = 0u32;
    loop {
        match operation().await {
            Ok(resp) if resp.status().is_server_error() && attempt < max_retries => {
                attempt += 1;
                let delay = Duration::from_millis(200 * u64::from(attempt));
                warn!(
                    provider,
                    attempt,
                    status = %resp.status(),
                    "retry après erreur serveur HTTP"
                );
                tokio::time::sleep(delay).await;
            }
            other => return other,
        }
    }
}