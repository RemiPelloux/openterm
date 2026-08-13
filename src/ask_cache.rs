use crate::providers::AskRequest;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const FILE_NAME: &str = "ask-cache.json";
const MAX_ENTRIES: usize = 256;
const TTL_SECS: u64 = 60 * 60 * 24;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct CacheFile {
    #[serde(default)]
    entries: Vec<CacheEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CacheEntry {
    key: String,
    answer: String,
    stored_at: u64,
    accessed_at: u64,
}

pub fn complete(
    request: &AskRequest,
    fetch: impl FnOnce() -> Result<String, String>,
) -> Result<String, String> {
    if let Some(answer) = lookup(request) {
        eprintln!("openterm: ask-cache hit");
        crate::hud::record_cache_hit();
        return Ok(answer);
    }
    let answer = fetch()?;
    store(request, &answer);
    Ok(answer)
}

pub fn lookup(request: &AskRequest) -> Option<String> {
    lookup_in(&cache_path()?, request)
}

pub fn store(request: &AskRequest, answer: &str) {
    let Some(path) = cache_path() else {
        return;
    };
    store_in(&path, request, answer);
}

fn lookup_in(path: &Path, request: &AskRequest) -> Option<String> {
    let key = cache_key(request);
    let now = now_secs();
    let mut file = load(path);
    let index = file.entries.iter().position(|entry| entry.key == key)?;
    if now.saturating_sub(file.entries[index].stored_at) > TTL_SECS {
        file.entries.remove(index);
        save(path, &file);
        return None;
    }
    file.entries[index].accessed_at = now;
    let answer = file.entries[index].answer.clone();
    file.entries[index..].rotate_left(1);
    save(path, &file);
    Some(answer)
}

fn store_in(path: &Path, request: &AskRequest, answer: &str) {
    if answer.trim().is_empty() || answer.contains("```opencode") {
        return;
    }
    let key = cache_key(request);
    let now = now_secs();
    let mut file = load(path);
    file.entries
        .retain(|entry| entry.key != key && now.saturating_sub(entry.stored_at) <= TTL_SECS);
    file.entries.push(CacheEntry {
        key,
        answer: answer.to_owned(),
        stored_at: now,
        accessed_at: now,
    });
    if file.entries.len() > MAX_ENTRIES {
        let drop = file.entries.len() - MAX_ENTRIES;
        file.entries.drain(..drop);
    }
    save(path, &file);
}

fn cache_key(request: &AskRequest) -> String {
    let mut digest = Sha256::new();
    digest.update(request.provider.to_string().as_bytes());
    digest.update([0]);
    digest.update(request.model.as_deref().unwrap_or_default().as_bytes());
    digest.update([0]);
    digest.update(request.prompt.as_bytes());
    hex::encode(digest.finalize())
}

fn cache_path() -> Option<PathBuf> {
    let dir = crate::results::data_root().ok()?;
    crate::results::create_private_directory(&dir).ok()?;
    Some(dir.join(FILE_NAME))
}

fn load(path: &Path) -> CacheFile {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

fn save(path: &Path, file: &CacheFile) {
    let Ok(json) = serde_json::to_string_pretty(file) else {
        return;
    };
    let _ = write_private(path, format!("{json}\n").as_bytes());
}

fn write_private(path: &Path, contents: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        crate::results::create_private_directory(parent).map_err(|error| error.to_string())?;
    }
    std::fs::write(path, contents).map_err(|error| error.to_string())?;
    set_private(path);
    Ok(())
}

fn set_private(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    }
    #[cfg(not(unix))]
    let _ = path;
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::{CacheEntry, CacheFile, load, lookup_in, save, store_in};
    use crate::providers::{AskRequest, Provider};
    use std::path::PathBuf;

    fn request(prompt: &str) -> AskRequest {
        AskRequest {
            provider: Provider::OpenAi,
            model: Some("gpt-test".to_owned()),
            prompt: prompt.to_owned(),
        }
    }

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("openterm-ask-cache-{name}-{}", std::process::id()))
    }

    #[test]
    fn roundtrip_and_provider_isolation() {
        let path = temp_path("roundtrip");
        let _ = std::fs::remove_file(&path);
        store_in(&path, &request("disk?"), "Use df -h.");
        assert_eq!(
            lookup_in(&path, &request("disk?")).as_deref(),
            Some("Use df -h.")
        );
        let other = AskRequest {
            provider: Provider::Claude,
            model: Some("gpt-test".to_owned()),
            prompt: "disk?".to_owned(),
        };
        assert_eq!(lookup_in(&path, &other), None);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn expired_entry_is_a_miss() {
        let path = temp_path("expired");
        let _ = std::fs::remove_file(&path);
        let mut file = CacheFile::default();
        file.entries.push(CacheEntry {
            key: super::cache_key(&request("old")),
            answer: "stale".to_owned(),
            stored_at: 1,
            accessed_at: 1,
        });
        save(&path, &file);
        assert_eq!(lookup_in(&path, &request("old")), None);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn empty_answers_are_not_stored() {
        let path = temp_path("empty");
        let _ = std::fs::remove_file(&path);
        store_in(&path, &request("hi"), "  ");
        assert_eq!(lookup_in(&path, &request("hi")), None);
        let loaded = load(&path);
        assert!(loaded.entries.is_empty());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn opencode_answers_are_not_cached() {
        let path = temp_path("opencode");
        let _ = std::fs::remove_file(&path);
        store_in(
            &path,
            &request("edit zsh"),
            "Plan\n```opencode\nEdit ~/.zshrc\n```\n",
        );
        assert_eq!(lookup_in(&path, &request("edit zsh")), None);
        let _ = std::fs::remove_file(path);
    }
}
