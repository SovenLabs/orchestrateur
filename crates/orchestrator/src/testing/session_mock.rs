use std::collections::HashMap;
use std::sync::RwLock;

use async_trait::async_trait;
use cortex::{ConversationTurn, CortexError, Session, SessionKey, SessionRepository};

/// Sessions agent en mémoire pour les tests.
pub struct InMemorySessionRepository {
    inner: RwLock<HashMap<String, Session>>,
}

impl InMemorySessionRepository {
    /// Crée un dépôt vide.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemorySessionRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SessionRepository for InMemorySessionRepository {
    async fn get_or_create(&self, key: &SessionKey) -> Result<Session, CortexError> {
        let mut guard = self
            .inner
            .write()
            .map_err(|e| CortexError::GraphError(e.to_string()))?;
        Ok(guard
            .entry(key.as_str().to_string())
            .or_insert_with(|| Session::new(key.clone()))
            .clone())
    }

    async fn append_turn(
        &self,
        key: &SessionKey,
        turn: ConversationTurn,
    ) -> Result<Session, CortexError> {
        let mut guard = self
            .inner
            .write()
            .map_err(|e| CortexError::GraphError(e.to_string()))?;
        let session = guard
            .entry(key.as_str().to_string())
            .or_insert_with(|| Session::new(key.clone()));
        session.push_turn(turn);
        Ok(session.clone())
    }

    async fn list_turns(&self, key: &SessionKey) -> Result<Vec<ConversationTurn>, CortexError> {
        let guard = self
            .inner
            .read()
            .map_err(|e| CortexError::GraphError(e.to_string()))?;
        Ok(guard
            .get(key.as_str())
            .map(|s| s.turns.clone())
            .unwrap_or_default())
    }

    async fn delete(&self, key: &SessionKey) -> Result<(), CortexError> {
        let mut guard = self
            .inner
            .write()
            .map_err(|e| CortexError::GraphError(e.to_string()))?;
        guard.remove(key.as_str());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cortex::TurnRole;

    #[tokio::test]
    async fn session_roundtrip() {
        let repo = InMemorySessionRepository::new();
        let key = SessionKey::default_chat();
        repo.append_turn(
            &key,
            ConversationTurn::new(TurnRole::User, "hello"),
        )
        .await
        .unwrap();
        let turns = repo.list_turns(&key).await.unwrap();
        assert_eq!(turns.len(), 1);
    }
}