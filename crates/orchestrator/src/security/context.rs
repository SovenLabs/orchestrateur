//! Contexte de sécurité partagé — orchestre les couches 2, 3 et 4.

use std::sync::Arc;

use cortex::{Memory, MemoryId, MemoryRepository};

use crate::config::OrchestratorConfig;

use super::audit_log::{AuditError, AuditLog};
use super::behavioral_guard::{BehavioralError, BehavioralGuard};
use super::honeypot::{is_honeypot_memory, seed_honeypots_if_needed};
use super::integrity::{verify_config_integrity, IntegrityError, IntegrityStatus};

/// Erreur opérationnelle liée au mode dégradé.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("mode dégradé: {0}")]
pub struct DegradedModeError(pub String);

/// Contexte de sécurité injecté dans [`crate::deps::AppDependencies`].
#[derive(Debug)]
pub struct SecurityContext {
    behavioral: BehavioralGuard,
    audit: AuditLog,
    integrity: IntegrityStatus,
}

impl SecurityContext {
    /// Initialise le contexte au démarrage (intégrité + audit).
    ///
    /// # Errors
    ///
    /// Propage [`IntegrityError`] ou [`AuditError`] si l'initialisation échoue.
    pub fn bootstrap(config: &OrchestratorConfig) -> Result<Self, SecurityBootstrapError> {
        let integrity = verify_config_integrity(config)?;
        let audit = AuditLog::open(&config.security.audit, &config.workspace_root)?;
        let behavioral = BehavioralGuard::new(config.security.behavioral.clone());

        let ctx = Self {
            behavioral,
            audit,
            integrity,
        };

        if let IntegrityStatus::Degraded { reason } = &ctx.integrity {
            ctx.audit_degraded(reason)?;
        } else {
            let _ = ctx.audit.record("startup", "integrity=healthy");
        }

        Ok(ctx)
    }

    /// Contexte pour tests unitaires — audit désactivé, limites élevées.
    #[must_use]
    pub fn for_tests(config: &OrchestratorConfig) -> Self {
        let mut behavioral_cfg = config.security.behavioral.clone();
        behavioral_cfg.enabled = false;
        Self {
            behavioral: BehavioralGuard::new(behavioral_cfg),
            audit: AuditLog::disabled(),
            integrity: IntegrityStatus::Healthy,
        }
    }

    /// Plante les honeypots si configuré.
    ///
    /// # Errors
    ///
    /// Propage les erreurs Cortex du dépôt.
    pub async fn seed_honeypots_if_needed(
        &self,
        repo: &dyn MemoryRepository,
        config: &OrchestratorConfig,
    ) -> Result<Vec<MemoryId>, cortex::CortexError> {
        if !config.security.integrity.enabled || !config.security.integrity.seed_honeypots {
            return Ok(Vec::new());
        }
        let ids = seed_honeypots_if_needed(repo, config.security.integrity.honeypot_count).await?;
        if !ids.is_empty() {
            let _ = self
                .audit
                .record("honeypot_seed", &format!("count={}", ids.len()));
        }
        Ok(ids)
    }

    /// Vérifie que les opérations sensibles sont autorisées (intégrité).
    ///
    /// # Errors
    ///
    /// Retourne [`DegradedModeError`] en mode dégradé.
    pub fn check_operational(&self) -> Result<(), DegradedModeError> {
        if let IntegrityStatus::Degraded { reason } = &self.integrity {
            return Err(DegradedModeError(reason.clone()));
        }
        Ok(())
    }

    /// Vérifie le garde avant assimilation (intégrité + comportement).
    ///
    /// # Errors
    ///
    /// Retourne [`SecurityGateError`] en mode dégradé ou si le rate limiting bloque.
    pub fn gate_assimilation(&self) -> Result<(), SecurityGateError> {
        self.check_operational()?;
        self.behavioral.check_assimilation()?;
        Ok(())
    }

    /// Vérifie le garde avant recherche.
    ///
    /// # Errors
    ///
    /// Retourne [`SecurityGateError`] si le rate limiting ou la répétition bloque.
    pub fn gate_search(&self, query: &str) -> Result<(), SecurityGateError> {
        self.behavioral.check_search(query)?;
        Ok(())
    }

    /// Enregistre une assimilation.
    pub fn record_assimilation(&self, title: &str) {
        self.behavioral.record_assimilation();
        let _ = self.audit.record("assimilate", &format!("title={title}"));
    }

    /// Enregistre une recherche.
    pub fn record_search(&self, query: &str, results: usize) {
        self.behavioral.record_search(query);
        let _ = self.audit.record(
            "search",
            &format!("query_len={} results={results}", query.len()),
        );
    }

    /// Surveille un accès mémoire (honeypot silencieux).
    pub fn observe_memory_access(&self, memory: &Memory, action: &str) {
        if is_honeypot_memory(memory) {
            self.behavioral.record_honeypot_access();
            let _ = self.audit.record(
                "honeypot_access",
                &format!("action={action} memory_id={}", memory.id),
            );
            tracing::warn!(
                memory_id = %memory.id,
                action,
                "accès honeypot détecté"
            );
        }
    }

    /// Enregistre un rejet de validation adversariale.
    pub fn record_validation_reject(&self, reason: &str) {
        let _ = self.audit.record("validation_reject", reason);
        self.behavioral.record_assimilation();
    }

    /// Enregistre un événement de sécurité générique.
    pub fn record_security_event(&self, event_type: &str, details: &str) {
        let _ = self.audit.record(event_type, details);
    }

    /// Statut d'intégrité courant.
    #[must_use]
    pub fn integrity_status(&self) -> &IntegrityStatus {
        &self.integrity
    }

    fn audit_degraded(&self, reason: &str) -> Result<(), AuditError> {
        self.audit.record("integrity_degraded", reason)
    }
}

/// Erreur combinée des garde-fous.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum SecurityGateError {
    /// Mode dégradé (intégrité).
    #[error(transparent)]
    Degraded(#[from] DegradedModeError),
    /// Limite comportementale.
    #[error(transparent)]
    Behavioral(#[from] BehavioralError),
}

/// Erreur d'initialisation sécurité.
#[derive(Debug, thiserror::Error)]
pub enum SecurityBootstrapError {
    /// Intégrité.
    #[error(transparent)]
    Integrity(#[from] IntegrityError),
    /// Audit.
    #[error(transparent)]
    Audit(#[from] AuditError),
}

/// Fabrique un [`Arc<SecurityContext>`] pour l'injection.
///
/// # Errors
///
/// Propage [`SecurityBootstrapError`] si l'intégrité ou l'audit ne peut pas s'initialiser.
pub fn build_security_context(
    config: &OrchestratorConfig,
) -> Result<Arc<SecurityContext>, SecurityBootstrapError> {
    Ok(Arc::new(SecurityContext::bootstrap(config)?))
}

/// Fabrique un contexte de test.
#[must_use]
pub fn build_test_security_context(config: &OrchestratorConfig) -> Arc<SecurityContext> {
    Arc::new(SecurityContext::for_tests(config))
}
