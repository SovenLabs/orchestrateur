use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cortex::{
    ConversationTurn, CortexError, Session, SessionKey, SessionRepository, SessionSummary,
    SessionTurnHit, TurnRole,
};
use rusqlite::{params, Connection};
use tokio::task::spawn_blocking;

/// Persistance SQLite des sessions agent (FTS optionnel Phase 8).
pub struct SqliteSessionStore {
    db_path: Arc<Path>,
}

impl SqliteSessionStore {
    /// Ouvre ou crée la base à `db_path`.
    ///
    /// # Errors
    ///
    /// Retourne [`CortexError::GraphError`] si l'initialisation échoue.
    pub fn open(db_path: impl AsRef<Path>) -> Result<Self, CortexError> {
        let db_path = db_path.as_ref().to_path_buf();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| CortexError::GraphError(e.to_string()))?;
        }
        let store = Self {
            db_path: Arc::from(db_path.as_path()),
        };
        store.init_schema()?;
        Ok(store)
    }

    fn init_schema(&self) -> Result<(), CortexError> {
        let conn = Connection::open(self.db_path.as_ref())
            .map_err(|e| CortexError::GraphError(e.to_string()))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sessions (
                key TEXT PRIMARY KEY,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS turns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_key TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY(session_key) REFERENCES sessions(key) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_turns_session ON turns(session_key, id);",
        )
        .map_err(|e| CortexError::GraphError(e.to_string()))?;
        Ok(())
    }

    async fn with_conn<F, T>(&self, f: F) -> Result<T, CortexError>
    where
        F: FnOnce(&Connection) -> Result<T, CortexError> + Send + 'static,
        T: Send + 'static,
    {
        let path = self.db_path.clone();
        spawn_blocking(move || {
            let conn = Connection::open(path.as_ref())
                .map_err(|e| CortexError::GraphError(e.to_string()))?;
            f(&conn)
        })
        .await
        .map_err(|e| CortexError::GraphError(e.to_string()))?
    }
}

fn role_to_str(role: TurnRole) -> &'static str {
    match role {
        TurnRole::User => "user",
        TurnRole::Assistant => "assistant",
        TurnRole::Tool => "tool",
        TurnRole::System => "system",
    }
}

fn str_to_role(s: &str) -> TurnRole {
    match s {
        "assistant" => TurnRole::Assistant,
        "tool" => TurnRole::Tool,
        "system" => TurnRole::System,
        _ => TurnRole::User,
    }
}

#[async_trait]
impl SessionRepository for SqliteSessionStore {
    async fn get_or_create(&self, key: &SessionKey) -> Result<Session, CortexError> {
        let key_str = key.as_str().to_string();
        self.with_conn(move |conn| {
            let now = Utc::now().to_rfc3339();
            conn.execute(
                "INSERT OR IGNORE INTO sessions (key, created_at, updated_at) VALUES (?1, ?2, ?2)",
                params![key_str, now],
            )
            .map_err(|e| CortexError::GraphError(e.to_string()))?;
            load_session(conn, &key_str)
        })
        .await
    }

    async fn append_turn(
        &self,
        key: &SessionKey,
        turn: ConversationTurn,
    ) -> Result<Session, CortexError> {
        let key_str = key.as_str().to_string();
        let role = role_to_str(turn.role).to_string();
        let content = turn.content;
        let created = turn.created_at.to_rfc3339();
        self.with_conn(move |conn| {
            let now = Utc::now().to_rfc3339();
            conn.execute(
                "INSERT OR IGNORE INTO sessions (key, created_at, updated_at) VALUES (?1, ?2, ?2)",
                params![key_str, now],
            )
            .map_err(|e| CortexError::GraphError(e.to_string()))?;
            conn.execute(
                "INSERT INTO turns (session_key, role, content, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![key_str, role, content, created],
            )
            .map_err(|e| CortexError::GraphError(e.to_string()))?;
            conn.execute(
                "UPDATE sessions SET updated_at = ?1 WHERE key = ?2",
                params![Utc::now().to_rfc3339(), key_str],
            )
            .map_err(|e| CortexError::GraphError(e.to_string()))?;
            load_session(conn, &key_str)
        })
        .await
    }

    async fn list_turns(&self, key: &SessionKey) -> Result<Vec<ConversationTurn>, CortexError> {
        let key_str = key.as_str().to_string();
        self.with_conn(move |conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT role, content, created_at FROM turns WHERE session_key = ?1 ORDER BY id ASC",
                )
                .map_err(|e| CortexError::GraphError(e.to_string()))?;
            let rows = stmt
                .query_map(params![key_str], |row| {
                    let role: String = row.get(0)?;
                    let content: String = row.get(1)?;
                    let created_at: String = row.get(2)?;
                    Ok((role, content, created_at))
                })
                .map_err(|e| CortexError::GraphError(e.to_string()))?;
            let mut turns = Vec::new();
            for row in rows {
                let (role, content, created_at) =
                    row.map_err(|e| CortexError::GraphError(e.to_string()))?;
                let created_at = DateTime::parse_from_rfc3339(&created_at)
                    .map_err(|e| CortexError::GraphError(e.to_string()))?
                    .with_timezone(&Utc);
                turns.push(ConversationTurn {
                    role: str_to_role(&role),
                    content,
                    created_at,
                });
            }
            Ok(turns)
        })
        .await
    }

    async fn delete(&self, key: &SessionKey) -> Result<(), CortexError> {
        let key_str = key.as_str().to_string();
        self.with_conn(move |conn| {
            conn.execute(
                "DELETE FROM turns WHERE session_key = ?1",
                params![key_str],
            )
            .map_err(|e| CortexError::GraphError(e.to_string()))?;
            conn.execute("DELETE FROM sessions WHERE key = ?1", params![key_str])
                .map_err(|e| CortexError::GraphError(e.to_string()))?;
            Ok(())
        })
        .await
    }

    async fn list_recent_sessions(&self, limit: usize) -> Result<Vec<SessionSummary>, CortexError> {
        let cap = limit.max(1).min(100);
        self.with_conn(move |conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT s.key, s.updated_at,
                        (SELECT COUNT(*) FROM turns t WHERE t.session_key = s.key),
                        (SELECT content FROM turns t WHERE t.session_key = s.key
                         AND t.role IN ('user','assistant')
                         ORDER BY t.id DESC LIMIT 1)
                     FROM sessions s
                     ORDER BY s.updated_at DESC
                     LIMIT ?1",
                )
                .map_err(|e| CortexError::GraphError(e.to_string()))?;
            let rows = stmt
                .query_map(params![cap as i64], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, Option<String>>(3)?,
                    ))
                })
                .map_err(|e| CortexError::GraphError(e.to_string()))?;
            let mut out = Vec::new();
            for row in rows {
                let (key, updated, count, preview) =
                    row.map_err(|e| CortexError::GraphError(e.to_string()))?;
                let updated_at = DateTime::parse_from_rfc3339(&updated)
                    .map_err(|e| CortexError::GraphError(e.to_string()))?
                    .with_timezone(&Utc);
                let key = SessionKey::new(key).map_err(|e| CortexError::GraphError(e.to_string()))?;
                let preview = preview.unwrap_or_default();
                let preview: String = preview.chars().take(200).collect();
                out.push(SessionSummary {
                    key,
                    turn_count: count as usize,
                    updated_at,
                    preview,
                });
            }
            Ok(out)
        })
        .await
    }

    async fn search_turns(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SessionTurnHit>, CortexError> {
        let q = query.trim().to_string();
        if q.is_empty() {
            return Ok(Vec::new());
        }
        let cap = limit.max(1).min(50);
        let pattern = format!("%{q}%");
        self.with_conn(move |conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT session_key, id, role, content
                     FROM turns
                     WHERE content LIKE ?1 ESCAPE '\\'
                     ORDER BY id DESC
                     LIMIT ?2",
                )
                .map_err(|e| CortexError::GraphError(e.to_string()))?;
            let rows = stmt
                .query_map(params![pattern, cap as i64], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                })
                .map_err(|e| CortexError::GraphError(e.to_string()))?;
            let mut out = Vec::new();
            for row in rows {
                let (key, id, role, content) =
                    row.map_err(|e| CortexError::GraphError(e.to_string()))?;
                let key = SessionKey::new(key).map_err(|e| CortexError::GraphError(e.to_string()))?;
                let snippet: String = content.chars().take(300).collect();
                out.push(SessionTurnHit {
                    key,
                    turn_index: id.saturating_sub(1) as usize,
                    role: str_to_role(&role),
                    snippet,
                });
            }
            Ok(out)
        })
        .await
    }
}

fn load_session(conn: &Connection, key: &str) -> Result<Session, CortexError> {
    let mut stmt = conn
        .prepare("SELECT created_at, updated_at FROM sessions WHERE key = ?1")
        .map_err(|e| CortexError::GraphError(e.to_string()))?;
    let (created, updated): (String, String) = stmt
        .query_row(params![key], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| CortexError::GraphError(e.to_string()))?;
    let created_at = DateTime::parse_from_rfc3339(&created)
        .map_err(|e| CortexError::GraphError(e.to_string()))?
        .with_timezone(&Utc);
    let updated_at = DateTime::parse_from_rfc3339(&updated)
        .map_err(|e| CortexError::GraphError(e.to_string()))?
        .with_timezone(&Utc);

    let mut tstmt = conn
        .prepare(
            "SELECT role, content, created_at FROM turns WHERE session_key = ?1 ORDER BY id ASC",
        )
        .map_err(|e| CortexError::GraphError(e.to_string()))?;
    let rows = tstmt
        .query_map(params![key], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| CortexError::GraphError(e.to_string()))?;

    let mut turns = Vec::new();
    for row in rows {
        let (role, content, created_at) =
            row.map_err(|e| CortexError::GraphError(e.to_string()))?;
        let created_at = DateTime::parse_from_rfc3339(&created_at)
            .map_err(|e| CortexError::GraphError(e.to_string()))?
            .with_timezone(&Utc);
        turns.push(ConversationTurn {
            role: str_to_role(&role),
            content,
            created_at,
        });
    }

    Ok(Session {
        key: SessionKey::new(key).map_err(|e| CortexError::GraphError(e.to_string()))?,
        turns,
        created_at,
        updated_at,
    })
}