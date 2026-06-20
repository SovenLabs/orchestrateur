//! Parser et sérialiseur du format Markdown canonique des mémoires.
//!
//! Contrat verrouillé : frontmatter YAML avec `deny_unknown_fields`,
//! délimiteurs `---` détectés explicitement (support `\r\n`, corps contenant `---`).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::{Backlink, CortexError, Memory, MemoryId, Tag};

/// Frontmatter YAML — contrat strict (champs inconnus rejetés).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MemoryFrontmatter {
    id: MemoryId,
    title: String,
    tags: Vec<Tag>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    backlinks: Vec<Backlink>,
}

/// Document Markdown complet (frontmatter + corps).
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDocument {
    /// Mémoire matérialisée après validation domaine.
    pub memory: Memory,
}

/// Parser du format Markdown canonique Orchestrateur.
pub struct MarkdownParser;

impl MarkdownParser {
    /// Parse un document Markdown vers une [`Memory`] validée.
    ///
    /// # Errors
    ///
    /// Propage les erreurs de [`parse_memory_markdown`] et de validation domaine.
    pub fn parse(markdown: &str) -> Result<Memory, CortexError> {
        let doc = parse_memory_markdown(markdown)?;
        Ok(doc.memory)
    }

    /// Sérialise une mémoire au format Markdown canonique.
    ///
    /// # Errors
    ///
    /// Propage les erreurs de [`serialize_memory`].
    pub fn serialize(memory: &Memory) -> Result<String, CortexError> {
        serialize_memory(memory)
    }
}

/// Parse le format Markdown canonique (`---` / YAML / `---` / contenu).
///
/// # Errors
///
/// Retourne [`CortexError::InvalidMarkdown`], [`CortexError::InvalidFrontmatter`]
/// ou une erreur de validation [`Memory::reconstruct`].
pub fn parse_memory_markdown(raw: &str) -> Result<MemoryDocument, CortexError> {
    let (yaml_part, body_part) = split_frontmatter(raw.trim())?;

    let frontmatter: MemoryFrontmatter = serde_yaml::from_str(yaml_part)
        .map_err(|e| CortexError::InvalidFrontmatter(e.to_string()))?;

    let memory = Memory::reconstruct(
        frontmatter.id,
        frontmatter.title,
        body_part,
        frontmatter.tags,
        frontmatter.created_at,
        frontmatter.updated_at,
        frontmatter.backlinks,
    )?;

    Ok(MemoryDocument { memory })
}

/// Sérialise une mémoire au format Markdown canonique.
///
/// # Errors
///
/// Retourne [`CortexError::InvalidFrontmatter`] si la sérialisation YAML échoue.
pub fn serialize_memory(memory: &Memory) -> Result<String, CortexError> {
    let frontmatter = MemoryFrontmatter {
        id: memory.id,
        title: memory.title.clone(),
        tags: memory.tags.clone(),
        created_at: memory.created_at,
        updated_at: memory.updated_at,
        backlinks: memory.backlinks.clone(),
    };

    let yaml = serde_yaml::to_string(&frontmatter)
        .map_err(|e| CortexError::InvalidFrontmatter(e.to_string()))?;

    Ok(format!("---\n{yaml}---\n\n{}", memory.content))
}

/// Détecte le frontmatter fermant sur une ligne `---` seule.
fn split_frontmatter(input: &str) -> Result<(&str, String), CortexError> {
    let after_first = if let Some(rest) = input.strip_prefix("---\r\n") {
        rest
    } else if let Some(rest) = input.strip_prefix("---\n") {
        rest
    } else if let Some(rest) = input.strip_prefix("---") {
        rest.trim_start_matches(['\r', '\n'])
    } else {
        return Err(CortexError::InvalidMarkdown(
            "le document doit commencer par '---\\n' ou '---\\r\\n'".into(),
        ));
    };

    let close_idx = find_closing_fence(after_first).ok_or_else(|| {
        CortexError::InvalidMarkdown("frontmatter non clos (ligne '---' attendue)".into())
    })?;

    let yaml = after_first[..close_idx].trim();
    let body_start = close_idx + 3; // longueur de "---"
    let raw_body = if body_start < after_first.len() {
        &after_first[body_start..]
    } else {
        ""
    };
    let body = raw_body.trim_start_matches(['\r', '\n']).to_string();

    Ok((yaml, body))
}

fn find_closing_fence(input: &str) -> Option<usize> {
    for (byte_idx, _) in input.match_indices("\n---") {
        let line_start = byte_idx + 1;
        let rest = &input[line_start..];
        if rest == "---" {
            return Some(line_start);
        }
        if rest.starts_with("---\n") || rest.starts_with("---\r\n") {
            return Some(line_start);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::BacklinkKind;

    fn sample_markdown() -> String {
        let id = MemoryId::new();
        format!(
            r#"---
id: "{id}"
title: "Décision stratégique"
tags: ["architecture", "cortex"]
created_at: "2026-06-20T12:00:00Z"
updated_at: "2026-06-20T12:00:00Z"
backlinks:
  - target: "{id}"
    score: 0.87
    kind: semantic
---

Contenu complet du souvenir en Markdown pur...
"#
        )
    }

    #[test]
    fn parses_canonical_format() {
        let raw = sample_markdown();
        let doc = parse_memory_markdown(&raw).unwrap();
        assert_eq!(doc.memory.title, "Décision stratégique");
        assert_eq!(doc.memory.tags.len(), 2);
        assert!(doc.memory.content.contains("Contenu complet"));
    }

    #[test]
    fn roundtrip_serialize_parse() {
        let mut mem = Memory::new("Test RT", "Corps du souvenir").unwrap();
        mem.add_tag(Tag::new("rust").unwrap());
        let bl = Backlink::new(mem.id, 0.5, BacklinkKind::Semantic).unwrap();
        mem.set_backlinks(vec![bl]);

        let md = serialize_memory(&mem).unwrap();
        let parsed = parse_memory_markdown(&md).unwrap();
        assert_eq!(parsed.memory.id, mem.id);
        assert_eq!(parsed.memory.title, mem.title);
        assert_eq!(parsed.memory.content, mem.content);
        assert_eq!(parsed.memory.tags, mem.tags);
        assert_eq!(parsed.memory.backlinks.len(), 1);
    }

    #[test]
    fn rejects_missing_frontmatter() {
        let err = parse_memory_markdown("# just markdown").unwrap_err();
        assert!(matches!(err, CortexError::InvalidMarkdown(_)));
    }

    #[test]
    fn rejects_invalid_yaml() {
        let raw = "---\nnot: [valid: yaml\n---\n\nbody";
        let err = parse_memory_markdown(raw).unwrap_err();
        assert!(matches!(err, CortexError::InvalidFrontmatter(_)));
    }

    #[test]
    fn rejects_unknown_frontmatter_field() {
        let raw = "---\nid: \"0192a3b4-8c2f-7a1e-9b3d-2e4f5a6b7c8d\"\ntitle: T\nunknown_field: true\ncreated_at: \"2026-06-20T12:00:00Z\"\nupdated_at: \"2026-06-20T12:00:00Z\"\ntags: []\nbacklinks: []\n---\n\nbody";
        let err = parse_memory_markdown(raw).unwrap_err();
        assert!(matches!(err, CortexError::InvalidFrontmatter(_)));
    }

    #[test]
    fn parses_body_containing_horizontal_rules() {
        let id = MemoryId::new();
        let raw = format!(
            "---\nid: \"{id}\"\ntitle: \"T\"\ntags: []\ncreated_at: \"2026-06-20T12:00:00Z\"\nupdated_at: \"2026-06-20T12:00:00Z\"\nbacklinks: []\n---\n\nSection\n---\n\nSuite du contenu."
        );
        let mem = MarkdownParser::parse(&raw).unwrap();
        assert!(mem.content.contains("Section"));
        assert!(mem.content.contains("Suite du contenu"));
    }

    #[test]
    fn parses_crlf_delimiters() {
        let id = MemoryId::new();
        let raw = format!(
            "---\r\nid: \"{id}\"\r\ntitle: \"CRLF\"\r\ntags: []\r\ncreated_at: \"2026-06-20T12:00:00Z\"\r\nupdated_at: \"2026-06-20T12:00:00Z\"\r\nbacklinks: []\r\n---\r\n\r\nCorps CRLF."
        );
        let mem = MarkdownParser::parse(&raw).unwrap();
        assert_eq!(mem.title, "CRLF");
        assert!(mem.content.contains("Corps CRLF"));
    }
}
