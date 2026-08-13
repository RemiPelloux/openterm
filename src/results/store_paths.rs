use super::AppResult;
use std::fs;
use std::path::{Path, PathBuf};

pub fn data_root() -> AppResult<PathBuf> {
    if let Some(path) = std::env::var_os("OPENTERM_DATA_DIR") {
        return Ok(PathBuf::from(path));
    }
    let home = PathBuf::from(std::env::var_os("HOME").ok_or("HOME is not set")?);
    if cfg!(target_os = "macos") {
        return Ok(home.join("Library/Application Support/OpenTerm"));
    }
    Ok(home.join(".local/share/openterm"))
}

#[cfg(unix)]
pub fn create_private_directory(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true).mode(0o700).create(path)
}

#[cfg(not(unix))]
pub fn create_private_directory(path: &Path) -> std::io::Result<()> {
    fs::create_dir_all(path)
}
