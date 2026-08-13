use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

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
    pub ollama: Option<ProviderEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lmstudio: Option<ProviderEntry>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderEntry {
    /// Empty for locally hosted runtimes, which take no credential.
    #[serde(default, skip_serializing_if = "String::is_empty")]
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
    Ok(home_dir()?.join(".config").join("openterm"))
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
    write_private(&path, format!("{json}\n").as_bytes())
}

pub fn save(settings: &ProviderSettings) -> Result<(), String> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    save_to_base(
        path.parent()
            .ok_or_else(|| "invalid config path".to_owned())?,
        settings,
    )
}

pub fn stored_model(provider: Provider) -> Option<String> {
    if provider == Provider::Codex {
        return None;
    }
    entry(provider, &load())?.model.clone()
}

pub fn is_stored(provider: Provider) -> bool {
    if provider == Provider::Codex {
        return false;
    }
    let settings = load();
    let Some(entry) = entry(provider, &settings) else {
        return false;
    };
    // A local runtime is connected once a model is chosen; there is no key.
    if provider.is_local() {
        return entry.model.is_some();
    }
    !entry.api_key.is_empty()
}

pub fn env_var(provider: Provider) -> Option<&'static str> {
    match provider {
        Provider::OpenAi => Some("OPENAI_API_KEY"),
        Provider::Claude => Some("ANTHROPIC_API_KEY"),
        Provider::Mistral => Some("MISTRAL_API_KEY"),
        Provider::Codex | Provider::Ollama | Provider::LmStudio => None,
    }
}

/// Records a connected provider so `ai-ask` can resolve its model later.
pub fn store_entry(settings: &mut ProviderSettings, provider: Provider, entry: ProviderEntry) {
    match provider {
        Provider::OpenAi => settings.openai = Some(entry),
        Provider::Claude => settings.claude = Some(entry),
        Provider::Mistral => settings.mistral = Some(entry),
        Provider::Ollama => settings.ollama = Some(entry),
        Provider::LmStudio => settings.lmstudio = Some(entry),
        Provider::Codex => {}
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
    Err(format!("{provider} does not use an API key"))
}

fn entry(provider: Provider, settings: &ProviderSettings) -> Option<&ProviderEntry> {
    match provider {
        Provider::OpenAi => settings.openai.as_ref(),
        Provider::Claude => settings.claude.as_ref(),
        Provider::Mistral => settings.mistral.as_ref(),
        Provider::Ollama => settings.ollama.as_ref(),
        Provider::LmStudio => settings.lmstudio.as_ref(),
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

fn write_private(path: &Path, contents: &[u8]) -> Result<(), String> {
    let temporary = temporary_path(path)?;
    let result = write_temporary(&temporary, contents)
        .and_then(|()| std::fs::rename(&temporary, path).map_err(|error| error.to_string()));
    if result.is_err() {
        let _ = std::fs::remove_file(temporary);
    }
    result
}

fn write_temporary(path: &Path, contents: &[u8]) -> Result<(), String> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options.open(path).map_err(|error| error.to_string())?;
    file.write_all(contents)
        .map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())
}

fn temporary_path(path: &Path) -> Result<PathBuf, String> {
    let mut bytes = [0_u8; 8];
    getrandom::fill(&mut bytes).map_err(|error| error.to_string())?;
    Ok(path.with_extension(format!("tmp-{}", hex::encode(bytes))))
}

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
        assert_private(&base.join(super::FILE_NAME));
        let _ = std::fs::remove_dir_all(base);
    }

    #[cfg(unix)]
    fn assert_private(path: &std::path::Path) {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(path)
            .expect("metadata")
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[cfg(not(unix))]
    fn assert_private(_path: &std::path::Path) {}

    #[test]
    fn config_dir_uses_home_when_unset() {
        let dir = config_dir().expect("dir");
        assert!(dir.ends_with(".config/openterm") || dir.ends_with("openterm"));
    }

    #[test]
    fn local_provider_roundtrip_has_no_api_key() {
        let base = temp_base().join("local");
        let _ = std::fs::remove_dir_all(&base);
        let mut settings = ProviderSettings::default();
        super::store_entry(
            &mut settings,
            crate::providers::Provider::Ollama,
            super::ProviderEntry {
                api_key: String::new(),
                model: Some("llama3.2".to_owned()),
            },
        );
        save_to_base(&base, &settings).expect("save");
        let loaded = load_from_base(&base);
        let model = loaded
            .ollama
            .as_ref()
            .and_then(|entry| entry.model.as_deref());
        assert_eq!(model, Some("llama3.2"));
        assert!(loaded.ollama.is_some_and(|entry| entry.api_key.is_empty()));
        let text = std::fs::read_to_string(base.join(super::FILE_NAME)).expect("read");
        assert!(!text.contains("api_key"));
        let _ = std::fs::remove_dir_all(base);
    }
}
