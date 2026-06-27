use std::path::PathBuf;

pub fn project_root_dir() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    let mut dir = exe
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| "Failed to resolve executable directory".to_string())?;

    let name = dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if name == "build" {
        if let Some(parent) = dir.parent() {
            dir = parent.to_path_buf();
        }
        return Ok(dir);
    }

    if name == "debug" || name == "release" {
        if let Some(parent) = dir.parent() {
            let parent_name = parent
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if parent_name == "build" {
                if let Some(grand) = parent.parent() {
                    dir = grand.to_path_buf();
                }
            }
        }
    }

    Ok(dir)
}

pub fn data_dir() -> Result<PathBuf, String> {
    Ok(project_root_dir()?.join("data"))
}

pub fn cache_dir() -> Result<PathBuf, String> {
    Ok(project_root_dir()?.join("cache"))
}