use std::sync::{Arc, Mutex};

use cortex::DomainEvent;

use crate::events::{B212Event, EventPublisher};
use crate::llm::LlmUsageRecorded;

/// Publisher de test qui enregistre tous les événements pour assertions.
#[derive(Debug, Default)]
pub struct CollectingEventPublisher {
    domain: Mutex<Vec<DomainEvent>>,
    b212: Mutex<Vec<B212Event>>,
    llm_usage: Mutex<Vec<LlmUsageRecorded>>,
}

impl CollectingEventPublisher {
    /// Crée un collecteur vide.
    #[must_use]
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Événements de domaine capturés.
    pub fn domain_events(&self) -> Vec<DomainEvent> {
        self.domain.lock().map(|g| g.clone()).unwrap_or_default()
    }

    /// Événements B212 capturés.
    pub fn b212_events(&self) -> Vec<B212Event> {
        self.b212.lock().map(|g| g.clone()).unwrap_or_default()
    }

    /// Traces d'usage LLM capturées.
    pub fn llm_usage_events(&self) -> Vec<LlmUsageRecorded> {
        self.llm_usage.lock().map(|g| g.clone()).unwrap_or_default()
    }

    /// Nombre d'assimilations enregistrées.
    pub fn assimilation_count(&self) -> usize {
        self.domain_events()
            .iter()
            .filter(|e| matches!(e, DomainEvent::MemoryAssimilated(_)))
            .count()
    }
}

impl EventPublisher for CollectingEventPublisher {
    fn publish(&self, events: &[DomainEvent]) {
        if let Ok(mut guard) = self.domain.lock() {
            guard.extend_from_slice(events);
        }
    }

    fn publish_b212(&self, events: &[B212Event]) {
        if let Ok(mut guard) = self.b212.lock() {
            guard.extend_from_slice(events);
        }
    }

    fn publish_llm_usage(&self, usage: &LlmUsageRecorded) {
        if let Ok(mut guard) = self.llm_usage.lock() {
            guard.push(usage.clone());
        }
    }
}
