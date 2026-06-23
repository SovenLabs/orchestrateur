//! Port journal d'audit B212 (JSONL).

use async_trait::async_trait;

use crate::error::B212Error;
use crate::types::JournalEntry;

/// Persistance append-only du journal B212.
#[async_trait]
pub trait B212Journal: Send + Sync {
    /// Ajoute une entrée au journal.
    async fn append(&self, entry: &JournalEntry) -> Result<(), B212Error>;
}