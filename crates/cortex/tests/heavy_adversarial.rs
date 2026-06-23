//! Tests adversariaux très coûteux — nécessitent `--features heavy-tests`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use cortex::{MemoryDraft, MemoryDraftValidator, MemoryDraftValidatorConfig};

/// Placeholder pour futurs scénarios adversariaux multi-passes (LLM réel, gros corpus).
#[test]
fn heavy_adversarial_validation_smoke() {
    let validator = MemoryDraftValidator::from_config(&MemoryDraftValidatorConfig::default());
    let mut draft = MemoryDraft::new(
        "Smoke heavy",
        "Validation adversarial complète sur gros volume.",
    );
    draft.tags = vec!["security".into()];
    validator.validate(&draft).expect("draft sain");
}