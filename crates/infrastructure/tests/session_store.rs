#![allow(clippy::unwrap_used, clippy::expect_used)]

use cortex::{ConversationTurn, SessionKey, SessionRepository, TurnRole};
use infrastructure::SqliteSessionStore;
use tempfile::tempdir;

#[tokio::test]
async fn sqlite_session_roundtrip() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("sessions.db");
    let store = SqliteSessionStore::open(&db).unwrap();
    let key = SessionKey::default_chat();

    store
        .append_turn(&key, ConversationTurn::new(TurnRole::User, "hello"))
        .await
        .unwrap();
    store
        .append_turn(
            &key,
            ConversationTurn::new(TurnRole::Assistant, "world"),
        )
        .await
        .unwrap();

    let turns = store.list_turns(&key).await.unwrap();
    assert_eq!(turns.len(), 2);
    assert_eq!(turns[0].content, "hello");
    assert_eq!(turns[1].content, "world");
}