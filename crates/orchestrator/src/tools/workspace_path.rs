//! Résolution de chemins relatifs au workspace (garde-fous traversal).

use std::path::{Component, Path, PathBuf};

/// Résout un chemin relatif sous `workspace` (interdit `..` et chemins absolus).
pub fn resolve_workspace_path(workspace: &Path, rel: &str) -> Result<PathBuf, String> {
    let path = Path::new(rel);
    if path.is_absolute() {
        return Err("chemin absolu interdit".into());
    }
    for component in path.components() {
        if matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        ) {
            return Err("chemin hors workspace".into());
        }
    }
    let resolved = workspace.join(path);
    let canonical_workspace = workspace
        .canonicalize()
        .unwrap_or_else(|_| workspace.to_path_buf());
    let canonical_resolved = resolved.canonicalize().unwrap_or(resolved.clone());
    if !canonical_resolved.starts_with(&canonical_workspace) {
        return Err("chemin hors workspace".into());
    }
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_parent_dir() {
        let ws = PathBuf::from("/tmp/workspace");
        assert!(resolve_workspace_path(&ws, "../etc/passwd").is_err());
    }
}