//! Couche 4 — journal d'audit append-only avec chaînage BLAKE3.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};

use blake3::Hasher;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::AuditConfig;

/// Hash initial de la chaîne d'audit.
pub const AUDIT_GENESIS: &str = "GENESIS";

/// Entrée d'audit tamper-evident.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Horodatage UTC ISO-8601.
    pub timestamp: String,
    /// Type d'événement (`assimilate`, `search`, `integrity`, …).
    pub event_type: String,
    /// Détail lisible (sans secrets).
    pub details: String,
    /// Hash de l'entrée précédente.
    pub previous_hash: String,
    /// Hash BLAKE3 de cette entrée.
    pub hash: String,
}

/// Erreur d'écriture du journal d'audit.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AuditError {
    /// Erreur disque.
    #[error("audit IO {path:?}: {message}")]
    Io {
        /// Chemin du journal.
        path: PathBuf,
        /// Détail.
        message: String,
    },
    /// Sérialisation JSON.
    #[error("audit sérialisation: {0}")]
    Serialize(String),
}

/// Journal d'audit chaîné — append-only.
#[derive(Debug)]
pub struct AuditLog {
    path: PathBuf,
    enabled: bool,
    state: Mutex<AuditChainState>,
}

#[derive(Debug)]
struct AuditChainState {
    last_hash: String,
}

impl AuditLog {
    /// Ouvre ou crée le journal d'audit.
    ///
    /// # Errors
    ///
    /// Retourne [`AuditError`] si le fichier existant est illisible.
    pub fn open(config: &AuditConfig, workspace_root: &Path) -> Result<Self, AuditError> {
        let path = workspace_root.join(&config.path);
        if config.enabled {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| AuditError::Io {
                    path: parent.to_path_buf(),
                    message: e.to_string(),
                })?;
            }
        }
        let last_hash = if config.enabled {
            load_last_hash(&path)?
        } else {
            AUDIT_GENESIS.to_string()
        };
        Ok(Self {
            path,
            enabled: config.enabled,
            state: Mutex::new(AuditChainState { last_hash }),
        })
    }

    /// Journal inactif (tests) — n'écrit rien.
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            path: PathBuf::from("/dev/null"),
            enabled: false,
            state: Mutex::new(AuditChainState {
                last_hash: AUDIT_GENESIS.to_string(),
            }),
        }
    }

    /// Ajoute un événement au journal.
    ///
    /// # Errors
    ///
    /// Retourne [`AuditError`] en cas d'échec d'écriture disque.
    pub fn record(&self, event_type: &str, details: &str) -> Result<(), AuditError> {
        if !self.enabled {
            return Ok(());
        }
        let timestamp = Utc::now().to_rfc3339();
        let mut state = lock_or_recover(&self.state);
        let payload = format!("{timestamp}|{event_type}|{details}|{}", state.last_hash);
        let hash = hash_payload(payload.as_bytes());
        let event = AuditEvent {
            timestamp,
            event_type: event_type.to_string(),
            details: details.to_string(),
            previous_hash: state.last_hash.clone(),
            hash: hash.clone(),
        };
        let line =
            serde_json::to_string(&event).map_err(|e| AuditError::Serialize(e.to_string()))?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| AuditError::Io {
                path: self.path.clone(),
                message: e.to_string(),
            })?;
        writeln!(file, "{line}").map_err(|e| AuditError::Io {
            path: self.path.clone(),
            message: e.to_string(),
        })?;
        state.last_hash = hash;
        tracing::debug!(event_type, "audit_recorded");
        Ok(())
    }

    /// Dernier hash connu (pour tests / vérification).
    #[must_use]
    pub fn last_hash(&self) -> String {
        lock_or_recover(&self.state).last_hash.clone()
    }

    /// Lit les entrées les plus récentes du journal (ordre chronologique).
    ///
    /// # Errors
    ///
    /// Retourne [`AuditError`] si le fichier est illisible ou mal formé.
    pub fn read_recent(&self, limit: usize) -> Result<Vec<AuditEvent>, AuditError> {
        if !self.enabled {
            return Ok(Vec::new());
        }
        let events = read_all_events(&self.path)?;
        let start = events.len().saturating_sub(limit);
        Ok(events[start..].to_vec())
    }

    /// Vérifie l'intégrité de la chaîne BLAKE3 sur le fichier complet.
    #[must_use]
    pub fn verify_chain(&self) -> bool {
        if !self.enabled {
            return true;
        }
        let Ok(events) = read_all_events(&self.path) else {
            return false;
        };
        verify_event_chain(&events)
    }
}

fn lock_or_recover<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

fn hash_payload(bytes: &[u8]) -> String {
    let mut hasher = Hasher::new();
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

fn read_all_events(path: &Path) -> Result<Vec<AuditEvent>, AuditError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(path).map_err(|e| AuditError::Io {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;
    let mut events = Vec::new();
    for line in raw.lines().filter(|l| !l.trim().is_empty()) {
        let event: AuditEvent =
            serde_json::from_str(line).map_err(|e| AuditError::Serialize(e.to_string()))?;
        events.push(event);
    }
    Ok(events)
}

fn verify_event_chain(events: &[AuditEvent]) -> bool {
    let mut previous_hash = AUDIT_GENESIS.to_string();
    for event in events {
        if event.previous_hash != previous_hash {
            return false;
        }
        let payload = format!(
            "{}|{}|{}|{}",
            event.timestamp, event.event_type, event.details, event.previous_hash
        );
        let expected = hash_payload(payload.as_bytes());
        if event.hash != expected {
            return false;
        }
        previous_hash.clone_from(&event.hash);
    }
    true
}

fn load_last_hash(path: &Path) -> Result<String, AuditError> {
    if !path.exists() {
        return Ok(AUDIT_GENESIS.to_string());
    }
    let raw = fs::read_to_string(path).map_err(|e| AuditError::Io {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;
    let Some(last_line) = raw.lines().rfind(|l| !l.trim().is_empty()) else {
        return Ok(AUDIT_GENESIS.to_string());
    };
    let event: AuditEvent =
        serde_json::from_str(last_line).map_err(|e| AuditError::Serialize(e.to_string()))?;
    Ok(event.hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_recent_returns_tail() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cfg = AuditConfig {
            enabled: true,
            path: PathBuf::from("logs/audit.jsonl"),
        };
        let log = AuditLog::open(&cfg, dir.path()).expect("open");
        log.record("a", "1").expect("r1");
        log.record("b", "2").expect("r2");
        log.record("c", "3").expect("r3");
        let recent = log.read_recent(2).expect("read");
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].event_type, "b");
        assert_eq!(recent[1].event_type, "c");
    }

    #[test]
    fn verify_chain_detects_tampering() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cfg = AuditConfig {
            enabled: true,
            path: PathBuf::from("logs/audit.jsonl"),
        };
        let log = AuditLog::open(&cfg, dir.path()).expect("open");
        log.record("test", "ok").expect("record");
        assert!(log.verify_chain());
        let path = dir.path().join("logs/audit.jsonl");
        let mut content = fs::read_to_string(&path).expect("read");
        content = content.replace("ok", "tampered");
        fs::write(&path, content).expect("write");
        assert!(!log.verify_chain());
    }

    #[test]
    fn chains_audit_entries() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cfg = AuditConfig {
            enabled: true,
            path: PathBuf::from("logs/audit.jsonl"),
        };
        let log = AuditLog::open(&cfg, dir.path()).expect("open");
        log.record("assimilate", "title=Test").expect("r1");
        log.record("search", "query=foo").expect("r2");
        let h1 = log.last_hash();
        assert_ne!(h1, AUDIT_GENESIS);
        let log2 = AuditLog::open(&cfg, dir.path()).expect("reopen");
        assert_eq!(log2.last_hash(), h1);
    }
}
