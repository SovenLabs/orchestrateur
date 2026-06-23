//! Surveillance des fichiers session Markdown — génère des brouillons à la fin de session.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use chrono::Utc;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use crate::bridge::WatcherStatus;
use crate::config::{OrchestratorConfig, WatcherConfig};
use crate::error::OrchestratorError;
use crate::facade::OrchestratorFacade;
use crate::draft::StoredDraft;
use crate::use_cases::GenerateInsightDraft;

/// Callback optionnel lorsqu'un brouillon session est prêt (broadcast daemon).
pub type DraftReadyCallback = Arc<dyn Fn(&StoredDraft) + Send + Sync>;

/// État runtime partagé du watcher (observabilité bridge).
#[derive(Debug, Clone, Default)]
pub struct WatcherRuntimeState {
    /// Watcher actif (tâche tokio en cours).
    pub running: bool,
    /// Sessions traitées depuis le démarrage.
    pub sessions_processed: usize,
    /// Brouillons créés depuis le démarrage.
    pub drafts_created: usize,
    /// Dernière activité UTC.
    pub last_activity_at: Option<chrono::DateTime<Utc>>,
    /// Dernière erreur lisible.
    pub last_error: Option<String>,
}

/// Handle partagé pour commandes bridge et daemon.
#[derive(Clone)]
pub struct SessionWatcherHandle {
    config: WatcherConfig,
    workspace: PathBuf,
    watch_dirs: Vec<PathBuf>,
    facade: Arc<OrchestratorFacade>,
    on_draft_ready: Option<DraftReadyCallback>,
    state: Arc<Mutex<WatcherRuntimeState>>,
    shutdown_tx: Arc<Mutex<Option<mpsc::Sender<()>>>>,
}

impl SessionWatcherHandle {
    /// Crée un handle sans démarrer la surveillance.
    #[must_use]
    pub fn new(
        facade: Arc<OrchestratorFacade>,
        on_draft_ready: Option<DraftReadyCallback>,
        config: &OrchestratorConfig,
    ) -> Self {
        let watch_dirs = resolve_watch_dirs(config);
        Self {
            config: config.watcher.clone(),
            workspace: config.workspace_root.clone(),
            watch_dirs,
            facade,
            on_draft_ready,
            state: Arc::new(Mutex::new(WatcherRuntimeState::default())),
            shutdown_tx: Arc::new(Mutex::new(None)),
        }
    }

    /// Démarre la tâche tokio de surveillance (idempotent si déjà actif).
    pub fn spawn(self: Arc<Self>) {
        let mut guard = self.state.lock().expect("watcher state lock");
        if guard.running {
            return;
        }
        guard.running = true;
        drop(guard);

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        if let Ok(mut slot) = self.shutdown_tx.lock() {
            *slot = Some(shutdown_tx);
        }

        let handle = Arc::clone(&self);
        tokio::spawn(async move {
            info!(
                dirs = ?handle.watch_dirs.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
                "session watcher démarré"
            );
            if let Err(err) = handle.run_loop(&mut shutdown_rx).await {
                error!(%err, "session watcher interrompu");
                if let Ok(mut guard) = handle.state.lock() {
                    guard.last_error = Some(err.to_string());
                    guard.running = false;
                }
            }
        });
    }

    /// Arrête la surveillance en cours.
    pub fn stop(&self) {
        if let Ok(slot) = self.shutdown_tx.lock() {
            if let Some(tx) = slot.as_ref() {
                let _ = tx.try_send(());
            }
        }
        if let Ok(mut guard) = self.state.lock() {
            guard.running = false;
        }
    }

    /// Statut sérialisable pour le bridge.
    pub async fn status(&self) -> WatcherStatus {
        use crate::draft::DraftStatus;

        let snapshot = {
            let state = self.state.lock().expect("watcher state lock");
            (
                state.running,
                state.sessions_processed,
                state.drafts_created,
                state.last_activity_at,
                state.last_error.clone(),
            )
        };

        let drafts_pending = self
            .facade
            .deps()
            .draft_repo
            .list(Some(DraftStatus::Pending))
            .await
            .map(|d| d.len())
            .unwrap_or(0);

        WatcherStatus {
            enabled: self.config.enabled,
            running: snapshot.0,
            watch_dirs: self
                .watch_dirs
                .iter()
                .map(|p| p.display().to_string())
                .collect(),
            sessions_processed: snapshot.1,
            drafts_created: snapshot.2,
            drafts_pending,
            last_activity_at: snapshot.3,
            last_error: snapshot.4,
        }
    }

    async fn run_loop(
        &self,
        shutdown_rx: &mut mpsc::Receiver<()>,
    ) -> Result<(), OrchestratorError> {
        for dir in &self.watch_dirs {
            tokio::fs::create_dir_all(dir)
                .await
                .map_err(|e| OrchestratorError::Internal(e.to_string()))?;
        }

        let (notify_tx, mut notify_rx) = mpsc::unbounded_channel();
        let watch_dirs = self.watch_dirs.clone();
        let debounce = Duration::from_secs(self.config.debounce_secs.max(2));

        let watcher = {
            let tx = notify_tx.clone();
            let mut watcher = RecommendedWatcher::new(
                move |res| {
                    if let Ok(event) = res {
                        let _ = tx.send(event);
                    }
                },
                notify::Config::default(),
            )
            .map_err(|e| OrchestratorError::Internal(e.to_string()))?;
            for dir in &watch_dirs {
                watcher
                    .watch(dir, RecursiveMode::Recursive)
                    .map_err(|e| OrchestratorError::Internal(e.to_string()))?;
            }
            watcher
        };

        let _watcher_guard = watcher;
        let mut pending: HashMap<PathBuf, Instant> = HashMap::new();
        let mut processed_hashes: HashMap<PathBuf, String> = HashMap::new();
        let mut ticker = tokio::time::interval(Duration::from_secs(1));

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("session watcher arrêt demandé");
                    if let Ok(mut guard) = self.state.lock() {
                        guard.running = false;
                    }
                    break;
                }
                maybe_event = notify_rx.recv() => {
                    if let Some(event) = maybe_event {
                        if matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_) | EventKind::Any) {
                            for path in event.paths {
                                if is_session_file(&path) {
                                    pending.insert(path, Instant::now());
                                }
                            }
                        }
                    }
                }
                _ = ticker.tick() => {
                    let now = Instant::now();
                    let ready: Vec<PathBuf> = pending
                        .iter()
                        .filter(|(_, started)| now.duration_since(**started) >= debounce)
                        .map(|(p, _)| p.clone())
                        .collect();
                    for path in ready {
                        pending.remove(&path);
                        if let Err(err) = self
                            .process_session_file(&path, &mut processed_hashes)
                            .await
                        {
                            debug!(path = %path.display(), %err, "session ignorée");
                            if let Ok(mut guard) = self.state.lock() {
                                guard.last_error = Some(err.to_string());
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn process_session_file(
        &self,
        path: &Path,
        processed_hashes: &mut HashMap<PathBuf, String>,
    ) -> Result<(), OrchestratorError> {
        if !path.is_file() {
            return Ok(());
        }
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| OrchestratorError::Internal(e.to_string()))?;
        if content.trim().len() < self.config.min_content_chars {
            return Err(OrchestratorError::InsightSkipped {
                reason: "session trop courte".into(),
            });
        }

        let hash = blake3::hash(content.as_bytes()).to_string();
        if processed_hashes.get(path) == Some(&hash) {
            return Ok(());
        }

        let rel = path
            .strip_prefix(&self.workspace)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| path.display().to_string());

        let draft = GenerateInsightDraft::new(self.facade.deps().clone())
            .execute(&content, &[], Some(&rel), None)
            .await?;

        let stored = self
            .facade
            .deps()
            .draft_repo
            .create_pending(draft, Some(rel))
            .await?;
        processed_hashes.insert(path.to_path_buf(), hash);

        if let Ok(mut guard) = self.state.lock() {
            guard.sessions_processed += 1;
            guard.drafts_created += 1;
            guard.last_activity_at = Some(Utc::now());
            guard.last_error = None;
        }

        if let Some(cb) = &self.on_draft_ready {
            cb(&stored);
        }
        info!(draft_id = %stored.id, title = %stored.draft.title, "brouillon session créé");
        Ok(())
    }
}

fn is_session_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
}

fn resolve_watch_dirs(config: &OrchestratorConfig) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(extra) = std::env::var("ORCHESTRATEUR_SESSIONS_DIR") {
        if !extra.is_empty() {
            dirs.push(PathBuf::from(extra));
        }
    }
    for entry in &config.watcher.watch_dirs {
        let path = if Path::new(entry).is_absolute() {
            PathBuf::from(entry)
        } else {
            config.workspace_root.join(entry)
        };
        dirs.push(path);
    }
    if dirs.is_empty() {
        dirs.push(config.sessions_watch_dir());
    }
    dirs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_markdown_sessions() {
        assert!(is_session_file(Path::new("foo/bar/session.md")));
        assert!(!is_session_file(Path::new("foo/bar/session.txt")));
    }
}