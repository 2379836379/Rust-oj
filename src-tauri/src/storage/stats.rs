use serde::Serialize;
use std::path::{Path, PathBuf};

use super::paths;

#[derive(Debug, Clone, Serialize)]
pub struct StorageSizes {
    pub cache_bytes: i64,
    pub app_bytes: i64,
    pub cache_dir: String,
    pub project_root_dir: String,
}

fn sqlite_file_size_bytes(base: &Path) -> i64 {
    let mut total: i64 = 0;

    let mut add = |p: PathBuf| {
        if let Ok(meta) = std::fs::metadata(p) {
            total += meta.len() as i64;
        }
    };

    add(base.to_path_buf());
    add(PathBuf::from(format!("{}-wal", base.display())));
    add(PathBuf::from(format!("{}-shm", base.display())));

    total
}

fn remove_sqlite_files(base: &Path) -> Result<(), String> {
    for suffix in ["", "-wal", "-shm"] {
        let p = if suffix.is_empty() {
            base.to_path_buf()
        } else {
            PathBuf::from(format!("{}{}", base.display(), suffix))
        };

        if p.exists() {
            std::fs::remove_file(&p).map_err(|e| format!("remove_file {}: {e}", p.display()))?;
        }
    }

    Ok(())
}

fn directory_size_bytes(root: &Path) -> Result<i64, String> {
    if !root.exists() {
        return Ok(0);
    }

    let mut total: i64 = 0;
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let rd = std::fs::read_dir(&dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))?;
        for ent in rd {
            let ent = ent.map_err(|e| format!("read_dir entry {}: {e}", dir.display()))?;
            let ft = ent
                .file_type()
                .map_err(|e| format!("file_type {}: {e}", ent.path().display()))?;

            if ft.is_symlink() {
                continue;
            }
            if ft.is_dir() {
                stack.push(ent.path());
                continue;
            }
            if ft.is_file() {
                let meta = ent
                    .metadata()
                    .map_err(|e| format!("metadata {}: {e}", ent.path().display()))?;
                total += meta.len() as i64;
            }
        }
    }

    Ok(total)
}

pub fn get_storage_sizes() -> Result<StorageSizes, String> {
    let cache_dir = paths::cache_dir()?;
    let root_dir = paths::project_root_dir()?;

    let class_db = cache_dir.join("class_cache.db");
    let contest_db = cache_dir.join("contest_cache.db");
    let problem_db = cache_dir.join("problem_cache.db");

    let cache_bytes = sqlite_file_size_bytes(&class_db)
        + sqlite_file_size_bytes(&contest_db)
        + sqlite_file_size_bytes(&problem_db);

    let app_bytes = directory_size_bytes(&root_dir)?;

    Ok(StorageSizes {
        cache_bytes,
        app_bytes,
        cache_dir: cache_dir.display().to_string(),
        project_root_dir: root_dir.display().to_string(),
    })
}

pub fn clear_all_caches() -> Result<StorageSizes, String> {
    let cache_dir = paths::cache_dir()?;

    remove_sqlite_files(&cache_dir.join("class_cache.db"))?;
    remove_sqlite_files(&cache_dir.join("contest_cache.db"))?;
    remove_sqlite_files(&cache_dir.join("problem_cache.db"))?;

    get_storage_sizes()
}