use std::sync::Arc;

use cortex::{ConversationTurn, SessionKey, TurnRole};
use crate::deps::AppDependencies;
use crate::skills::marketplace::best_skill_match;
use crate::skills::{SkillContext, SkillRegistry};
use crate::llm::ChatMessage;
use crate::tools::{ToolContext, ToolRegistry};
use tracing::{debug, info};

use super::config::AgentConfig;
use super::stream::{AgentStreamEvent, AgentStreamSink};
use super::context::{base_system_prompt, build_context, format_tool_definitions};
use crate::error::AgentError;
use crate::tool_parse::{extract_tool_call, has_tool_call};

/// Requête d'un tour agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentTurnRequest {
    /// Clé de session (créée si absente).
    pub session_key: SessionKey,
    /// Message utilisateur.
    pub message: String,
}

/// Résultat d'un tour agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentTurnResult {
    /// Réponse finale assistant.
    pub reply: String,
    /// Clé de session utilisée.
    pub session_key: SessionKey,
    /// Noms des outils invoqués pendant le tour.
    pub tools_invoked: Vec<String>,
    /// Message d'assimilation auto (si activée).
    pub auto_assimilated: Option<String>,
    /// Skills exécutées automatiquement avant le LLM (Phase 14).
    pub auto_executed_skills: Vec<String>,
}

/// Boucle agent Phase 7 — LLM + outils mémoire + contexte graphe.
pub struct AgentLoop {
    deps: AppDependencies,
    tools: ToolRegistry,
    config: AgentConfig,
    skills: Option<Arc<SkillRegistry>>,
}

impl AgentLoop {
    /// Crée la boucle avec registre d'outils mémoire par défaut.
    #[must_use]
    pub fn new(
        deps: AppDependencies,
        config: AgentConfig,
        skills: Option<Arc<SkillRegistry>>,
    ) -> Self {
        let tools = ToolRegistry::build_for_agent(&deps, &config, skills.clone());
        Self {
            deps,
            tools,
            config,
            skills,
        }
    }

    /// Crée la boucle avec registre personnalisé.
    #[must_use]
    pub fn with_tools(
        deps: AppDependencies,
        tools: ToolRegistry,
        config: AgentConfig,
        skills: Option<Arc<SkillRegistry>>,
    ) -> Self {
        Self {
            deps,
            tools,
            config,
            skills,
        }
    }

    /// Exécute un tour complet : session → contexte → LLM ↔ outils → persistance.
    ///
    /// # Errors
    ///
    /// Propage [`AgentError`] si session, LLM ou outils échouent.
    pub async fn run_turn(&self, request: AgentTurnRequest) -> Result<AgentTurnResult, AgentError> {
        self.run_turn_with_stream(request, AgentStreamSink::noop())
            .await
    }

    /// Exécute un tour avec émission d'événements [`AgentStreamEvent`] (gateway Phase 8).
    ///
    /// # Errors
    ///
    /// Propage [`AgentError`] si session, LLM ou outils échouent.
    pub async fn run_turn_with_stream(
        &self,
        request: AgentTurnRequest,
        stream: AgentStreamSink,
    ) -> Result<AgentTurnResult, AgentError> {
        let session_key = request.session_key;
        let user_message = request.message;

        self.deps
            .session_repo
            .append_turn(
                &session_key,
                ConversationTurn::new(TurnRole::User, user_message.clone()),
            )
            .await?;

        let built = build_context(
            &self.deps,
            &self.tools,
            &self.config,
            &user_message,
            self.skills.as_deref(),
        )
        .await?;

        let (auto_executed_skills, auto_execute_section) =
            try_auto_execute_skill(&self.config, self.skills.as_deref(), &user_message).await;

        let mut context_section = built.system_context;
        if !auto_execute_section.is_empty() {
            if context_section.is_empty() {
                context_section = auto_execute_section;
            } else {
                context_section.push_str("\n\n");
                context_section.push_str(&auto_execute_section);
            }
        }

        let tool_section = format_tool_definitions(&self.tools);
        let system_prompt = base_system_prompt(&tool_section, &context_section);

        let history = self
            .deps
            .session_repo
            .list_turns(&session_key)
            .await?;
        let mut messages = vec![ChatMessage {
            role: "system".into(),
            content: system_prompt,
        }];

        let history_start = history.len().saturating_sub(self.config.max_history_turns);
        for turn in history.iter().skip(history_start) {
            let role = match turn.role {
                TurnRole::User => "user",
                TurnRole::Assistant => "assistant",
                TurnRole::Tool => "user",
                TurnRole::System => "system",
            };
            messages.push(ChatMessage {
                role: role.into(),
                content: turn.content.clone(),
            });
        }

        let tool_ctx = ToolContext::new(self.deps.clone());
        let mut tools_invoked = Vec::new();
        let mut reply = String::new();

        for iteration in 0..self.config.max_tool_iterations {
            let assistant_raw = self.deps.llm.chat(&messages).await?;
            debug!(iteration, "réponse LLM reçue");

            if !has_tool_call(&assistant_raw) {
                reply = assistant_raw;
                break;
            }

            let Some(call) = extract_tool_call(&assistant_raw) else {
                reply = assistant_raw;
                break;
            };

            info!(tool = %call.name, "exécution outil agent");
            tools_invoked.push(call.name.clone());
            stream.emit(AgentStreamEvent::ToolStart {
                name: call.name.clone(),
            });

            let tool_result = match self
                .tools
                .execute(&tool_ctx, &call.name, &call.arguments)
                .await
            {
                Ok(result) => {
                    stream.emit(AgentStreamEvent::ToolEnd {
                        name: call.name.clone(),
                        success: true,
                    });
                    result
                }
                Err(err) => {
                    stream.emit(AgentStreamEvent::ToolEnd {
                        name: call.name.clone(),
                        success: false,
                    });
                    return Err(err);
                }
            };

            messages.push(ChatMessage {
                role: "assistant".into(),
                content: assistant_raw,
            });
            messages.push(ChatMessage {
                role: "user".into(),
                content: format!(
                    "Résultat outil {}:\n{}",
                    call.name, tool_result.content
                ),
            });

            if iteration + 1 >= self.config.max_tool_iterations {
                return Err(AgentError::MaxToolIterations {
                    max: self.config.max_tool_iterations,
                });
            }
        }

        if reply.is_empty() {
            return Err(AgentError::InvalidLlmResponse(
                "réponse assistant vide après boucle outils".into(),
            ));
        }

        self.deps
            .session_repo
            .append_turn(
                &session_key,
                ConversationTurn::new(TurnRole::Assistant, reply.clone()),
            )
            .await?;

        let auto_assimilated = if self.config.auto_assimilate_turn {
            let summary = format!(
                "Tour agent session {} — utilisateur: {}\nassistant: {}",
                session_key, user_message, reply
            );
            let args = serde_json::json!({ "text": summary, "tags": ["agent-turn"] });
            match self.tools.execute(&tool_ctx, "memory_assimilate", &args).await {
                Ok(r) => Some(r.content),
                Err(e) => {
                    debug!(%e, "auto-assimilation échouée");
                    None
                }
            }
        } else {
            None
        };

        for chunk in chunk_text(&reply, 48) {
            stream.emit(AgentStreamEvent::Delta { content: chunk });
        }
        stream.emit(AgentStreamEvent::End {
            reply: reply.clone(),
            tools_invoked: tools_invoked.clone(),
        });

        Ok(AgentTurnResult {
            reply,
            session_key,
            tools_invoked,
            auto_assimilated,
            auto_executed_skills,
        })
    }
}

async fn try_auto_execute_skill(
    config: &AgentConfig,
    skills: Option<&SkillRegistry>,
    user_message: &str,
) -> (Vec<String>, String) {
    if !config.skill_auto_execute || !config.skill_tools_enabled {
        return (Vec::new(), String::new());
    }
    let Some(registry) = skills else {
        return (Vec::new(), String::new());
    };
    let Some((score, entry)) = best_skill_match(&registry.list(), user_message) else {
        return (Vec::new(), String::new());
    };
    if score < config.skill_auto_execute_threshold {
        return (Vec::new(), String::new());
    }
    let skill_ctx = SkillContext {
        query: Some(user_message.to_string()),
        text: None,
        tags: Vec::new(),
        limit: None,
    };
    match registry.execute(&entry.name, &skill_ctx).await {
        Ok(output) => {
            info!(
                skill = %entry.name,
                score,
                "auto-exécution skill avant LLM"
            );
            let section = format!(
                "## Skill auto-exécutée ({}) — score {score}\n{}",
                entry.name, output.message
            );
            (vec![entry.name], section)
        }
        Err(err) => {
            debug!(skill = %entry.name, %err, "auto-exécution skill échouée");
            (
                Vec::new(),
                format!(
                    "## Skill auto-exécutée ({entry.name})\n(échec: {err})"
                ),
            )
        }
    }
}

fn chunk_text(text: &str, chunk_size: usize) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }
    let size = chunk_size.max(1);
    text.chars()
        .collect::<Vec<_>>()
        .chunks(size)
        .map(|chunk| chunk.iter().collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::stream::{AgentStreamEvent, AgentStreamSink};
    use cortex::Memory;
    use crate::testing::MockBundle;

    #[tokio::test]
    async fn run_turn_simple_chat() {
        let deps = MockBundle::new().into_deps();
        let agent = AgentLoop::new(deps, AgentConfig::default(), None);
        let result = agent
            .run_turn(AgentTurnRequest {
                session_key: SessionKey::default_chat(),
                message: "Bonjour".into(),
            })
            .await
            .unwrap();
        assert_eq!(result.reply, "Bonjour");
        assert!(result.tools_invoked.is_empty());
    }

    #[tokio::test]
    async fn run_turn_emits_stream_events() {
        let deps = MockBundle::new().into_deps();
        let agent = AgentLoop::new(deps, AgentConfig::default(), None);
        let (tx, rx) = flume::unbounded();
        let sink = AgentStreamSink::from_sender(tx);
        let result = agent
            .run_turn_with_stream(
                AgentTurnRequest {
                    session_key: SessionKey::default_chat(),
                    message: "Stream".into(),
                },
                sink,
            )
            .await
            .unwrap();
        assert_eq!(result.reply, "Stream");
        let mut saw_delta = false;
        let mut saw_end = false;
        while let Ok(event) = rx.try_recv() {
            match event {
                AgentStreamEvent::Delta { .. } => saw_delta = true,
                AgentStreamEvent::End { reply, .. } => {
                    saw_end = true;
                    assert_eq!(reply, "Stream");
                }
                _ => {}
            }
        }
        assert!(saw_delta);
        assert!(saw_end);
    }

    #[tokio::test]
    async fn run_turn_with_memory_context() {
        let bundle = MockBundle::new();
        let mem = Memory::new("Rust agent", "Le Cortex est souverain.").unwrap();
        bundle.memory_repo.save(&mem).await.unwrap();
        let deps = bundle.into_deps();
        let agent = AgentLoop::new(deps, AgentConfig::default(), None);
        let result = agent
            .run_turn(AgentTurnRequest {
                session_key: SessionKey::new("test").unwrap(),
                message: "Cortex".into(),
            })
            .await
            .unwrap();
        assert!(!result.reply.is_empty());
    }
}