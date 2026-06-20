//! Couche 3 — détection comportementale et rate limiting adaptatif.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use thiserror::Error;

use crate::config::BehavioralConfig;

/// Action soumise au garde comportemental.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardAction {
    /// Assimilation d'un brouillon ou texte.
    Assimilation,
    /// Recherche sémantique.
    Search,
}

/// Erreur de garde comportemental.
#[derive(Debug, Error, PartialEq)]
pub enum BehavioralError {
    /// Limite par fenêtre glissante dépassée.
    #[error("limite comportementale atteinte pour {action:?} ({count}/{max} par minute)")]
    RateLimited {
        /// Action concernée.
        action: GuardAction,
        /// Compteur observé.
        count: u32,
        /// Limite configurée.
        max: u32,
    },
    /// Requêtes de recherche identiques en rafale.
    #[error("requêtes de recherche trop répétitives ({count}/{max})")]
    RepetitiveSearch {
        /// Occurrences de la même requête.
        count: u32,
        /// Limite configurée.
        max: u32,
    },
    /// Score d'anomalie trop élevé (honeypot, comportement suspect).
    #[error("score d'anomalie élevé ({score}) — action bloquée")]
    AnomalyBlocked {
        /// Score courant.
        score: f32,
    },
}

/// État interne du garde (fenêtre glissante + score d'anomalie).
#[derive(Debug)]
pub struct BehavioralGuard {
    config: BehavioralConfig,
    assimilation_events: Mutex<Vec<Instant>>,
    search_events: Mutex<Vec<Instant>>,
    search_queries: Mutex<HashMap<String, u32>>,
    anomaly_score: Mutex<f32>,
}

impl BehavioralGuard {
    /// Construit un garde à partir de la configuration.
    #[must_use]
    pub fn new(config: BehavioralConfig) -> Self {
        Self {
            config,
            assimilation_events: Mutex::new(Vec::new()),
            search_events: Mutex::new(Vec::new()),
            search_queries: Mutex::new(HashMap::new()),
            anomaly_score: Mutex::new(0.0),
        }
    }

    /// Vérifie qu'une assimilation est autorisée.
    ///
    /// # Errors
    ///
    /// Retourne [`BehavioralError`] si la limite ou le score d'anomalie est dépassé.
    pub fn check_assimilation(&self) -> Result<(), BehavioralError> {
        if !self.config.enabled {
            return Ok(());
        }
        self.check_anomaly()?;
        let count = self.count_recent(&self.assimilation_events);
        if count >= self.config.max_assimilations_per_minute {
            return Err(BehavioralError::RateLimited {
                action: GuardAction::Assimilation,
                count,
                max: self.config.max_assimilations_per_minute,
            });
        }
        Ok(())
    }

    /// Enregistre une assimilation réussie ou tentée.
    pub fn record_assimilation(&self) {
        if !self.config.enabled {
            return;
        }
        self.push_event(&self.assimilation_events);
        self.bump_anomaly(0.5);
    }

    /// Vérifie qu'une recherche est autorisée.
    ///
    /// # Errors
    ///
    /// Retourne [`BehavioralError`] si la limite, la répétition ou le score est dépassé.
    pub fn check_search(&self, query: &str) -> Result<(), BehavioralError> {
        if !self.config.enabled {
            return Ok(());
        }
        self.check_anomaly()?;
        let count = self.count_recent(&self.search_events);
        if count >= self.config.max_searches_per_minute {
            return Err(BehavioralError::RateLimited {
                action: GuardAction::Search,
                count,
                max: self.config.max_searches_per_minute,
            });
        }
        let key = normalize_query(query);
        let repetitive = {
            let map = lock_or_recover(&self.search_queries);
            map.get(&key).copied().unwrap_or(0) + 1
        };
        if repetitive > self.config.max_repetitive_searches {
            return Err(BehavioralError::RepetitiveSearch {
                count: repetitive,
                max: self.config.max_repetitive_searches,
            });
        }
        Ok(())
    }

    /// Enregistre une recherche.
    pub fn record_search(&self, query: &str) {
        if !self.config.enabled {
            return;
        }
        self.push_event(&self.search_events);
        let key = normalize_query(query);
        let mut map = lock_or_recover(&self.search_queries);
        self.prune_search_queries(&mut map);
        *map.entry(key).or_insert(0) += 1;
        self.bump_anomaly(0.2);
    }

    /// Signale un accès à un honeypot — augmente fortement le score d'anomalie.
    pub fn record_honeypot_access(&self) {
        self.bump_anomaly(50.0);
    }

    /// Score d'anomalie courant (lecture seule).
    #[must_use]
    pub fn anomaly_score(&self) -> f32 {
        *lock_or_recover(&self.anomaly_score)
    }

    fn check_anomaly(&self) -> Result<(), BehavioralError> {
        let score = self.anomaly_score();
        if score >= self.config.anomaly_block_threshold {
            return Err(BehavioralError::AnomalyBlocked { score });
        }
        Ok(())
    }

    fn bump_anomaly(&self, delta: f32) {
        let mut score = lock_or_recover(&self.anomaly_score);
        *score = (*score + delta).min(100.0);
    }

    fn count_recent(&self, events: &Mutex<Vec<Instant>>) -> u32 {
        let mut vec = lock_or_recover(events);
        self.prune_instant_vec(&mut vec);
        u32::try_from(vec.len()).unwrap_or(u32::MAX)
    }

    fn push_event(&self, events: &Mutex<Vec<Instant>>) {
        let mut vec = lock_or_recover(events);
        self.prune_instant_vec(&mut vec);
        vec.push(Instant::now());
    }

    fn prune_instant_vec(&self, vec: &mut Vec<Instant>) {
        let window = Duration::from_secs(self.config.window_secs);
        let cutoff = Instant::now()
            .checked_sub(window)
            .unwrap_or_else(Instant::now);
        vec.retain(|t| *t >= cutoff);
    }

    fn prune_search_queries(&self, map: &mut HashMap<String, u32>) {
        let _window = Duration::from_secs(self.config.window_secs);
        if map.len() > 512 {
            map.clear();
        }
    }
}

fn lock_or_recover<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

fn normalize_query(query: &str) -> String {
    query
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tight_config() -> BehavioralConfig {
        BehavioralConfig {
            enabled: true,
            max_assimilations_per_minute: 3,
            max_searches_per_minute: 5,
            max_repetitive_searches: 2,
            window_secs: 60,
            anomaly_block_threshold: 80.0,
        }
    }

    #[test]
    #[ignore = "sécurité: garde comportementale burst assimilation"]
    fn blocks_assimilation_burst() {
        let guard = BehavioralGuard::new(tight_config());
        for _ in 0..3 {
            guard.check_assimilation().expect("sous limite");
            guard.record_assimilation();
        }
        let err = guard.check_assimilation().unwrap_err();
        assert!(matches!(
            err,
            BehavioralError::RateLimited {
                action: GuardAction::Assimilation,
                ..
            }
        ));
    }

    #[test]
    #[ignore = "sécurité: garde comportementale recherche répétitive"]
    fn blocks_repetitive_search() {
        let guard = BehavioralGuard::new(tight_config());
        guard.check_search("secret token").expect("première");
        guard.record_search("secret token");
        guard.check_search("secret token").expect("deuxième");
        guard.record_search("secret token");
        let err = guard.check_search("secret token").unwrap_err();
        assert!(matches!(err, BehavioralError::RepetitiveSearch { .. }));
    }

    #[test]
    #[ignore = "sécurité: accès honeypot augmente score anomalie"]
    fn honeypot_access_raises_anomaly() {
        let guard = BehavioralGuard::new(tight_config());
        guard.record_honeypot_access();
        assert!(guard.anomaly_score() >= 50.0);
    }
}
