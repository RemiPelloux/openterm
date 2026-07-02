use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::Provider;

const FILE_NAME: &str = "providers.json";

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderSettings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub openai: Option<ProviderEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claude: Option<ProviderEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mistral: Option<ProviderEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex: Option<CodexEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexEntry {
    pub access_token: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub source: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderEntry {
    pub api_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

pub fn config_dir() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("OPENTERM_CONFIG_DIR") {
        return Ok(PathBuf::from(path));
    }
    if let Some(path) = std::env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(path).join("openterm"));
    }
    home_dir()?.join(".config").join("openterm").pipe(Ok)
}

pub fn config_path() -> Result<PathBuf, String> {
    Ok(config_dir()?.join(FILE_NAME))
}

pub fn load() -> ProviderSettings {
    let Ok(path) = config_path() else {
        return ProviderSettings::default();
    };
    load_from(&path).unwrap_or_default()
}

#[cfg(test)]
pub fn load_from_base(base: &Path) -> ProviderSettings {
    load_from(&base.join(FILE_NAME)).unwrap_or_default()
}

pub fn save_to_base(base: &Path, settings: &ProviderSettings) -> Result<(), String> {
    std::fs::create_dir_all(base).map_err(|error| error.to_string())?;
    let path = base.join(FILE_NAME);
    let json = serde_json::to_string_pretty(settings).map_err(|error| error.to_string())?;
    std::fs::write(&path, format!("{json}\n")).map_err(|error| error.to_string())?;
    restrict_permissions(&path);
    Ok(())
}

pub fn save(settings: &ProviderSettings) -> Result<(), String> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    save_to_base(
        path.parent().ok_or_else(|| "invalid config path".to_owned())?,
        settings,
    )
}

pub fn stored_model(provider: Provider) -> Option<String> {
    if provider == Provider::Codex {
        return load().codex.as_ref().and_then(|entry| entry.model.clone());
    }
    entry(provider, &load())?.model.clone()
}

pub fn codex_connected() -> bool {
    load()
        .codex
        .as_ref()
        .is_some_and(|entry| !entry.access_token.is_empty())
}

pub fn is_stored(provider: Provider) -> bool {
    if provider == Provider::Codex {
        return codex_connected();
    }
    entry(provider, &load()).is_some_and(|entry| !entry.api_key.is_empty())
}

pub fn env_var(provider: Provider) -> Option<&'static str> {
    match provider {
        Provider::OpenAi => Some("OPENAI_API_KEY"),
        Provider::Claude => Some("ANTHROPIC_API_KEY"),
        Provider::Mistral => Some("MISTRAL_API_KEY"),
        Provider::Codex => None,
    }
}

pub fn resolve_api_key(provider: Provider) -> Result<String, String> {
    if let Some(variable) = env_var(provider) {
        if let Ok(value) = std::env::var(variable) {
            return Ok(value);
        }
        let settings = load();
        let entry = entry(provider, &settings)
            .ok_or_else(|| format!("{variable} is not configured (run ai-connect)"))?;
        if entry.api_key.is_empty() {
            return Err(format!("{variable} is not configured (run ai-connect)"));
        }
        return Ok(entry.api_key.clone());
    }
    Err("codex does not use an API key; run codex login".to_owned())
}

fn entry(provider: Provider, settings: &ProviderSettings) -> Option<&ProviderEntry> {
    match provider {
        Provider::OpenAi => settings.openai.as_ref(),
        Provider::Claude => settings.claude.as_ref(),
        Provider::Mistral => settings.mistral.as_ref(),
        Provider::Codex => None,
    }
}

fn load_from(path: &Path) -> Result<ProviderSettings, String> {
    let text = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str(&text).map_err(|error| error.to_string())
}

fn home_dir() -> Result<PathBuf, String> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME is not set".to_owned())
}

fn restrict_permissions(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(path) {
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o600);
            let _ = std::fs::set_permissions(path, permissions);
        }
    }
}

trait Pipe: Sized {
    fn pipe<F, R>(self, function: F) -> R
    where
        F: FnOnce(Self) -> R,
    {
        function(self)
    }
}

impl<T> Pipe for T {}

#[cfg(test)]
mod tests {
    use super::{ProviderSettings, config_dir, load_from_base, save_to_base};
    use std::path::PathBuf;

    fn temp_base() -> PathBuf {
        std::env::temp_dir().join(format!("openterm-config-test-{}", std::process::id()))
    }

    #[test]
    fn save_and_load_roundtrip() {
        let base = temp_base();
        let _ = std::fs::remove_dir_all(&base);
        let settings = ProviderSettings {
            default_provider: Some("openai".to_owned()),
            openai: Some(super::ProviderEntry {
                api_key: "sk-test".to_owned(),
                model: Some("gpt-4o-mini".to_owned()),
            }),
            ..ProviderSettings::default()
        };
        save_to_base(&base, &settings).expect("save");
        let loaded = load_from_base(&base);
        assert_eq!(loaded, settings);
        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn config_dir_uses_home_when_unset() {
        let dir = config_dir().expect("dir");
        assert!(dir.ends_with(".config/openterm") || dir.ends_with("openterm"));
    }
}
