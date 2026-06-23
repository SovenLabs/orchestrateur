//! Édition chirurgicale de `orchestrator.toml` (onboard / channels enable).

use std::path::Path;

use super::ConfigError;

/// Active ou désactive un canal gateway dans le TOML.
///
/// # Errors
///
/// Propage [`ConfigError`] si le fichier est illisible.
pub fn set_channel_enabled(path: &Path, channel_id: &str, enabled: bool) -> Result<(), ConfigError> {
    let section = channel_section_name(channel_id);
    let key = format!("enabled = {enabled}");
    patch_toml_section_key(path, &section, "enabled", &key)
}

/// Définit le profil sécurité (`[security] profile = "..."`).
pub fn set_security_profile(path: &Path, profile: &str) -> Result<(), ConfigError> {
    patch_toml_section_key(
        path,
        "[security]",
        "profile",
        &format!("profile = \"{profile}\""),
    )
}

/// Définit le provider LLM primaire.
pub fn set_primary_llm(path: &Path, provider: &str) -> Result<(), ConfigError> {
    patch_toml_section_key(
        path,
        "[providers]",
        "primary_llm",
        &format!("primary_llm = \"{provider}\""),
    )
}

fn channel_section_name(channel_id: &str) -> String {
    match channel_id {
        "telegram" | "discord" | "slack" | "webhook" => {
            format!("[gateway.{channel_id}]")
        }
        _ => format!("[gateway.channels.{channel_id}]"),
    }
}

fn patch_toml_section_key(
    path: &Path,
    section_header: &str,
    key: &str,
    replacement_line: &str,
) -> Result<(), ConfigError> {
    let raw = std::fs::read_to_string(path).map_err(|e| ConfigError::Io {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;
    let mut lines: Vec<String> = raw.lines().map(str::to_string).collect();
    let section_idx = lines
        .iter()
        .position(|l| l.trim() == section_header)
        .ok_or_else(|| ConfigError::Parse {
            path: path.to_path_buf(),
            message: format!("section {section_header} introuvable"),
        })?;
    let mut end = lines.len();
    for i in (section_idx + 1)..lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            end = i;
            break;
        }
    }
    let mut found = false;
    for line in lines.iter_mut().take(end).skip(section_idx + 1) {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with(&format!("{key} =")) {
            *line = replacement_line.to_string();
            found = true;
            break;
        }
    }
    if !found {
        lines.insert(section_idx + 1, replacement_line.to_string());
    }
    let out = format!("{}\n", lines.join("\n"));
    std::fs::write(path, out).map_err(|e| ConfigError::Io {
        path: path.to_path_buf(),
        message: e.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn toggles_telegram_enabled() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("orchestrator.toml");
        let sample = r#"
[security]
profile = "ai_assisted"

[gateway.telegram]
enabled = false
token_env = "TELEGRAM_BOT_TOKEN"
"#;
        let mut f = std::fs::File::create(&path).expect("create");
        write!(f, "{sample}").expect("write");
        set_channel_enabled(&path, "telegram", true).expect("patch");
        let body = std::fs::read_to_string(&path).expect("read");
        assert!(body.contains("enabled = true"));
    }
}