use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::paths;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRecord {
    pub email: String,
    pub password: String,
    pub updated_at: i64,
}

pub struct LoginCache;

impl LoginCache {
    fn db_path() -> Result<PathBuf, String> {
        Ok(paths::data_dir()?.join("login_cache.db"))
    }

    fn open() -> Result<Connection, String> {
        let path = Self::db_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("create_dir_all {}: {e}", parent.display()))?;
        }

        let conn = Connection::open(path).map_err(|e| format!("sqlite open: {e}"))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS login_cache (\
               email TEXT PRIMARY KEY,\
               password TEXT NOT NULL,\
               updated_at INTEGER NOT NULL\
             )",
            [],
        )
        .map_err(|e| format!("sqlite create table: {e}"))?;

        Ok(conn)
    }

    pub fn save_login(email: &str, password: &str) -> Result<(), String> {
        let email = email.trim();
        if email.is_empty() {
            return Err("Email cannot be empty".to_string());
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| format!("time: {e}"))?
            .as_secs() as i64;

        let conn = Self::open()?;
        conn.execute(
            "INSERT INTO login_cache(email, password, updated_at) VALUES (?1, ?2, ?3)\
             ON CONFLICT(email) DO UPDATE SET password = excluded.password, updated_at = excluded.updated_at",
            params![email, password, now],
        )
        .map_err(|e| format!("sqlite upsert: {e}"))?;

        Ok(())
    }

    pub fn load_last_login() -> Result<Option<LoginRecord>, String> {
        let conn = Self::open()?;
        let mut stmt = conn
            .prepare(
                "SELECT email, password, updated_at FROM login_cache ORDER BY updated_at DESC, email ASC LIMIT 1",
            )
            .map_err(|e| format!("sqlite prepare: {e}"))?;

        let mut rows = stmt.query([]).map_err(|e| format!("sqlite query: {e}"))?;
        let Some(row) = rows.next().map_err(|e| format!("sqlite next: {e}"))? else {
            return Ok(None);
        };

        Ok(Some(LoginRecord {
            email: row.get(0).map_err(|e| format!("sqlite get: {e}"))?,
            password: row.get(1).map_err(|e| format!("sqlite get: {e}"))?,
            updated_at: row.get(2).map_err(|e| format!("sqlite get: {e}"))?,
        }))
    }

    pub fn load_by_email(email: &str) -> Result<Option<LoginRecord>, String> {
        let email = email.trim();
        if email.is_empty() {
            return Ok(None);
        }

        let conn = Self::open()?;
        let mut stmt = conn
            .prepare("SELECT email, password, updated_at FROM login_cache WHERE email = ?1")
            .map_err(|e| format!("sqlite prepare: {e}"))?;

        let mut rows = stmt
            .query(params![email])
            .map_err(|e| format!("sqlite query: {e}"))?;

        let Some(row) = rows.next().map_err(|e| format!("sqlite next: {e}"))? else {
            return Ok(None);
        };

        Ok(Some(LoginRecord {
            email: row.get(0).map_err(|e| format!("sqlite get: {e}"))?,
            password: row.get(1).map_err(|e| format!("sqlite get: {e}"))?,
            updated_at: row.get(2).map_err(|e| format!("sqlite get: {e}"))?,
        }))
    }
}