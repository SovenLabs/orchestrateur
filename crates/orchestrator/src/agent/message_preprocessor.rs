//! Prétraitement bidirectionnel des messages utilisateur avant [`super::context::build_context`] (PR-6).

use crate::deps::AppDependencies;
use crate::llm::ChatMessage;
use crate::tools::{ToolContext, ToolRegistry};

use super::config::AgentConfig;
use super::stream::{AgentStreamEvent, AgentStreamSink};
use super::AgentError;

/// Route choisie par le routeur de prétraitement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreprocessRoute {
    /// Message inchangé.
    PassThrough,
    /// Message court ou vague — enrichissement.
    Expand,
    /// Message trop long — compression map-reduce.
    Compress,
}

/// Résultat du prétraitement d'un message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreprocessResult {
    /// Route appliquée.
    pub route: PreprocessRoute,
    /// Message utilisateur original (persisté en session).
    pub original_message: String,
    /// Message effectif envoyé au LLM et à `build_context`.
    pub effective_message: String,
}

/// Préprocesseur bidirectionnel (expand / compress / pass-through).
pub struct MessagePreprocessor<'a> {
    deps: &'a AppDependencies,
    tools: &'a ToolRegistry,
    config: &'a AgentConfig,
}

impl<'a> MessagePreprocessor<'a> {
    /// Crée un préprocesseur lié aux dépendances et à la configuration agent.
    #[must_use]
    pub fn new(
        deps: &'a AppDependencies,
        tools: &'a ToolRegistry,
        config: &'a AgentConfig,
    ) -> Self {
        Self {
            deps,
            tools,
            config,
        }
    }

    /// Route un message sans effet de bord (tests unitaires du routeur).
    #[must_use]
    pub fn route_message(message: &str, config: &AgentConfig) -> PreprocessRoute {
        if !config.message_preprocess {
            return PreprocessRoute::PassThrough;
        }
        let chars = message.chars().count();
        if chars > config.compression_max_chars {
            return PreprocessRoute::Compress;
        }
        if chars < config.enrichment_min_chars || is_vague(message) {
            return PreprocessRoute::Expand;
        }
        PreprocessRoute::PassThrough
    }

    /// Prétraite le message : ancrage Cortex, expand ou compress, événements stream.
    ///
    /// # Errors
    ///
    /// Propage [`AgentError`] si la recherche mémoire ou le LLM échoue.
    pub async fn preprocess(
        &self,
        message: &str,
        stream: &AgentStreamSink,
    ) -> Result<PreprocessResult, AgentError> {
        let route = Self::route_message(message, self.config);
        let original_message = message.to_string();

        if route == PreprocessRoute::PassThrough {
            return Ok(PreprocessResult {
                route,
                original_message,
                effective_message: message.to_string(),
            });
        }

        stream.emit(AgentStreamEvent::PreprocessProgress {
            stage: match route {
                PreprocessRoute::Expand => "expand_start",
                PreprocessRoute::Compress => "compress_start",
                PreprocessRoute::PassThrough => "pass_through",
            }
            .into(),
            detail: format!("{} caractères", message.chars().count()),
        });

        let memory_anchors = self.fetch_memory_anchors(message, stream).await?;

        let effective_message = match route {
            PreprocessRoute::Expand => {
                self.expand_message(message, &memory_anchors, stream)
                    .await?
            }
            PreprocessRoute::Compress => {
                self.compress_message(message, &memory_anchors, stream)
                    .await?
            }
            PreprocessRoute::PassThrough => unreachable!(),
        };

        match route {
            PreprocessRoute::Expand => {
                stream.emit(AgentStreamEvent::MessageExpanded {
                    original_chars: message.chars().count(),
                    expanded_chars: effective_message.chars().count(),
                });
            }
            PreprocessRoute::Compress => {
                stream.emit(AgentStreamEvent::MessageCompressed {
                    original_chars: message.chars().count(),
                    compressed_chars: effective_message.chars().count(),
                });
            }
            PreprocessRoute::PassThrough => {}
        }

        Ok(PreprocessResult {
            route,
            original_message,
            effective_message,
        })
    }

    async fn fetch_memory_anchors(
        &self,
        message: &str,
        stream: &AgentStreamSink,
    ) -> Result<String, AgentError> {
        stream.emit(AgentStreamEvent::PreprocessProgress {
            stage: "memory_search".into(),
            detail: "ancrage Cortex".into(),
        });

        let ctx = ToolContext::new(self.deps.clone());
        let args = serde_json::json!({
            "query": message,
            "limit": self.config.proactive_search_limit
        });
        match self.tools.execute(&ctx, "memory_search", &args).await {
            Ok(result) => Ok(result.content),
            Err(err) => {
                stream.emit(AgentStreamEvent::PreprocessProgress {
                    stage: "memory_search".into(),
                    detail: format!("indisponible: {err}"),
                });
                Ok(String::new())
            }
        }
    }

    async fn expand_message(
        &self,
        message: &str,
        memory_anchors: &str,
        stream: &AgentStreamSink,
    ) -> Result<String, AgentError> {
        stream.emit(AgentStreamEvent::PreprocessProgress {
            stage: "expand_llm".into(),
            detail: "enrichissement du message".into(),
        });

        let anchors_section = if memory_anchors.is_empty() {
            "(aucun souvenir pertinent)".into()
        } else {
            memory_anchors.to_string()
        };

        let system = "Tu es un préprocesseur de messages pour un agent IA.\n\
            Enrichis un message utilisateur court ou vague en une requête claire, \
            actionnable et autonome en français.\n\
            Utilise les ancrages Cortex fournis pour préciser le contexte si pertinent.\n\
            Réponds uniquement avec le message enrichi, sans commentaire ni markdown.";
        let user = format!(
            "## Message original\n{message}\n\n## Ancrages Cortex\n{anchors_section}"
        );

        let expanded = self
            .deps
            .llm
            .chat(&[
                ChatMessage {
                    role: "system".into(),
                    content: system.into(),
                },
                ChatMessage {
                    role: "user".into(),
                    content: user,
                },
            ])
            .await?;

        let trimmed = expanded.trim();
        if trimmed.is_empty() {
            Ok(message.to_string())
        } else {
            Ok(trimmed.to_string())
        }
    }

    async fn compress_message(
        &self,
        message: &str,
        memory_anchors: &str,
        stream: &AgentStreamSink,
    ) -> Result<String, AgentError> {
        let chunk_size = (self.config.compression_max_chars / 2).max(2000);
        let chunks = split_chunks(message, chunk_size);
        let total = chunks.len();

        stream.emit(AgentStreamEvent::PreprocessProgress {
            stage: "compress_map".into(),
            detail: format!("{total} segment(s)"),
        });

        let mut partial_digests = Vec::with_capacity(total);
        for (index, chunk) in chunks.iter().enumerate() {
            stream.emit(AgentStreamEvent::PreprocessProgress {
                stage: "compress_chunk".into(),
                detail: format!("{}/{}", index + 1, total),
            });
            let digest = self.summarize_chunk(chunk, index + 1, total).await?;
            partial_digests.push(digest);
        }

        stream.emit(AgentStreamEvent::PreprocessProgress {
            stage: "compress_reduce".into(),
            detail: "fusion des résumés".into(),
        });

        let reduced = if partial_digests.len() == 1 {
            partial_digests.into_iter().next().unwrap_or_default()
        } else {
            self.reduce_digests(&partial_digests, memory_anchors)
                .await?
        };

        Ok(format_markdown_digest(message, &reduced, memory_anchors, self.config.compression_preserve_entities))
    }

    async fn summarize_chunk(
        &self,
        chunk: &str,
        index: usize,
        total: usize,
    ) -> Result<String, AgentError> {
        let system = "Tu résumes un segment de texte utilisateur pour un agent IA.\n\
            Conserve les faits, décisions, noms propres, chemins de fichiers et questions ouvertes.\n\
            Réponds en markdown structuré concis (puces).";
        let user = format!(
            "## Segment {index}/{total}\n\n{chunk}"
        );

        let summary = self
            .deps
            .llm
            .chat(&[
                ChatMessage {
                    role: "system".into(),
                    content: system.into(),
                },
                ChatMessage {
                    role: "user".into(),
                    content: user,
                },
            ])
            .await?;
        Ok(summary)
    }

    async fn reduce_digests(
        &self,
        partial_digests: &[String],
        memory_anchors: &str,
    ) -> Result<String, AgentError> {
        let mut body = partial_digests.join("\n\n---\n\n");
        if self.config.compression_preserve_entities && !memory_anchors.is_empty() {
            body.push_str("\n\n## Entités ancrées Cortex\n");
            body.push_str(memory_anchors);
        }

        let system = "Tu fusionnes plusieurs résumés partiels en un digest markdown unique.\n\
            Structure : ## Résumé, ## Points clés, ## Entités et références, ## Questions ouvertes.\n\
            Élimine les redondances. Réponds uniquement en markdown.";
        let user = format!("## Résumés partiels\n\n{body}");

        let reduced = self
            .deps
            .llm
            .chat(&[
                ChatMessage {
                    role: "system".into(),
                    content: system.into(),
                },
                ChatMessage {
                    role: "user".into(),
                    content: user,
                },
            ])
            .await?;
        Ok(reduced)
    }
}

/// Heuristique de message vague (pronoms, acquiescements, requêtes trop courtes).
#[must_use]
pub fn is_vague(message: &str) -> bool {
    let trimmed = message.trim();
    if trimmed.is_empty() {
        return true;
    }

    let lower = trimmed.to_lowercase();
    const VAGUE_UTTERANCES: &[&str] = &[
        "ok", "oui", "non", "continue", "suite", "go", "help", "aide", "ça", "ca", "hmm", "merci",
        "thanks", "next", "plus", "encore", "?", "pourquoi", "comment", "quoi", "et", "donc",
        "alors", "bien", "d'accord", "daccord",
    ];
    if VAGUE_UTTERANCES.contains(&lower.as_str()) {
        return true;
    }

    let word_count = trimmed.split_whitespace().count();
    let chars = trimmed.chars().count();
    word_count <= 2 && !trimmed.contains('?') && chars < 20
}

fn split_chunks(message: &str, chunk_size: usize) -> Vec<String> {
    if message.is_empty() {
        return vec![String::new()];
    }
    let size = chunk_size.max(1);
    message
        .chars()
        .collect::<Vec<_>>()
        .chunks(size)
        .map(|chunk| chunk.iter().collect())
        .collect()
}

fn format_markdown_digest(
    original: &str,
    reduced: &str,
    memory_anchors: &str,
    preserve_entities: bool,
) -> String {
    let mut digest = String::from("# Digest message compressé\n\n");
    digest.push_str(reduced.trim());
    digest.push_str("\n\n");

    if preserve_entities && !memory_anchors.is_empty() {
        digest.push_str("## Ancrages Cortex\n");
        digest.push_str(memory_anchors.trim());
        digest.push_str("\n\n");
    }

    digest.push_str("## Message original (référence)\n");
    digest.push_str("```\n");
    let preview_limit = 500;
    if original.chars().count() <= preview_limit {
        digest.push_str(original);
    } else {
        let preview: String = original.chars().take(preview_limit).collect();
        digest.push_str(&preview);
        digest.push_str("\n… [tronqué]");
    }
    digest.push_str("\n```");
    digest
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::AgentConfig;

    fn test_config() -> AgentConfig {
        AgentConfig {
            message_preprocess: true,
            enrichment_min_chars: 40,
            compression_max_chars: 8000,
            ..AgentConfig::default()
        }
    }

    #[test]
    fn route_passthrough_medium_message() {
        let config = test_config();
        let msg = "Explique le fonctionnement du graphe de connaissances du Cortex en détail.";
        assert_eq!(
            MessagePreprocessor::route_message(msg, &config),
            PreprocessRoute::PassThrough
        );
    }

    #[test]
    fn route_expand_short_message() {
        let config = test_config();
        let msg = "court";
        assert_eq!(
            MessagePreprocessor::route_message(msg, &config),
            PreprocessRoute::Expand
        );
    }

    #[test]
    fn route_expand_vague_message() {
        let config = test_config();
        assert_eq!(
            MessagePreprocessor::route_message("ok", &config),
            PreprocessRoute::Expand
        );
        assert_eq!(
            MessagePreprocessor::route_message("continue", &config),
            PreprocessRoute::Expand
        );
    }

    #[test]
    fn route_compress_long_message() {
        let config = test_config();
        let msg = "a".repeat(8001);
        assert_eq!(
            MessagePreprocessor::route_message(&msg, &config),
            PreprocessRoute::Compress
        );
    }

    #[test]
    fn route_disabled_always_passthrough() {
        let config = AgentConfig {
            message_preprocess: false,
            ..test_config()
        };
        assert_eq!(
            MessagePreprocessor::route_message("ok", &config),
            PreprocessRoute::PassThrough
        );
        let long = "x".repeat(10_000);
        assert_eq!(
            MessagePreprocessor::route_message(&long, &config),
            PreprocessRoute::PassThrough
        );
    }

    #[test]
    fn route_compress_takes_priority_over_vague_when_long() {
        let config = test_config();
        let msg = format!("{} ok", "word ".repeat(2000));
        assert_eq!(
            MessagePreprocessor::route_message(&msg, &config),
            PreprocessRoute::Compress
        );
    }

    #[test]
    fn is_vague_detects_common_utterances() {
        assert!(is_vague("ok"));
        assert!(is_vague("  continue  "));
        assert!(!is_vague(
            "Peux-tu détailler l'architecture du module orchestrateur ?"
        ));
    }

    #[test]
    fn split_chunks_respects_size() {
        let chunks = split_chunks("abcdefgh", 3);
        assert_eq!(chunks, vec!["abc", "def", "gh"]);
    }
}