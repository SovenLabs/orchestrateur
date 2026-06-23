use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use thiserror::Error;
use tracing::{info, warn};

use crate::config::OrchestratorConfig;
use crate::skills::registry::SkillRegistry;

/// Erreurs du hot-reload des skills.
#[derive(Debug, Error)]
pub enum HotReloadError {
    /// Erreur watcher filesystem.
    #[error("watcher: {0}")]
    Watcher(String),
}

/// Surveille le hub skills et recharge les plugins (Phase 6).
pub struct SkillHotReload {
    _watcher: RecommendedWatcher,
}

impl SkillHotReload {
    /// Démarre le watcher si `skills_hub.hot_reload` est activé.
    ///
    /// # Errors
    ///
    /// Propage [`HotReloadError`] si le watcher ne peut pas démarrer.
    pub fn spawn_if_enabled(
        registry: Arc<Mutex<SkillRegistry>>,
        config: OrchestratorConfig,
    ) -> Result<Option<Self>, HotReloadError> {
        if !config.skills_hub.hot_reload {
            return Ok(None);
        }
        let hub_dir = config.skills_hub_dir();
        if !hub_dir.is_dir() {
            return Ok(None);
        }

        let registry_watch = Arc::clone(&registry);
        let cfg = config.clone();
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    if matches!(
                        event.kind,
                        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                    ) {
                        let registry = Arc::clone(&registry_watch);
                        let cfg = cfg.clone();
                        std::thread::spawn(move || {
                            std::thread::sleep(Duration::from_millis(250));
                            let Ok(mut guard) = registry.lock() else {
                                return;
                            };
                            match guard.reload_hub(&cfg) {
                                Ok(count) => info!(count, "skills hub rechargées (hot reload)"),
                                Err(err) => warn!(%err, "échec hot reload skills"),
                            }
                        });
                    }
                }
            },
            notify::Config::default(),
        )
        .map_err(|e| HotReloadError::Watcher(e.to_string()))?;

        watcher
            .watch(&hub_dir, RecursiveMode::Recursive)
            .map_err(|e| HotReloadError::Watcher(e.to_string()))?;

        info!(dir = %hub_dir.display(), "hot reload skills actif");
        Ok(Some(Self { _watcher: watcher }))
    }
}

/// Chemin surveillé (tests / diagnostic).
#[must_use]
#[allow(dead_code)]
pub fn watched_hub_path(config: &OrchestratorConfig) -> PathBuf {
    config.skills_hub_dir()
}