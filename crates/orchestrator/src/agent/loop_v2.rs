//! Boucle agent v2 — ports Cortex (`ContextProvider`, `AssimilationService`) + LLM.
//!
//! Version simplifiée sans tool-calling ; complète [`AgentLoop`] (Phase 7) pour
//! les intégrations qui consomment directement le contrat `agent_ports`.

use std::sync::Arc;

use cortex::{
    AssimilationPolicy, AssimilationResult, AssimilationService, ContextProvider, SessionKey,
    TurnRole,
};

use crate::llm::{ChatMessage, LlmProvider};

use super::adapters::agent_exchange_turn;
use super::error::AgentError;

/// Réponse d'un tour agent v2.
#[derive(Debug, Clone, PartialEq)]
pub struct AgentResponse {
    /// Contenu assistant renvoyé au client.
    pub content: String,
    /// Résultat d'assimilation (éventuellement vide).
    pub assimilation: AssimilationResult,
}

/// Boucle agent branchée sur les ports Agent ↔ Cortex.
pub struct AgentLoopV2 {
    context_provider: Arc<dyn ContextProvider>,
    assimilation_service: Arc<dyn AssimilationService>,
    llm: Arc<dyn LlmProvider>,
    system_prompt: String,
    context_limit: usize,
    assimilation_policy: AssimilationPolicy,
}

impl AgentLoopV2 {
    /// Crée la boucle avec les ports injectés.
    #[must_use]
    pub fn new(
        context_provider: Arc<dyn ContextProvider>,
        assimilation_service: Arc<dyn AssimilationService>,
        llm: Arc<dyn LlmProvider>,
    ) -> Self {
        Self {
            context_provider,
            assimilation_service,
            llm,
            system_prompt: "Tu es l'assistant souverain Orchestrateur — second cerveau avec mémoire persistante. Réponds en français.".into(),
            context_limit: 10,
            assimilation_policy: AssimilationPolicy::AutoIfChange,
        }
    }

    /// Remplace le prompt système par défaut.
    #[must_use]
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = prompt.into();
        self
    }

    /// Borne la recherche sémantique proactive.
    #[must_use]
    pub fn with_context_limit(mut self, limit: usize) -> Self {
        self.context_limit = limit.max(1);
        self
    }

    /// Politique d'assimilation appliquée après chaque tour.
    #[must_use]
    pub fn with_assimilation_policy(mut self, policy: AssimilationPolicy) -> Self {
        self.assimilation_policy = policy;
        self
    }

    /// Traite un message utilisateur : contexte → LLM → assimilation.
    ///
    /// # Errors
    ///
    /// Propage [`AgentError`] si contexte, LLM ou assimilation échoue.
    pub async fn handle_user_message(
        &self,
        session_id: SessionKey,
        message: &str,
    ) -> Result<AgentResponse, AgentError> {
        let context = self
            .context_provider
            .build_context(message, Some(session_id.clone()), self.context_limit)
            .await?;

        let messages = self.build_messages(&context, message);

        let response = self
            .llm
            .complete_with_context(&self.system_prompt, &messages)
            .await?;

        let turn = agent_exchange_turn(message, &response);
        let assimilation = match self
            .assimilation_service
            .assimilate_turn(&turn, self.assimilation_policy)
            .await
        {
            Ok(result) => result,
            Err(cortex::AssimilationError::UserApprovalRequired(drafts)) => AssimilationResult {
                created: Vec::new(),
                updated: Vec::new(),
                pending_drafts: drafts,
            },
            Err(e) => return Err(AgentError::Assimilation(e)),
        };

        Ok(AgentResponse {
            content: response,
            assimilation,
        })
    }

    fn build_messages(
        &self,
        context: &cortex::AgentContext,
        user_message: &str,
    ) -> Vec<ChatMessage> {
        let mut messages = Vec::new();

        if let Some(graph) = &context.graph_context {
            messages.push(ChatMessage {
                role: "system".into(),
                content: format!("## Graphe Cortex\n{graph}"),
            });
        }

        if !context.memories.is_empty() {
            let lines: Vec<String> = context
                .memories
                .iter()
                .map(|m| format!("- [{}] {}", m.id, m.title))
                .collect();
            messages.push(ChatMessage {
                role: "system".into(),
                content: format!("## Souvenirs pertinents\n{}", lines.join("\n")),
            });
        }

        for turn in &context.session_turns {
            let role = match turn.role {
                TurnRole::User => "user",
                TurnRole::Assistant => "assistant",
                TurnRole::Tool | TurnRole::System => "system",
            };
            messages.push(ChatMessage {
                role: role.into(),
                content: turn.content.clone(),
            });
        }

        messages.push(ChatMessage {
            role: "user".into(),
            content: user_message.into(),
        });

        messages
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::build_agent_adapters;
    use crate::agent::AgentConfig;
    use crate::testing::MockBundle;

    #[tokio::test]
    async fn loop_v2_returns_mock_reply() {
        let bundle = MockBundle::new();
        let deps = bundle.into_deps();
        let llm = deps.llm.clone();
        let (ctx, assim, _) = build_agent_adapters(deps, AgentConfig::default());
        let loop_v2 = AgentLoopV2::new(ctx, assim, llm);
        let session = SessionKey::default_chat();
        let out = loop_v2
            .handle_user_message(session, "Explique l'architecture hexagonale du Cortex")
            .await
            .unwrap();
        assert!(!out.content.is_empty());
    }
}