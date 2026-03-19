/// Repository management — clone and update Rules repo.

use crate::shared::Error_x;
use std::path::PathBuf;

const REPO_URL: &str = "https://github.com/lpmwfx/Rules.git";

/// Ensure Rules repo is cloned and up-to-date.
/// Returns path to repo cache directory.
pub async fn ensure_repo() -> Result<PathBuf, Error_x> {
    let cache_dir = get_cache_dir();
    let repo_path = cache_dir.join("Rules");

    if !repo_path.join(".git").exists() {
        std::fs::create_dir_all(&cache_dir)?;
        git2::Repository::clone(REPO_URL, &repo_path)?;
    } else {
        // Pull updates from origin/main
        let repo = git2::Repository::open(&repo_path)?;
        let mut remote = repo.find_remote("origin")?;
        remote.fetch(&["main"], None, None)?;

        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let oid = fetch_head.target()
            .expect("FETCH_HEAD has no target");

        repo.set_head_detached(oid)?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
    }

    Ok(repo_path)
}

/// Get cache directory for rules-mcp data.
fn get_cache_dir() -> PathBuf {
    directories::ProjectDirs::from("", "", "rules-mcp")
        .map(|dirs| dirs.cache_dir().to_path_buf())
        .unwrap_or_else(|| std::env::temp_dir().join("rules-mcp"))
}
