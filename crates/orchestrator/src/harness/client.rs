//! Client HTTP partagé pour sondes harness.

use std::time::Duration;

use reqwest::Client;

/// Client HTTP court timeout pour sondes /health.
#[must_use]
pub fn probe_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap_or_else(|_| Client::new())
}