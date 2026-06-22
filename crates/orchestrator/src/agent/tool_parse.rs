use serde::Deserialize;
use serde_json::Value;

/// Appel d'outil extrait d'une réponse LLM.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ParsedToolCall {
    /// Nom de l'outil.
    pub name: String,
    /// Arguments JSON.
    pub arguments: Value,
}

/// Extrait le premier bloc ```tool ... ``` d'une réponse assistant.
#[must_use]
pub fn extract_tool_call(content: &str) -> Option<ParsedToolCall> {
    let marker = "```tool";
    let start = content.find(marker)? + marker.len();
    let rest = &content[start..];
    let end = rest.find("```")?;
    let json_str = rest[..end].trim();
    serde_json::from_str::<ParsedToolCall>(json_str).ok()
}

/// Indique si le contenu contient un appel outil à traiter.
#[must_use]
pub fn has_tool_call(content: &str) -> bool {
    extract_tool_call(content).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_tool_block() {
        let text = r#"Je cherche.
```tool
{"name":"memory_search","arguments":{"query":"rust"}}
```
"#;
        let call = extract_tool_call(text).unwrap();
        assert_eq!(call.name, "memory_search");
        assert_eq!(call.arguments["query"], "rust");
    }

    #[test]
    fn returns_none_without_tool() {
        assert!(extract_tool_call("réponse simple").is_none());
    }
}