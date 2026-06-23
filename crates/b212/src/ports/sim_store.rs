use async_trait::async_trait;

use crate::error::B212Error;
use crate::types::SimFill;

/// Persistance des fills paper B212.
#[async_trait]
pub trait SimTradeRepository: Send + Sync {
    /// Enregistre un fill simulé.
    async fn save(&self, fill: &SimFill) -> Result<(), B212Error>;

    /// Charge un fill par identifiant.
    async fn get(&self, id: &str) -> Result<SimFill, B212Error>;

    /// Liste les fills d'une proposition.
    async fn list_for_proposal(&self, proposal_id: &str) -> Result<Vec<SimFill>, B212Error>;
}