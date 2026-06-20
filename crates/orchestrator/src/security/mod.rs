//! Défense en profondeur — couches de sécurité adversariale (Phase 3+).
//!
//! La validation adversariale des brouillons vit dans [`cortex::MemoryDraftValidator`].
//! Ce module orchestre profils, garde comportemental, audit et honeypots.

mod audit_log;
mod behavioral_guard;
mod context;
mod honeypot;
mod integrity;
mod profile;

pub use audit_log::{AuditEvent, AuditLog, AUDIT_GENESIS};
pub use behavioral_guard::{BehavioralError, BehavioralGuard, GuardAction};
pub use context::{
    build_security_context, build_test_security_context, DegradedModeError, SecurityBootstrapError,
    SecurityContext, SecurityGateError,
};
pub use cortex::{
    MemoryDraftValidator, MemoryDraftValidatorConfig, ValidationError,
};
pub use honeypot::{is_honeypot_memory, seed_honeypots_if_needed, CANARY_TAG};
pub use integrity::{verify_config_integrity, IntegrityError, IntegrityManifest, IntegrityStatus};
pub use profile::SecurityProfile;