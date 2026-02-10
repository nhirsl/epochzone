use tokio_rusqlite::Connection;

pub async fn init_db(database_url: &str) -> Connection {
    let conn = if database_url == ":memory:" {
        Connection::open_in_memory().await.expect("Failed to open in-memory database")
    } else {
        Connection::open(database_url).await.expect("Failed to open database")
    };

    conn.call(|conn| {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS api_keys (
                id TEXT PRIMARY KEY,
                key_hash TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                is_active INTEGER NOT NULL DEFAULT 1,
                expires_at TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_api_keys_key_hash ON api_keys (key_hash);
            CREATE INDEX IF NOT EXISTS idx_api_keys_is_active ON api_keys (is_active);",
        )?;
        Ok(())
    })
    .await
    .expect("Failed to initialize database schema");

    conn
}
