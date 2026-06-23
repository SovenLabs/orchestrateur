use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use tracing::debug;

/// Hook exécuté par une skill Cortex sur le pipeline mémoire.
#[async_trait]
pub trait CortexExtension: Send + Sync {
    /// Identifiant stable de l'extension.
    fn name(&self) -> &str;

    /// Transforme une requête de recherche avant exécution.
    fn transform_search_query(&self, query: &str) -> Option<String> {
        let _ = query;
        None
    }

    /// Notifie après assimilation réussie.
    fn on_memory_assimilated(&self, memory_id: &str) {
        let _ = memory_id;
    }

    /// Notifie après publication d'un brouillon.
    fn on_draft_published(&self, draft_id: &str) {
        let _ = draft_id;
    }
}

/// Registre central des points d'extension Cortex (Phase 6).
#[derive(Clone, Default)]
pub struct CortexExtensionRegistry {
    extensions: Arc<RwLock<Vec<Arc<dyn CortexExtension>>>>,
}

impl CortexExtensionRegistry {
    /// Crée un registre vide.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enregistre une extension (écrase par nom si déjà présent).
    pub fn register(&self, extension: Arc<dyn CortexExtension>) {
        let name = extension.name().to_string();
        let mut guard = self.extensions.write().expect("cortex extensions lock");
        guard.retain(|e| e.name() != name);
        debug!(extension = %name, "extension cortex enregistrée");
        guard.push(extension);
    }

    /// Liste les noms d'extensions actives.
    #[must_use]
    pub fn list(&self) -> Vec<String> {
        let guard = self.extensions.read().expect("cortex extensions lock");
        guard.iter().map(|e| e.name().to_string()).collect()
    }

    /// Applique les transformations de recherche en chaîne.
    #[must_use]
    pub fn apply_search_transforms(&self, mut query: String) -> String {
        let guard = self.extensions.read().expect("cortex extensions lock");
        for ext in guard.iter() {
            if let Some(next) = ext.transform_search_query(&query) {
                query = next;
            }
        }
        query
    }

    /// Propage un événement d'assimilation.
    pub fn notify_assimilated(&self, memory_id: &str) {
        let guard = self.extensions.read().expect("cortex extensions lock");
        for ext in guard.iter() {
            ext.on_memory_assimilated(memory_id);
        }
    }

    /// Propage un événement de brouillon publié.
    pub fn notify_draft_published(&self, draft_id: &str) {
        let guard = self.extensions.read().expect("cortex extensions lock");
        for ext in guard.iter() {
            ext.on_draft_published(draft_id);
        }
    }
}