use async_trait::async_trait;

use crate::domain::{
    ConversationTurn, CortexError, Session, SessionKey, SessionSummary, SessionTurnHit,
};

/// Port de persistance des sessions de conversation agent.
///
/// Implémenté par `infrastructure` (`SqliteSessionStore`) ou mocks de test.
#[async_trait]
pub trait SessionRepository: Send + Sync {
    /// Récupère une session existante ou en crée une nouvelle vide.
    async fn get_or_create(&self, key: &SessionKey) -> Result<Session, CortexError>;

    /// Ajoute un tour à la session (crée la session si absente).
    async fn append_turn(
        &self,
        key: &SessionKey,
        turn: ConversationTurn,
    ) -> Result<Session, CortexError>;

    /// Liste les tours d'une session (session vide si absente).
    async fn list_turns(&self, key: &SessionKey) -> Result<Vec<ConversationTurn>, CortexError>;

    /// Supprime une session et son historique.
    async fn delete(&self, key: &SessionKey) -> Result<(), CortexError>;

    /// Liste les sessions les plus récentes (mode browse `session_search`).
    async fn list_recent_sessions(&self, limit: usize) -> Result<Vec<SessionSummary>, CortexError>;

    /// Recherche textuelle dans le contenu des tours (mode discovery).
    async fn search_turns(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SessionTurnHit>, CortexError>;
}