use std::collections::HashMap;
use std::sync::RwLock;

use async_trait::async_trait;
use cortex::{
    ConversationTurn, CortexError, Session, SessionKey, SessionRepository, SessionSummary,
    SessionTurnHit, TurnRole,
};

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

    async fn list_recent_sessions(&self, limit: usize) -> Result<Vec<SessionSummary>, CortexError> {
        let guard = self
            .inner
            .read()
            .map_err(|e| CortexError::GraphError(e.to_string()))?;
        let mut sessions: Vec<_> = guard.values().cloned().collect();
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(sessions
            .into_iter()
            .take(limit.max(1))
            .map(|s| {
                let preview = s
                    .turns
                    .iter()
                    .rev()
                    .find(|t| matches!(t.role, TurnRole::User | TurnRole::Assistant))
                    .map(|t| t.content.chars().take(200).collect::<String>())
                    .unwrap_or_default();
                SessionSummary {
                    key: s.key,
                    turn_count: s.turns.len(),
                    updated_at: s.updated_at,
                    preview,
                }
            })
            .collect())
    }

    async fn search_turns(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SessionTurnHit>, CortexError> {
        let q = query.to_lowercase();
        let guard = self
            .inner
            .read()
            .map_err(|e| CortexError::GraphError(e.to_string()))?;
        let mut hits = Vec::new();
        for session in guard.values() {
            for (idx, turn) in session.turns.iter().enumerate() {
                if turn.content.to_lowercase().contains(&q) {
                    hits.push(SessionTurnHit {
                        key: session.key.clone(),
                        turn_index: idx,
                        role: turn.role,
                        snippet: turn.content.chars().take(300).collect(),
                    });
                }
            }
        }
        hits.sort_by(|a, b| b.turn_index.cmp(&a.turn_index));
        hits.truncate(limit.max(1));
        Ok(hits)
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