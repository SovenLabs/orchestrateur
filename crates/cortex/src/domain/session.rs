use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::CortexError;

/// Clé de session agent (ex. `default`, `hud-chat`, identifiant canal).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionKey(String);

impl SessionKey {
    /// Crée une clé depuis une chaîne non vide (max 128 caractères).
    ///
    /// # Errors
    ///
    /// Retourne [`CortexError::InvalidSessionKey`] si vide ou trop longue.
    pub fn new(key: impl Into<String>) -> Result<Self, CortexError> {
        let key = key.into();
        if key.trim().is_empty() {
            return Err(CortexError::InvalidSessionKey(
                "clé de session vide".into(),
            ));
        }
        if key.len() > 128 {
            return Err(CortexError::InvalidSessionKey(
                "clé de session > 128 caractères".into(),
            ));
        }
        Ok(Self(key))
    }

    /// Clé par défaut pour le chat HUD/TUI.
    #[must_use]
    pub fn default_chat() -> Self {
        Self("default".into())
    }

    /// Accès à la représentation string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for SessionKey {
    fn default() -> Self {
        Self::default_chat()
    }
}

impl std::fmt::Display for SessionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Rôle d'un tour de conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TurnRole {
    /// Message utilisateur.
    User,
    /// Réponse assistant.
    Assistant,
    /// Résultat d'outil (Phase 7+).
    Tool,
    /// Message système injecté.
    System,
}

/// Un tour de conversation dans une session agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationTurn {
    /// Rôle du message.
    pub role: TurnRole,
    /// Contenu textuel.
    pub content: String,
    /// Horodatage UTC.
    pub created_at: DateTime<Utc>,
}

impl ConversationTurn {
    /// Crée un tour avec l'horodatage courant.
    #[must_use]
    pub fn new(role: TurnRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
            created_at: Utc::now(),
        }
    }
}

/// Session de conversation agent persistée.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    /// Identifiant logique de session.
    pub key: SessionKey,
    /// Historique ordonné des tours.
    pub turns: Vec<ConversationTurn>,
    /// Création de la session.
    pub created_at: DateTime<Utc>,
    /// Dernière activité.
    pub updated_at: DateTime<Utc>,
}

impl Session {
    /// Nouvelle session vide.
    #[must_use]
    pub fn new(key: SessionKey) -> Self {
        let now = Utc::now();
        Self {
            key,
            turns: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Ajoute un tour et met à jour `updated_at`.
    pub fn push_turn(&mut self, turn: ConversationTurn) {
        self.updated_at = Utc::now();
        self.turns.push(turn);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_key_rejects_empty() {
        assert!(SessionKey::new("").is_err());
        assert!(SessionKey::new("   ").is_err());
    }

    #[test]
    fn session_push_turn_updates_timestamp() {
        let mut session = Session::new(SessionKey::default_chat());
        let before = session.updated_at;
        session.push_turn(ConversationTurn::new(TurnRole::User, "hello"));
        assert!(session.updated_at >= before);
        assert_eq!(session.turns.len(), 1);
    }
}