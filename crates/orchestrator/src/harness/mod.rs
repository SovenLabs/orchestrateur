//! Utilitaires harness partagés (sondes, bootstrap).

mod probe;

pub use probe::{probe_health, probe_harness_services, HarnessServiceProbe, ServiceHealth};