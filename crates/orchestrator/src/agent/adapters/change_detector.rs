//! Détection de changement significatif pour `AssimilationPolicy::AutoIfChange`.

use cortex::{ConversationTurn, SemanticSearch, TurnRole};

use crate::agent::adapters::semantic_search::CortexSemanticSearch;
use cortex::AssimilationError;

/// Seuils de détection de nouveauté.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChangeDetectorConfig {
    /// Longueur minimale du texte combiné user+assistant.
    pub min_combined_chars: usize,
    /// Score sémantique au-dessus duquel le tour est considéré redondant.
    pub max_redundancy_score: f32,
}

impl Default for ChangeDetectorConfig {
    fn default() -> Self {
        Self {
            min_combined_chars: 60,
            max_redundancy_score: 0.82,
        }
    }
}

/// Détermine si un échange agent mérite une assimilation.
pub struct ChangeDetector {
    config: ChangeDetectorConfig,
}

impl ChangeDetector {
    /// Crée un détecteur avec la configuration donnée.
    #[must_use]
    pub fn new(config: ChangeDetectorConfig) -> Self {
        Self { config }
    }

    /// Heuristiques synchrones (longueur, salutations triviales).
    #[must_use]
    pub fn is_trivial_exchange(&self, user: &str, assistant: &str) -> bool {
        let combined = format!("{user}\n{assistant}");
        if combined.chars().count() < self.config.min_combined_chars {
            return true;
        }
        let user_l = user.trim().to_lowercase();
        let asst_l = assistant.trim().to_lowercase();
        if user_l.is_empty() || asst_l.is_empty() {
            return true;
        }
        const TRIVIAL: &[&str] = &[
            "ok", "okay", "merci", "thanks", "salut", "hello", "bonjour", "d'accord",
        ];
        if TRIVIAL.contains(&user_l.as_str()) && asst_l.chars().count() < 120 {
            return true;
        }
        false
    }

    /// Vérifie la redondance sémantique via recherche Cortex.
    pub async fn is_semantically_redundant(
        &self,
        search: &CortexSemanticSearch,
        user: &str,
        assistant: &str,
    ) -> Result<bool, AssimilationError> {
        let query = format!("{user}\n{assistant}");
        let hits = match search.search(&query, 3).await {
            Ok(h) => h,
            Err(cortex::RetrievalError::NoRelevantMemories) => return Ok(false),
            Err(_) => return Err(AssimilationError::ChangeDetectionFailed),
        };
        let top = hits.first().map(|h| h.score).unwrap_or(0.0);
        Ok(top >= self.config.max_redundancy_score)
    }

    /// Retourne `true` si l'échange doit être assimilé.
    pub async fn should_assimilate(
        &self,
        search: &CortexSemanticSearch,
        user: &str,
        assistant: &str,
    ) -> Result<bool, AssimilationError> {
        if self.is_trivial_exchange(user, assistant) {
            return Ok(false);
        }
        if self
            .is_semantically_redundant(search, user, assistant)
            .await?
        {
            return Ok(false);
        }
        Ok(true)
    }
}

/// Encode un échange user/assistant dans un tour `System` pour [`AssimilationService`].
#[must_use]
pub fn agent_exchange_turn(user: &str, assistant: &str) -> ConversationTurn {
    ConversationTurn::new(
        TurnRole::System,
        format!("__agent_exchange__\nuser:{user}\nassistant:{assistant}"),
    )
}

/// Décode un tour produit par [`agent_exchange_turn`].
pub(crate) fn parse_agent_exchange(turn: &ConversationTurn) -> Result<(String, String), AssimilationError> {
    if turn.role != TurnRole::System || !turn.content.starts_with("__agent_exchange__") {
        return Err(AssimilationError::ChangeDetectionFailed);
    }
    let mut user = None;
    let mut assistant = None;
    for line in turn.content.lines().skip(1) {
        if let Some(rest) = line.strip_prefix("user:") {
            user = Some(rest.to_string());
        } else if let Some(rest) = line.strip_prefix("assistant:") {
            assistant = Some(rest.to_string());
        }
    }
    match (user, assistant) {
        (Some(u), Some(a)) if !u.trim().is_empty() && !a.trim().is_empty() => Ok((u, a)),
        _ => Err(AssimilationError::ChangeDetectionFailed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trivial_short_exchange() {
        let d = ChangeDetector::new(ChangeDetectorConfig::default());
        assert!(d.is_trivial_exchange("ok", "D'accord !"));
    }

    #[test]
    fn roundtrip_exchange_turn() {
        let turn = agent_exchange_turn("Question ?", "Réponse détaillée.");
        let (u, a) = parse_agent_exchange(&turn).unwrap();
        assert_eq!(u, "Question ?");
        assert_eq!(a, "Réponse détaillée.");
    }
}