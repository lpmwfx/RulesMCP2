/// Repository management — clone and update Rules repo.

use crate::shared::Error_x;
use std::path::PathBuf;

const REPO_URL: &str = "https://github.com/lpmwfx/Rules.git";

/// Ensure Rules repo is cloned.
/// Returns path to repo cache directory.
pub async fn ensure_repo() -> Result<PathBuf, Error_x> {
    let cache_dir = get_cache_dir();
    let repo_path = cache_dir.join("Rules");

    if !repo_path.join(".git").exists() {
        std::fs::create_dir_all(&cache_dir)?;
        git2::Repository::clone(REPO_URL, &repo_path)?;
    }

    Ok(repo_path)
}

/// Get cache directory for rules-mcp data.
fn get_cache_dir() -> PathBuf {
    directories::ProjectDirs::from("", "", "rules-mcp")
        .map(|dirs| dirs.cache_dir().to_path_buf())
        .unwrap_or_else(|| std::env::temp_dir().join("rules-mcp"))
}
