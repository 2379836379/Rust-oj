use rusqlite::Connection;
use std::path::PathBuf;

use crate::storage;

pub fn open_cache_db(file_name: &str) -> Result<(Connection, PathBuf), String> {
    let dir = storage::cache_dir()?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("create_dir_all {}: {e}", dir.display()))?;
    let path = dir.join(file_name);

    let conn = Connection::open(&path).map_err(|e| format!("sqlite open {}: {e}", path.display()))?;
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| format!("sqlite pragma journal_mode: {e}"))?;
    Ok((conn, path))
}

pub fn cache_file_size_bytes(path: &PathBuf) -> i64 {
    let mut total: i64 = 0;
    for suffix in ["", "-wal", "-shm"] {
        let p = if suffix.is_empty() {
            path.clone()
        } else {
            PathBuf::from(format!("{}{}", path.display(), suffix))
        };
        if let Ok(meta) = std::fs::metadata(&p) {
            total += meta.len() as i64;
        }
    }
    total
}

pub fn remove_cache_file(path: &PathBuf) -> Result<(), String> {
    for suffix in ["", "-wal", "-shm"] {
        let p = if suffix.is_empty() {
            path.clone()
        } else {
            PathBuf::from(format!("{}{}", path.display(), suffix))
        };
        if p.exists() {
            std::fs::remove_file(&p).map_err(|e| format!("remove_file {}: {e}", p.display()))?;
        }
    }
    Ok(())
}


