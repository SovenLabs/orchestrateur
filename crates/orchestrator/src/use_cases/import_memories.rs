use std::path::{Path, PathBuf};

use cortex::{parse_memory_markdown, CortexError, MemoryId};

use crate::deps::AppDependencies;
use crate::error::OrchestratorError;
use crate::use_cases::SaveMemory;

/// Résultat d'un import de mémoires Markdown depuis un répertoire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportResult {
    /// Nombre de mémoires importées.
    pub imported: usize,
    /// Nombre de doublons ignorés (même `MemoryId`).
    pub skipped: usize,
    /// Erreurs par fichier (parse ou persistance).
    pub errors: Vec<String>,
}

/// Use case : importe des fichiers `*.md` depuis un répertoire.
pub struct ImportMemories {
    deps: AppDependencies,
}

impl ImportMemories {
    /// Crée le use case avec les dépendances injectées.
    #[must_use]
    pub fn new(deps: AppDependencies) -> Self {
        Self { deps }
    }

    /// Scanne `source_dir` pour les `*.md`, parse et persiste les nouvelles mémoires.
    ///
    /// # Errors
    ///
    /// Propage une [`OrchestratorError`] si la lecture du répertoire échoue.
    pub async fn execute(&self, source_dir: &Path) -> Result<ImportResult, OrchestratorError> {
        let mut files = Vec::new();
        collect_md_files(source_dir, &mut files)?;
        files.sort();

        let save = SaveMemory::new(self.deps.clone());
        let mut result = ImportResult {
            imported: 0,
            skipped: 0,
            errors: Vec::new(),
        };

        for path in files {
            let raw = match std::fs::read_to_string(&path) {
                Ok(content) => content,
                Err(err) => {
                    result
                        .errors
                        .push(format!("{}: lecture: {err}", path.display()));
                    continue;
                }
            };

            let doc = match parse_memory_markdown(&raw) {
                Ok(doc) => doc,
                Err(err) => {
                    result
                        .errors
                        .push(format!("{}: parse: {err}", path.display()));
                    continue;
                }
            };

            let memory_id = doc.memory.id;
            if memory_exists(&self.deps, memory_id).await? {
                result.skipped += 1;
                continue;
            }

            match save.execute(&doc.memory).await {
                Ok(_) => result.imported += 1,
                Err(err) => result
                    .errors
                    .push(format!("{}: save: {err}", path.display())),
            }
        }

        Ok(result)
    }
}

fn collect_md_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), OrchestratorError> {
    if !dir.is_dir() {
        return Err(OrchestratorError::Cortex(CortexError::InvalidMarkdown(format!(
            "répertoire source introuvable: {}",
            dir.display()
        ))));
    }
    for entry in std::fs::read_dir(dir).map_err(|e| {
        OrchestratorError::Cortex(CortexError::InvalidMarkdown(format!(
            "lecture {}: {e}",
            dir.display()
        )))
    })? {
        let entry = entry.map_err(|e| {
            OrchestratorError::Cortex(CortexError::InvalidMarkdown(format!(
                "entrée {}: {e}",
                dir.display()
            )))
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_md_files(&path, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            out.push(path);
        }
    }
    Ok(())
}

async fn memory_exists(deps: &AppDependencies, id: MemoryId) -> Result<bool, OrchestratorError> {
    match deps.memory_repo.get_by_id(id).await {
        Ok(_) => Ok(true),
        Err(cortex::CortexError::MemoryNotFound(_)) => Ok(false),
        Err(err) => Err(OrchestratorError::Cortex(err)),
    }
}

#[cfg(test)]
mod tests {
    use cortex::{serialize_memory, Memory};

    use super::*;
    use crate::testing::MockBundle;

    #[tokio::test]
    async fn imports_new_memories_and_skips_duplicates() {
        let bundle = MockBundle::new();
        let deps = bundle.into_deps();
        let dir = tempfile::tempdir().expect("tempdir");

        let mem = Memory::new("Import A", "Corps A").expect("memory");
        let md = serialize_memory(&mem).expect("serialize");
        let path = dir.path().join(format!("{}.md", mem.id));
        std::fs::write(&path, md).expect("write");

        let uc = ImportMemories::new(deps.clone());
        let first = uc.execute(dir.path()).await.expect("import");
        assert_eq!(first.imported, 1);
        assert_eq!(first.skipped, 0);
        assert!(first.errors.is_empty());

        let second = uc.execute(dir.path()).await.expect("reimport");
        assert_eq!(second.imported, 0);
        assert_eq!(second.skipped, 1);
    }

    #[tokio::test]
    async fn reports_parse_errors_without_aborting() {
        let bundle = MockBundle::new();
        let deps = bundle.into_deps();
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("bad.md"), "# not valid").expect("write");

        let mem = Memory::new("OK", "C").expect("memory");
        let md = serialize_memory(&mem).expect("serialize");
        std::fs::write(dir.path().join("good.md"), md).expect("write");

        let uc = ImportMemories::new(deps);
        let result = uc.execute(dir.path()).await.expect("import");
        assert_eq!(result.imported, 1);
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].contains("bad.md"));
    }
}