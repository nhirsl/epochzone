use sha2::{Digest, Sha256};
use tokio_rusqlite::Connection;
use uuid::Uuid;

use super::models::{ApiKeyListItem, CreateApiKeyResponse};

pub fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn generate_api_key() -> String {
    format!("ez_{}", Uuid::new_v4().simple())
}

pub async fn create_api_key(
    db: &Connection,
    name: String,
    expires_at: Option<String>,
) -> Result<CreateApiKeyResponse, String> {
    let raw_key = generate_api_key();
    let key_hash = hash_api_key(&raw_key);
    let id = Uuid::new_v4().to_string();

    let name_clone = name.clone();
    let expires_clone = expires_at.clone();
    let id_for_insert = id.clone();
    let id_for_select = id.clone();

    db.call(move |conn| {
        conn.execute(
            "INSERT INTO api_keys (id, key_hash, name, expires_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![id_for_insert, key_hash, name_clone, expires_clone],
        )?;
        Ok(())
    })
    .await
    .map_err(|e| format!("Failed to create API key: {}", e))?;

    let created_at = db
        .call(move |conn| {
            let created: String = conn.query_row(
                "SELECT created_at FROM api_keys WHERE id = ?1",
                rusqlite::params![id_for_select],
                |row| row.get(0),
            )?;
            Ok(created)
        })
        .await
        .map_err(|e| format!("Failed to read created_at: {}", e))?;

    Ok(CreateApiKeyResponse {
        id,
        name,
        api_key: raw_key,
        created_at,
        expires_at,
    })
}

pub async fn list_api_keys(db: &Connection) -> Result<Vec<ApiKeyListItem>, String> {
    db.call(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, name, created_at, is_active, expires_at FROM api_keys ORDER BY created_at DESC",
        )?;
        let keys = stmt
            .query_map([], |row| {
                Ok(ApiKeyListItem {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                    is_active: row.get::<_, i32>(3)? == 1,
                    expires_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(keys)
    })
    .await
    .map_err(|e| format!("Failed to list API keys: {}", e))
}

pub async fn revoke_api_key(db: &Connection, id: String) -> Result<bool, String> {
    db.call(move |conn| {
        let rows_affected =
            conn.execute("UPDATE api_keys SET is_active = 0 WHERE id = ?1", rusqlite::params![id])?;
        Ok(rows_affected > 0)
    })
    .await
    .map_err(|e| format!("Failed to revoke API key: {}", e))
}

pub async fn validate_api_key(db: &Connection, raw_key: &str) -> bool {
    let key_hash = hash_api_key(raw_key);
    db.call(move |conn| {
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM api_keys WHERE key_hash = ?1 AND is_active = 1 AND (expires_at IS NULL OR expires_at > datetime('now'))",
                rusqlite::params![key_hash],
                |row| row.get::<_, i32>(0),
            )
            .map(|count| count > 0)
            .unwrap_or(false);
        Ok(exists)
    })
    .await
    .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_db;

    #[test]
    fn test_hash_determinism() {
        let key = "ez_test123";
        assert_eq!(hash_api_key(key), hash_api_key(key));
    }

    #[test]
    fn test_hash_different_keys() {
        assert_ne!(hash_api_key("key1"), hash_api_key("key2"));
    }

    #[test]
    fn test_generate_api_key_format() {
        let key = generate_api_key();
        assert!(key.starts_with("ez_"));
        assert_eq!(key.len(), 3 + 32); // "ez_" + 32-char hex UUID
    }

    #[test]
    fn test_generate_api_key_unique() {
        let key1 = generate_api_key();
        let key2 = generate_api_key();
        assert_ne!(key1, key2);
    }

    #[tokio::test]
    async fn test_create_and_validate_api_key() {
        let db = init_db(":memory:").await;
        let resp = create_api_key(&db, "test-key".to_string(), None)
            .await
            .unwrap();

        assert!(resp.api_key.starts_with("ez_"));
        assert_eq!(resp.name, "test-key");
        assert!(validate_api_key(&db, &resp.api_key).await);
    }

    #[tokio::test]
    async fn test_validate_invalid_key() {
        let db = init_db(":memory:").await;
        assert!(!validate_api_key(&db, "ez_nonexistent").await);
    }

    #[tokio::test]
    async fn test_list_api_keys() {
        let db = init_db(":memory:").await;
        create_api_key(&db, "key-1".to_string(), None).await.unwrap();
        create_api_key(&db, "key-2".to_string(), None).await.unwrap();

        let keys = list_api_keys(&db).await.unwrap();
        assert_eq!(keys.len(), 2);
    }

    #[tokio::test]
    async fn test_revoke_api_key() {
        let db = init_db(":memory:").await;
        let resp = create_api_key(&db, "revoke-me".to_string(), None)
            .await
            .unwrap();

        assert!(validate_api_key(&db, &resp.api_key).await);

        let revoked = revoke_api_key(&db, resp.id).await.unwrap();
        assert!(revoked);

        assert!(!validate_api_key(&db, &resp.api_key).await);
    }

    #[tokio::test]
    async fn test_revoke_nonexistent_key() {
        let db = init_db(":memory:").await;
        let revoked = revoke_api_key(&db, "nonexistent-id".to_string())
            .await
            .unwrap();
        assert!(!revoked);
    }
}
