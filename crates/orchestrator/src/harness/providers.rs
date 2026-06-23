//! Sondes providers LLM / embedding.

use crate::health::probe_services;
use crate::harness::error::HarnessError;
use crate::harness::types::ProviderProbeResult;
use crate::OrchestratorFacade;

/// Sonde les providers configurés.
pub async fn probe_providers(facade: &OrchestratorFacade) -> ProviderProbeResult {
    let probe = probe_services(facade.deps()).await;
    let config = &facade.deps().config;
    ProviderProbeResult {
        llm_ok: probe.llm_available,
        embedding_ok: probe.embedding_available,
        llm_id: config.providers.primary_llm.clone(),
        embedding_id: config.providers.primary_embedding.clone(),
    }
}

/// Valide la sonde selon le filtre (`llm`, `embedding`, ou tous).
pub fn validate_probe(result: &ProviderProbeResult, kind: Option<&str>) -> Result<(), HarnessError> {
    match kind {
        Some("llm") if !result.llm_ok => {
            Err(HarnessError::ProviderProbe("LLM échouée".into()))
        }
        Some("embedding") if !result.embedding_ok => {
            Err(HarnessError::ProviderProbe("embedding échouée".into()))
        }
        Some(other) => Err(HarnessError::ProviderProbe(format!(
            "kind inconnu: {other} (llm ou embedding)"
        ))),
        None if !result.llm_ok && !result.embedding_ok => {
            Err(HarnessError::ProviderProbe("aucun provider joignable".into()))
        }
        _ => Ok(()),
    }
}