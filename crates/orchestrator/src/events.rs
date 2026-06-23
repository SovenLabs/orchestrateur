use cortex::DomainEvent;
use serde::{Deserialize, Serialize};

use crate::llm::LlmUsageRecorded;

/// Événements protocole B212 (optionnels, WS / HUD).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum B212Event {
    /// Analyse setup complétée.
    AnalysisComplete {
        /// Symbole analysé.
        symbol: String,
        /// Session.
        session: String,
        /// Proposition créée (si éligible).
        #[serde(skip_serializing_if = "Option::is_none")]
        proposal_id: Option<String>,
    },
    /// Proposition HITL créée.
    ProposalCreated {
        /// Identifiant proposition.
        proposal_id: String,
        /// Symbole.
        symbol: String,
        /// Direction.
        side: String,
    },
    /// Proposition approuvée.
    ProposalApproved {
        /// Identifiant proposition.
        proposal_id: String,
    },
    /// Proposition rejetée.
    ProposalRejected {
        /// Identifiant proposition.
        proposal_id: String,
    },
    /// Fill paper exécuté.
    SimExecuted {
        /// Identifiant proposition.
        proposal_id: String,
        /// Identifiant fill.
        fill_id: String,
        /// Prix d'entrée.
        entry_price: f64,
        /// Notionnel USD.
        notional_usd: f64,
    },
}

/// Publie les événements de domaine vers des consommateurs externes (logs, UI, bus futur).
pub trait EventPublisher: Send + Sync {
    /// Publie un lot d'événements Cortex.
    fn publish(&self, events: &[DomainEvent]);

    /// Publie des événements B212 (no-op par défaut).
    fn publish_b212(&self, events: &[B212Event]) {
        let _ = events;
    }

    /// Publie une trace d'usage LLM (coût, tokens, traçabilité long terme).
    fn publish_llm_usage(&self, usage: &LlmUsageRecorded) {
        let _ = usage;
    }
}

/// Publisher silencieux — utile en tests ou quand aucun consommateur n'est branché.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopEventPublisher;

impl EventPublisher for NoopEventPublisher {
    fn publish(&self, _events: &[DomainEvent]) {}
}

/// Publisher qui trace les événements via [`tracing`].
#[derive(Debug, Clone, Copy, Default)]
pub struct TracingEventPublisher;

impl EventPublisher for TracingEventPublisher {
    fn publish(&self, events: &[DomainEvent]) {
        for event in events {
            tracing::info!(?event, "domain_event");
        }
    }

    fn publish_llm_usage(&self, usage: &LlmUsageRecorded) {
        tracing::info!(
            provider = %usage.provider,
            operation = %usage.operation,
            prompt_tokens = ?usage.prompt_tokens,
            completion_tokens = ?usage.completion_tokens,
            "llm_usage_recorded"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cortex::MemoryId;

    #[test]
    fn noop_does_not_panic() {
        NoopEventPublisher.publish(&[DomainEvent::memory_assimilated(MemoryId::new(), 0)]);
    }

    #[test]
    fn tracing_publisher_accepts_events() {
        TracingEventPublisher.publish(&[DomainEvent::memory_assimilated(MemoryId::new(), 1)]);
    }
}
