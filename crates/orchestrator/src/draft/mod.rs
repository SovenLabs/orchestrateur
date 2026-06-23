//! Brouillons persistés — file de revue avant assimilation Cortex.

mod record;
mod repository;

pub use record::{DraftStatus, StoredDraft};
pub use repository::{DraftError, DraftRepository};