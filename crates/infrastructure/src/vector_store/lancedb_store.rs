use std::path::Path;
use std::sync::Arc;

use arrow_array::types::Float32Type;
use arrow_array::{
    Array, FixedSizeListArray, Float32Array, RecordBatch, RecordBatchIterator, StringArray,
};
use arrow_schema::{DataType, Field, Schema};
use async_trait::async_trait;
use cortex::{CortexError, MemoryId, SearchFilter, SearchHit, VectorStore};
use futures::TryStreamExt;
use lancedb::index::vector::IvfFlatIndexBuilder;
use lancedb::index::Index;
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::{connect, Connection, Table};
use tokio::sync::Mutex;
use tracing::instrument;

const TABLE_NAME: &str = "memories";

/// Vector store `LanceDB` — persistance locale avec index ANN.
#[derive(Clone)]
pub struct LancedbVectorStore {
    connection: Connection,
    dimension: usize,
    table: Arc<Mutex<Option<Table>>>,
    index_created: Arc<Mutex<bool>>,
}

impl LancedbVectorStore {
    /// Ouvre (ou crée) une base `LanceDB` au chemin indiqué.
    ///
    /// # Errors
    ///
    /// Retourne [`CortexError::GraphError`] si la connexion ou la table échoue.
    pub async fn open(path: impl AsRef<Path>, dimension: usize) -> Result<Self, CortexError> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent).map_err(|e| CortexError::GraphError(e.to_string()))?;
        }
        let connection = connect(&path_str)
            .execute()
            .await
            .map_err(|e| CortexError::GraphError(format!("connexion LanceDB: {e}")))?;

        let store = Self {
            connection,
            dimension,
            table: Arc::new(Mutex::new(None)),
            index_created: Arc::new(Mutex::new(false)),
        };
        store.ensure_table().await?;
        Ok(store)
    }

    fn schema_with_dim(dimension: usize) -> Arc<Schema> {
        let dim_i32 = i32::try_from(dimension).unwrap_or(i32::MAX);
        Arc::new(Schema::new(vec![
            Field::new("memory_id", DataType::Utf8, false),
            Field::new(
                "embedding",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    dim_i32,
                ),
                false,
            ),
        ]))
    }

    fn record_batch(
        memory_id: &str,
        embedding: &[f32],
        dimension: usize,
    ) -> Result<RecordBatch, CortexError> {
        if embedding.len() != dimension {
            return Err(CortexError::GraphError(format!(
                "dimension embedding {} != attendue {dimension}",
                embedding.len()
            )));
        }
        let schema = Self::schema_with_dim(dimension);
        let ids = StringArray::from(vec![memory_id]);
        let values: Vec<Option<f32>> = embedding.iter().map(|v| Some(*v)).collect();
        let list = FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
            std::iter::once(Some(values)),
            i32::try_from(dimension)
                .map_err(|_| CortexError::GraphError("dimension invalide".into()))?,
        );
        RecordBatch::try_new(schema, vec![Arc::new(ids), Arc::new(list)])
            .map_err(|e| CortexError::GraphError(e.to_string()))
    }

    fn scannable_batch(batch: RecordBatch) -> Box<dyn arrow_array::RecordBatchReader + Send> {
        let schema = batch.schema();
        Box::new(RecordBatchIterator::new(vec![Ok(batch)], schema))
    }

    async fn ensure_table(&self) -> Result<(), CortexError> {
        let mut guard = self.table.lock().await;
        if guard.is_some() {
            return Ok(());
        }

        let names = self
            .connection
            .table_names()
            .execute()
            .await
            .map_err(|e| CortexError::GraphError(e.to_string()))?;

        let table = if names.iter().any(|n| n == TABLE_NAME) {
            self.connection
                .open_table(TABLE_NAME)
                .execute()
                .await
                .map_err(|e| CortexError::GraphError(e.to_string()))?
        } else {
            let schema = Self::schema_with_dim(self.dimension);
            let empty = RecordBatch::new_empty(schema.clone());
            self.connection
                .create_table(TABLE_NAME, Self::scannable_batch(empty))
                .execute()
                .await
                .map_err(|e| CortexError::GraphError(e.to_string()))?
        };

        let table_clone = table.clone();
        *guard = Some(table);
        drop(guard);
        self.try_create_vector_index(&table_clone).await;
        Ok(())
    }

    async fn try_create_vector_index(&self, table: &Table) {
        let mut created = self.index_created.lock().await;
        if *created {
            return;
        }
        let index = Index::IvfFlat(
            IvfFlatIndexBuilder::default()
                .num_partitions(1)
                .sample_rate(1),
        );
        match table.create_index(&["embedding"], index).execute().await {
            Ok(()) => {
                *created = true;
                tracing::info!("index vectoriel LanceDB créé à l'initialisation");
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "création index vectoriel à l'init — nouvelle tentative après upsert"
                );
            }
        }
    }

    async fn with_table<F, Fut, T>(&self, f: F) -> Result<T, CortexError>
    where
        F: FnOnce(Table) -> Fut,
        Fut: std::future::Future<Output = Result<T, CortexError>>,
    {
        self.ensure_table().await?;
        let guard = self.table.lock().await;
        let table = guard
            .as_ref()
            .ok_or_else(|| CortexError::GraphError("table LanceDB non initialisée".into()))?
            .clone();
        f(table).await
    }

    fn parse_hits(batch: &RecordBatch, limit: usize) -> Result<Vec<SearchHit>, CortexError> {
        let id_col = batch
            .column_by_name("memory_id")
            .ok_or_else(|| CortexError::GraphError("colonne memory_id absente".into()))?
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| CortexError::GraphError("memory_id invalide".into()))?;

        let dist_col = batch
            .column_by_name("_distance")
            .and_then(|c| c.as_any().downcast_ref::<Float32Array>())
            .cloned();

        let mut hits = Vec::new();
        for i in 0..id_col.len() {
            let id_str = id_col.value(i);
            let memory_id: MemoryId = id_str
                .parse()
                .map_err(|_| CortexError::GraphError(format!("uuid invalide: {id_str}")))?;
            let score = dist_col.as_ref().map_or(1.0, |d| {
                let distance = d.value(i);
                (1.0 - distance).clamp(0.0, 1.0)
            });
            hits.push(SearchHit {
                memory_id,
                score,
                snippet: None,
            });
        }
        hits.truncate(limit);
        Ok(hits)
    }
}

#[async_trait]
impl VectorStore for LancedbVectorStore {
    #[instrument(skip(self, embedding))]
    async fn upsert(&self, memory_id: MemoryId, embedding: &[f32]) -> Result<(), CortexError> {
        let id_str = memory_id.to_string();
        let batch = Self::record_batch(&id_str, embedding, self.dimension)?;
        self.with_table(move |table| async move {
            let _ = table
                .delete(&format!("memory_id = '{id_str}'"))
                .await
                .map_err(|e| CortexError::GraphError(e.to_string()))?;

            table
                .add(Self::scannable_batch(batch))
                .execute()
                .await
                .map_err(|e| CortexError::GraphError(e.to_string()))?;

            Ok(())
        })
        .await?;

        let table = {
            let guard = self.table.lock().await;
            guard.as_ref().cloned()
        };
        if let Some(table) = table {
            self.try_create_vector_index(&table).await;
        }
        Ok(())
    }

    #[instrument(skip(self, query_embedding))]
    async fn semantic_search(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchHit>, CortexError> {
        if query_embedding.len() != self.dimension {
            return Err(CortexError::GraphError(format!(
                "dimension requête {} != {}",
                query_embedding.len(),
                self.dimension
            )));
        }
        let query = query_embedding.to_vec();

        self.with_table(move |table| async move {
            let results = table
                .query()
                .nearest_to(query.as_slice())
                .map_err(|e| CortexError::GraphError(e.to_string()))?
                .limit(limit)
                .execute()
                .await
                .map_err(|e| CortexError::GraphError(e.to_string()))?;

            let batches: Vec<RecordBatch> = results
                .try_collect()
                .await
                .map_err(|e| CortexError::GraphError(e.to_string()))?;

            let mut hits = Vec::new();
            for batch in batches {
                hits.extend(Self::parse_hits(&batch, limit)?);
            }
            hits.sort_by(|a, b| {
                b.score
                    .partial_cmp(&a.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            hits.truncate(limit);
            Ok(hits)
        })
        .await
    }

    async fn hybrid_search(
        &self,
        query_embedding: &[f32],
        filter: &SearchFilter,
    ) -> Result<Vec<SearchHit>, CortexError> {
        let candidate_limit = filter.limit.unwrap_or(256);
        let mut hits = self
            .semantic_search(query_embedding, candidate_limit)
            .await?;
        if let Some(min) = filter.min_score {
            hits.retain(|h| h.score >= min);
        }
        Ok(hits)
    }

    async fn get_embedding(&self, memory_id: MemoryId) -> Result<Option<Vec<f32>>, CortexError> {
        let id_str = memory_id.to_string();
        self.with_table(move |table| async move {
            let results = table
                .query()
                .only_if(format!("memory_id = '{id_str}'"))
                .execute()
                .await
                .map_err(|e| CortexError::GraphError(e.to_string()))?;

            let batches: Vec<RecordBatch> = results
                .try_collect()
                .await
                .map_err(|e| CortexError::GraphError(e.to_string()))?;

            for batch in batches {
                if batch.num_rows() == 0 {
                    continue;
                }
                let list = batch
                    .column_by_name("embedding")
                    .ok_or_else(|| CortexError::GraphError("colonne embedding absente".into()))?
                    .as_any()
                    .downcast_ref::<FixedSizeListArray>()
                    .ok_or_else(|| CortexError::GraphError("embedding invalide".into()))?;
                let row = list.value(0);
                let floats = row
                    .as_any()
                    .downcast_ref::<Float32Array>()
                    .ok_or_else(|| CortexError::GraphError("float32 invalide".into()))?;
                return Ok(Some(floats.values().to_vec()));
            }
            Ok(None)
        })
        .await
    }

    async fn delete(&self, memory_id: MemoryId) -> Result<(), CortexError> {
        let id_str = memory_id.to_string();
        self.with_table(move |table| async move {
            table
                .delete(&format!("memory_id = '{id_str}'"))
                .await
                .map_err(|e| CortexError::GraphError(e.to_string()))?;
            Ok(())
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn lancedb_upsert_and_search() {
        let dir = tempfile::tempdir().unwrap();
        let dim = 4;
        let store = LancedbVectorStore::open(dir.path(), dim).await.unwrap();
        let id = MemoryId::new();
        let vec = vec![1.0, 0.0, 0.0, 0.0];
        store.upsert(id, &vec).await.unwrap();

        let cached = store.get_embedding(id).await.unwrap();
        assert_eq!(cached, Some(vec));

        let query = vec![0.9, 0.1, 0.0, 0.0];
        let hits = store.semantic_search(&query, 5).await.unwrap();
        assert!(!hits.is_empty());
        assert_eq!(hits[0].memory_id, id);
    }
}
