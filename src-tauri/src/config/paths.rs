use directories::ProjectDirs;
use std::path::{Path, PathBuf};

const APP_QUALIFIER: &str = "com";
const APP_ORG: &str = "openjudge";
const APP_NAME: &str = "oj-client";

pub fn config_dir() -> Result<PathBuf, String> {
    let dirs = ProjectDirs::from(APP_QUALIFIER, APP_ORG, APP_NAME)
        .ok_or_else(|| "Failed to resolve user config directory".to_string())?;
    Ok(dirs.config_dir().to_path_buf())
}

pub fn exe_dir() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    exe.parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| "Failed to resolve executable directory".to_string())
}

pub fn cwd() -> Result<PathBuf, String> {
    std::env::current_dir().map_err(|e| format!("current_dir: {e}"))
}

pub fn legacy_candidate_paths(file_name: &str) -> Result<Vec<PathBuf>, String> {
    let cwd = cwd()?;
    let exe_dir = exe_dir()?;

    Ok(vec![
        cwd.join(file_name),
        exe_dir.join(file_name),
        exe_dir.join("..").join(file_name),
        exe_dir.join("..").join("oj-client").join(file_name),
        exe_dir.join("..").join("..").join("oj-client").join(file_name),
    ])
}

pub fn find_first_existing(paths: &[PathBuf]) -> Option<PathBuf> {
    paths.iter().find(|p| p.is_file()).cloned()
}

pub fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("Invalid path (no parent): {}", path.display()))?;
    std::fs::create_dir_all(parent).map_err(|e| format!("create_dir_all {}: {e}", parent.display()))
}

