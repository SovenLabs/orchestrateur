use cortex::MemoryDraft;

use crate::llm::{ChatMessage, LlmError, LlmProvider};

/// Prompt système pour extraction d'insights typés (Pulse-style).
pub const INSIGHT_ASSIMILATION_SYSTEM_PROMPT: &str = r#"Tu es l'assistant d'assimilation de l'Orchestrateur.
Analyse le texte fourni et produis UNIQUEMENT un objet JSON valide.

Si le texte ne contient AUCUNE contribution significative (salutations, remplissage, bruit) :
{"skip": true, "reason": "explication courte"}

Sinon produis :
{
  "title": "string non vide",
  "content": "string markdown non vide",
  "kind": "decision"|"dead_end"|"pattern"|"context"|"progress"|"business",
  "tags": ["minuscules-sans-espaces"],
  "structured": { ... champs selon kind ... },
  "source_files": ["chemins/relatifs/optionnels"],
  "backlinks": [{"target": "uuid", "score": 0.0-1.0, "kind": "semantic"|"explicit_wikilink"}]
}

Ne produis aucun texte hors JSON."#;

/// Construit le prompt utilisateur avec contexte lié optionnel.
#[must_use]
pub fn build_insight_user_prompt(text: &str, related_context: &str, tags: &[String]) -> String {
    let mut parts = Vec::new();
    if !tags.is_empty() {
        parts.push(format!("Tags suggérés : {}", tags.join(", ")));
    }
    if !related_context.is_empty() {
        parts.push(format!("## Souvenirs liés\n{related_context}"));
    }
    parts.push(format!("## Texte à analyser\n{text}"));
    parts.join("\n\n")
}

/// Extrait un insight via chat JSON (gère `skip`).
pub async fn generate_insight_draft(
    llm: &dyn LlmProvider,
    system: &str,
    user: &str,
) -> Result<Option<MemoryDraft>, LlmError> {
    let messages = [
        ChatMessage {
            role: "system".into(),
            content: system.into(),
        },
        ChatMessage {
            role: "user".into(),
            content: user.into(),
        },
    ];
    let raw = llm.chat(&messages).await?;
    parse_insight_response(&raw).map_err(|message| LlmError::StructuredOutputInvalid {
        provider: llm.name().into(),
        message,
    })
}

/// Parse la réponse JSON LLM — `None` si skip.
pub fn parse_insight_response(raw_json: &str) -> Result<Option<MemoryDraft>, String> {
    let value: serde_json::Value =
        serde_json::from_str(raw_json).map_err(|e| format!("JSON invalide: {e}"))?;
    if value
        .get("skip")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        return Ok(None);
    }
    let draft: MemoryDraft =
        serde_json::from_value(value).map_err(|e| format!("brouillon invalide: {e}"))?;
    if draft.title.trim().is_empty() || draft.content.trim().is_empty() {
        return Err("title ou content vide".into());
    }
    Ok(Some(draft))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cortex::MemoryKind;

    #[test]
    fn parses_skip_response() {
        let raw = r#"{"skip": true, "reason": "rien d'utile"}"#;
        assert!(parse_insight_response(raw).unwrap().is_none());
    }

    #[test]
    fn parses_valid_draft() {
        let raw = r#"{
            "title": "Choix Rust",
            "content": "On garde Rust.",
            "kind": "decision",
            "tags": ["rust"],
            "structured": {"rationale": "perf"},
            "source_files": ["src/main.rs"]
        }"#;
        let draft = parse_insight_response(raw).unwrap().unwrap();
        assert_eq!(draft.kind, MemoryKind::Decision);
        assert_eq!(draft.source_files, vec!["src/main.rs"]);
    }
}