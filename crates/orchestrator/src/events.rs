use cortex::DomainEvent;

use crate::llm::LlmUsageRecorded;

/// Publie les événements de domaine vers des consommateurs externes (logs, UI, bus futur).
pub trait EventPublisher: Send + Sync {
    /// Publie un lot d'événements Cortex.
    fn publish(&self, events: &[DomainEvent]);

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
