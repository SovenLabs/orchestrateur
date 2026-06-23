//! Surveillance des sessions — génération automatique de brouillons insight.

mod session_watcher;

use std::sync::{Arc, OnceLock};

pub use session_watcher::{DraftReadyCallback, SessionWatcherHandle, WatcherRuntimeState};

static GLOBAL_WATCHER: OnceLock<Arc<SessionWatcherHandle>> = OnceLock::new();

/// Installe le handle global (daemon / CLI `watch`).
pub fn install_global(handle: Arc<SessionWatcherHandle>) {
    let _ = GLOBAL_WATCHER.set(handle);
}

/// Handle global optionnel (commandes bridge `WatcherStatus`).
#[must_use]
pub fn global_handle() -> Option<Arc<SessionWatcherHandle>> {
    GLOBAL_WATCHER.get().cloned()
}

/// Démarre le watcher si activé dans la config et l'enregistre globalement.
pub fn spawn_if_enabled(
    facade: Arc<crate::facade::OrchestratorFacade>,
    on_draft_ready: Option<DraftReadyCallback>,
) {
    let config = &facade.deps().config;
    if !config.watcher.enabled {
        return;
    }
    let handle = Arc::new(SessionWatcherHandle::new(
        Arc::clone(&facade),
        on_draft_ready,
        config,
    ));
    install_global(Arc::clone(&handle));
    handle.spawn();
}