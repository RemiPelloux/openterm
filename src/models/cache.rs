//! Discovered model ids per provider, so pickers never hard-code fast-moving
//! names and never wait on the network while the list is still fresh.

use crate::providers::{Endpoint, Provider};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

const FILE_NAME: &str = "models.json";
const MS_PER_HOUR: u64 = 60 * 60 * 1000;
/// Providers ship new models often, but not hourly.
pub const TTL_MS: u64 = 24 * MS_PER_HOUR;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelCache {
    #[serde(default)]
    pub providers: BTreeMap<String, CachedModels>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CachedModels {
    /// Endpoint the list came from; another gateway makes it stale.
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub fetched_at_ms: u64,
    #[serde(default)]
    pub models: Vec<String>,
}

impl CachedModels {
    pub fn new(endpoint: &Endpoint, models: Vec<String>, fetched_at_ms: u64) -> Self {
        Self {
            base_url: endpoint.base_url.clone(),
            fetched_at_ms,
            models,
        }
    }

    pub fn is_fresh(&self, base_url: &str, now_ms: u64) -> bool {
        self.base_url == base_url && now_ms.saturating_sub(self.fetched_at_ms) < TTL_MS
    }
}

impl ModelCache {
    /// Cached list for `provider`, only if it came from `base_url`.
    pub fn for_endpoint(&self, provider: Provider, base_url: &str) -> Option<&CachedModels> {
        self.providers
            .get(&provider.to_string())
            .filter(|entry| entry.base_url == base_url)
    }

    pub fn insert(&mut self, provider: Provider, entry: CachedModels) {
        self.providers.insert(provider.to_string(), entry);
    }

    pub fn is_stale(&self, provider: Provider, base_url: &str, now_ms: u64) -> bool {
        !self
            .for_endpoint(provider, base_url)
            .is_some_and(|entry| entry.is_fresh(base_url, now_ms))
    }
}

pub fn load() -> ModelCache {
    path().map(|path| load_from(&path)).unwrap_or_default()
}

/// Re-reads the file just before writing so a concurrent refresh of another
/// provider is not lost.
pub fn store(updates: Vec<(Provider, CachedModels)>) -> Result<(), String> {
    let path = path()?;
    let mut cache = load_from(&path);
    for (provider, entry) in updates {
        cache.insert(provider, entry);
    }
    save_to(&path, &cache)
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|elapsed| u64::try_from(elapsed.as_millis()).ok())
        .unwrap_or(0)
}

fn path() -> Result<PathBuf, String> {
    let dir = crate::results::data_root().map_err(|error| error.to_string())?;
    crate::results::create_private_directory(&dir).map_err(|error| error.to_string())?;
    Ok(dir.join(FILE_NAME))
}

fn load_from(path: &Path) -> ModelCache {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// Atomic, owner-only write: a temporary file renamed over the old one.
fn save_to(path: &Path, cache: &ModelCache) -> Result<(), String> {
    let json = serde_json::to_string_pretty(cache).map_err(|error| error.to_string())?;
    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
    let result = write_temporary(&temporary, format!("{json}\n").as_bytes())
        .and_then(|()| std::fs::rename(&temporary, path).map_err(|error| error.to_string()));
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

fn write_temporary(path: &Path, contents: &[u8]) -> Result<(), String> {
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options.open(path).map_err(|error| error.to_string())?;
    file.write_all(contents)
        .map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::{CachedModels, ModelCache, TTL_MS, load_from, save_to};
    use crate::providers::{Compat, Provider};

    const BASE: &str = "https://api.groq.com/openai/v1";
    const FETCHED_AT: u64 = 1_731_300_000_000;

    fn entry() -> CachedModels {
        CachedModels {
            base_url: BASE.to_owned(),
            fetched_at_ms: FETCHED_AT,
            models: vec!["new".to_owned(), "old".to_owned()],
        }
    }

    #[test]
    fn roundtrip_is_private_and_lossless() {
        let dir = std::env::temp_dir().join(format!("openterm-models-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("dir");
        let path = dir.join("models.json");
        let mut cache = ModelCache::default();
        cache.insert(Provider::Compat(Compat::Groq), entry());
        save_to(&path, &cache).expect("save");
        assert_eq!(load_from(&path), cache);
        assert_private(&path);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[cfg(unix)]
    fn assert_private(path: &std::path::Path) {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(path).expect("meta").permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[cfg(not(unix))]
    fn assert_private(_path: &std::path::Path) {}

    #[test]
    fn entries_expire_after_ttl() {
        let mut cache = ModelCache::default();
        let groq = Provider::Compat(Compat::Groq);
        cache.insert(groq, entry());
        assert!(!cache.is_stale(groq, BASE, FETCHED_AT + TTL_MS - 1));
        assert!(cache.is_stale(groq, BASE, FETCHED_AT + TTL_MS));
        assert!(cache.is_stale(Provider::OpenAi, BASE, FETCHED_AT));
    }

    #[test]
    fn another_base_url_is_stale() {
        let mut cache = ModelCache::default();
        let groq = Provider::Compat(Compat::Groq);
        cache.insert(groq, entry());
        assert!(cache.is_stale(groq, "https://proxy.example/v1", FETCHED_AT));
        assert!(
            cache
                .for_endpoint(groq, "https://proxy.example/v1")
                .is_none()
        );
    }

    #[test]
    fn missing_file_loads_empty() {
        let path = std::env::temp_dir().join("openterm-models-missing.json");
        assert_eq!(load_from(&path), ModelCache::default());
    }
}
