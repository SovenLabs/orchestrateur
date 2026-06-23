use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use blake3::hash;
use cortex::{Embedding, EmbeddingCapabilities, EmbeddingError, EmbeddingProvider};
use tracing::debug;

/// Cache LRU devant un [`EmbeddingProvider`] — évite les recalculs sur textes identiques.
pub struct CachedEmbeddingProvider {
    inner: Arc<dyn EmbeddingProvider>,
    cache: Mutex<CacheState>,
    max_entries: usize,
}

struct CacheState {
    map: HashMap<String, Embedding>,
    order: VecDeque<String>,
}

impl CachedEmbeddingProvider {
    /// Enveloppe un provider avec un cache borné (défaut recommandé : 4096 entrées).
    #[must_use]
    pub fn new(inner: Arc<dyn EmbeddingProvider>, max_entries: usize) -> Arc<Self> {
        let capacity = max_entries.max(1);
        Arc::new(Self {
            inner,
            cache: Mutex::new(CacheState {
                map: HashMap::with_capacity(capacity.min(1024)),
                order: VecDeque::with_capacity(capacity.min(1024)),
            }),
            max_entries: capacity,
        })
    }

    fn cache_key(text: &str) -> String {
        hash(text.as_bytes()).to_hex().to_string()
    }

    fn get_cached(&self, key: &str) -> Option<Embedding> {
        let guard = self.cache.lock().ok()?;
        guard.map.get(key).cloned()
    }

    fn store_cached(&self, key: String, embedding: Embedding) {
        let Ok(mut guard) = self.cache.lock() else {
            return;
        };
        if guard.map.contains_key(&key) {
            guard.map.insert(key.clone(), embedding);
            if let Some(pos) = guard.order.iter().position(|k| k == &key) {
                guard.order.remove(pos);
                guard.order.push_back(key);
            }
            return;
        }
        while guard.order.len() >= self.max_entries {
            if let Some(old) = guard.order.pop_front() {
                guard.map.remove(&old);
            } else {
                break;
            }
        }
        guard.order.push_back(key.clone());
        guard.map.insert(key, embedding);
    }
}

#[async_trait]
impl EmbeddingProvider for CachedEmbeddingProvider {
    fn name(&self) -> &'static str {
        "cached"
    }

    fn capabilities(&self) -> EmbeddingCapabilities {
        self.inner.capabilities()
    }

    async fn embed(&self, text: &str) -> Result<Embedding, EmbeddingError> {
        let key = Self::cache_key(text);
        if let Some(hit) = self.get_cached(&key) {
            debug!(provider = self.inner.name(), "embedding cache hit");
            return Ok(hit);
        }
        let embedding = self.inner.embed(text).await?;
        self.store_cached(key, embedding.clone());
        Ok(embedding)
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Embedding>, EmbeddingError> {
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            results.push(self.embed(text).await?);
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cortex::EmbeddingProvider;

    struct CountingInner {
        calls: std::sync::atomic::AtomicUsize,
        dim: usize,
    }

    #[async_trait]
    impl EmbeddingProvider for CountingInner {
        fn name(&self) -> &'static str {
            "counting-inner"
        }

        fn capabilities(&self) -> EmbeddingCapabilities {
            EmbeddingCapabilities::default()
        }

        async fn embed(&self, _text: &str) -> Result<Embedding, EmbeddingError> {
            self.calls
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(Embedding::new(vec![1.0; self.dim]))
        }

        async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Embedding>, EmbeddingError> {
            let mut out = Vec::with_capacity(texts.len());
            for _ in texts {
                out.push(self.embed("batch").await?);
            }
            Ok(out)
        }
    }

    #[tokio::test]
    async fn cache_avoids_duplicate_embed_calls() {
        let inner = Arc::new(CountingInner {
            calls: std::sync::atomic::AtomicUsize::new(0),
            dim: 4,
        });
        let cached = CachedEmbeddingProvider::new(inner.clone(), 8);
        let _ = cached.embed("hello").await.unwrap();
        let _ = cached.embed("hello").await.unwrap();
        assert_eq!(
            inner.calls.load(std::sync::atomic::Ordering::SeqCst),
            1
        );
    }
}