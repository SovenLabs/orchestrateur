use std::path::PathBuf;

use async_trait::async_trait;
use cortex::{serialize_memory, CortexError, Memory, MemoryId, MemoryRepository, parse_memory_markdown};
use tokio::fs;

/// Persistance des mémoires au format Markdown canonique sur disque.
pub struct FileMemoryRepository {
    memories_dir: PathBuf,
}

impl FileMemoryRepository {
    /// Crée un dépôt fichier pointant vers `memories_dir`.
    pub fn new(memories_dir: impl Into<PathBuf>) -> Self {
        Self {
            memories_dir: memories_dir.into(),
        }
    }

    fn file_path(&self, id: MemoryId) -> PathBuf {
        self.memories_dir.join(format!("{id}.md"))
    }

    async fn ensure_dir(&self) -> Result<(), CortexError> {
        fs::create_dir_all(&self.memories_dir)
            .await
            .map_err(|e| CortexError::GraphError(format!("création répertoire mémoires: {e}")))?;
        Ok(())
    }
}

#[async_trait]
impl MemoryRepository for FileMemoryRepository {
    async fn save(&self, memory: &Memory) -> Result<(), CortexError> {
        self.ensure_dir().await?;
        let markdown = serialize_memory(memory)?;
        let path = self.file_path(memory.id);
        fs::write(&path, markdown)
            .await
            .map_err(|e| CortexError::GraphError(format!("écriture {}: {e}", path.display())))?;
        tracing::debug!(path = %path.display(), memory_id = %memory.id, "mémoire persistée sur disque");
        Ok(())
    }

    async fn get_by_id(&self, id: MemoryId) -> Result<Memory, CortexError> {
        let path = self.file_path(id);
        let raw = fs::read_to_string(&path)
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    CortexError::MemoryNotFound(id)
                } else {
                    CortexError::GraphError(format!("lecture {}: {e}", path.display()))
                }
            })?;
        let doc = parse_memory_markdown(&raw)?;
        Ok(doc.memory)
    }

    async fn list(&self) -> Result<Vec<Memory>, CortexError> {
        self.ensure_dir().await?;
        let mut entries = fs::read_dir(&self.memories_dir)
            .await
            .map_err(|e| CortexError::GraphError(format!("listage répertoire: {e}")))?;

        let mut memories = Vec::new();
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| CortexError::GraphError(format!("lecture entrée: {e}")))?
        {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let raw = fs::read_to_string(&path)
                .await
                .map_err(|e| CortexError::GraphError(format!("lecture {}: {e}", path.display())))?;
            let doc = parse_memory_markdown(&raw)?;
            memories.push(doc.memory);
        }
        Ok(memories)
    }

    async fn delete(&self, id: MemoryId) -> Result<(), CortexError> {
        let path = self.file_path(id);
        fs::remove_file(&path)
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    CortexError::MemoryNotFound(id)
                } else {
                    CortexError::GraphError(format!("suppression {}: {e}", path.display()))
                }
            })?;
        Ok(())
    }
}

/// Chemin attendu pour une mémoire (utilitaire de test).
#[cfg(test)]
pub(crate) fn expected_path(dir: &std::path::Path, id: MemoryId) -> PathBuf {
    dir.join(format!("{id}.md"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cortex::Tag;

    #[tokio::test]
    async fn save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let repo = FileMemoryRepository::new(dir.path());
        let mut mem = Memory::new("Titre FS", "Contenu persistant.").unwrap();
        mem.add_tag(Tag::new("rust").unwrap());
        let id = mem.id;

        repo.save(&mem).await.unwrap();
        assert!(expected_path(dir.path(), id).exists());

        let loaded = repo.get_by_id(id).await.unwrap();
        assert_eq!(loaded.title, "Titre FS");
        assert_eq!(loaded.tags.len(), 1);
    }

    #[tokio::test]
    async fn list_reads_all_markdown_files() {
        let dir = tempfile::tempdir().unwrap();
        let repo = FileMemoryRepository::new(dir.path());
        repo.save(&Memory::new("A", "a").unwrap()).await.unwrap();
        repo.save(&Memory::new("B", "b").unwrap()).await.unwrap();
        assert_eq!(repo.list().await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn delete_removes_file() {
        let dir = tempfile::tempdir().unwrap();
        let repo = FileMemoryRepository::new(dir.path());
        let mem = Memory::new("X", "x").unwrap();
        let id = mem.id;
        repo.save(&mem).await.unwrap();
        repo.delete(id).await.unwrap();
        assert!(repo.get_by_id(id).await.is_err());
    }

    #[tokio::test]
    async fn missing_memory_returns_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let repo = FileMemoryRepository::new(dir.path());
        let id = MemoryId::new();
        assert!(matches!(
            repo.get_by_id(id).await.unwrap_err(),
            CortexError::MemoryNotFound(_)
        ));
    }
}