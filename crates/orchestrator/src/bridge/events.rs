use std::sync::{Arc, Mutex};

use cortex::DomainEvent;
use flume::Sender;

use crate::events::{B212Event, EventPublisher};
use crate::llm::LlmUsageRecorded;

/// Publisher fan-out : chaque abonné HUD reçoit une copie des événements Cortex et B212.
#[derive(Debug, Clone)]
pub struct FanoutEventPublisher {
    domain_subscribers: Arc<Mutex<Vec<Sender<DomainEvent>>>>,
    b212_subscribers: Arc<Mutex<Vec<Sender<B212Event>>>>,
}

impl Default for FanoutEventPublisher {
    fn default() -> Self {
        Self::new()
    }
}

impl FanoutEventPublisher {
    /// Crée un publisher sans abonné initial.
    #[must_use]
    pub fn new() -> Self {
        Self {
            domain_subscribers: Arc::new(Mutex::new(Vec::new())),
            b212_subscribers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Ajoute un abonné domaine et retourne son récepteur dédié.
    #[must_use]
    pub fn subscribe(&self) -> flume::Receiver<DomainEvent> {
        let (tx, rx) = flume::unbounded();
        if let Ok(mut guard) = self.domain_subscribers.lock() {
            guard.push(tx);
        }
        rx
    }

    /// Ajoute un abonné B212 et retourne son récepteur dédié.
    #[must_use]
    pub fn subscribe_b212(&self) -> flume::Receiver<B212Event> {
        let (tx, rx) = flume::unbounded();
        if let Ok(mut guard) = self.b212_subscribers.lock() {
            guard.push(tx);
        }
        rx
    }
}

impl EventPublisher for FanoutEventPublisher {
    fn publish(&self, events: &[DomainEvent]) {
        let Ok(mut guard) = self.domain_subscribers.lock() else {
            return;
        };
        for event in events {
            guard.retain(|tx| tx.send(event.clone()).is_ok());
        }
    }

    fn publish_b212(&self, events: &[B212Event]) {
        let Ok(mut guard) = self.b212_subscribers.lock() else {
            return;
        };
        for event in events {
            guard.retain(|tx| tx.send(event.clone()).is_ok());
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
    use cortex::MemoryId;

    use super::*;

    #[test]
    fn fanout_delivers_domain_to_subscriber() {
        let publisher = FanoutEventPublisher::new();
        let rx = publisher.subscribe();
        let event = DomainEvent::memory_assimilated(MemoryId::new(), 2);
        publisher.publish(&[event.clone()]);
        let received = rx.recv_timeout(std::time::Duration::from_millis(100));
        assert_eq!(received.ok(), Some(event));
    }

    #[test]
    fn fanout_delivers_b212_to_subscriber() {
        let publisher = FanoutEventPublisher::new();
        let rx = publisher.subscribe_b212();
        let event = B212Event::ProposalApproved {
            proposal_id: "b212-test".into(),
        };
        publisher.publish_b212(&[event.clone()]);
        let received = rx.recv_timeout(std::time::Duration::from_millis(100));
        assert_eq!(received.ok(), Some(event));
    }
}