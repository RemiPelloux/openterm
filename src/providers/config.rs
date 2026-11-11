use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

mod credentials;
pub mod keychain;
mod settings;

pub use credentials::{KeySource, Saved};
pub use settings::env_var;

const FILE_NAME: &str = "providers.json";

/// On-disk provider settings. Each provider's entry sits under its own name
/// (`"openai"`, `"groq"`, ...), which keeps older files readable unchanged.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderSettings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_provider: Option<String>,
    #[serde(flatten)]
    pub entries: BTreeMap<String, ProviderEntry>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderEntry {
    /// Empty for locally hosted runtimes, which take no credential.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub api_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// OpenAI-compatible gateway origin, without a trailing `/v1`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
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

    fn temp_base(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "openterm-config-test-{name}-{}",
            std::process::id()
        ))
    }

    #[test]
    fn save_and_load_roundtrip() {
        let base = temp_base("roundtrip");
        let _ = std::fs::remove_dir_all(&base);
        let mut settings = ProviderSettings {
            default_provider: Some("openai".to_owned()),
            ..ProviderSettings::default()
        };
        settings.set_entry(
            crate::providers::Provider::OpenAi,
            super::ProviderEntry {
                api_key: "sk-test".to_owned(),
                model: Some("gpt-test".to_owned()),
                base_url: Some("https://llm.example.com".to_owned()),
            },
        );
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
    fn legacy_file_layout_still_loads() {
        let text = r#"{"default_provider":"claude","claude":{"api_key":"k","model":"m"}}"#;
        let settings: ProviderSettings = serde_json::from_str(text).expect("parse");
        assert_eq!(
            settings.model(crate::providers::Provider::Claude),
            Some("m")
        );
    }

    #[test]
    fn config_dir_uses_home_when_unset() {
        let dir = config_dir().expect("dir");
        assert!(dir.ends_with(".config/openterm") || dir.ends_with("openterm"));
    }

    #[test]
    fn local_provider_roundtrip_has_no_api_key() {
        let base = temp_base("local");
        let _ = std::fs::remove_dir_all(&base);
        let mut settings = ProviderSettings::default();
        settings.set_entry(
            crate::providers::Provider::Ollama,
            super::ProviderEntry {
                api_key: String::new(),
                model: Some("llama3.2".to_owned()),
                base_url: None,
            },
        );
        save_to_base(&base, &settings).expect("save");
        let loaded = load_from_base(&base);
        let provider = crate::providers::Provider::Ollama;
        assert_eq!(loaded.model(provider), Some("llama3.2"));
        assert!(
            loaded
                .entry(provider)
                .is_some_and(|entry| entry.api_key.is_empty())
        );
        let text = std::fs::read_to_string(base.join(super::FILE_NAME)).expect("read");
        assert!(!text.contains("api_key"));
        let _ = std::fs::remove_dir_all(base);
    }
}
