//! API key resolution and storage. Order: environment, keychain, then the
//! legacy key in `providers.json`. Keys are read at request time only.

use super::ProviderSettings;
use super::keychain::{KeyStore, Keychain};
use super::settings::env_var;
use crate::providers::Provider;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeySource {
    Env,
    Keychain,
    File,
    None,
}

impl KeySource {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Env => "env",
            Self::Keychain => "keychain",
            Self::File => "file",
            Self::None => "none",
        }
    }
}

/// Where `store_key` put a key. `File` carries why the keychain was skipped.
#[derive(Debug, PartialEq, Eq)]
pub enum Saved {
    Keychain,
    File(String),
}

impl ProviderSettings {
    /// `None` for keyless providers or when no key is configured anywhere.
    pub fn api_key(&self, provider: Provider) -> Option<String> {
        self.resolve_key(provider, &Keychain).map(|(_, key)| key)
    }

    /// Reports where the key comes from without exposing it.
    pub fn key_source(&self, provider: Provider) -> KeySource {
        self.resolve_key(provider, &Keychain)
            .map_or(KeySource::None, |(source, _)| source)
    }

    pub fn resolve_key(
        &self,
        provider: Provider,
        store: &dyn KeyStore,
    ) -> Option<(KeySource, String)> {
        let variable = env_var(provider)?;
        let env = std::env::var(variable).ok();
        let file = self.entry(provider).map_or("", |entry| &entry.api_key);
        first_key(env, || store.get(&provider.to_string()), file)
    }

    /// Keychain first; when it is unavailable the key stays in the 0600 file.
    pub fn store_key(&mut self, provider: Provider, key: String, store: &dyn KeyStore) -> Saved {
        let mut entry = self.entry(provider).cloned().unwrap_or_default();
        let saved = match store.set(&provider.to_string(), &key) {
            Ok(()) => {
                entry.api_key.clear();
                Saved::Keychain
            }
            Err(reason) => {
                entry.api_key = key;
                Saved::File(reason)
            }
        };
        self.set_entry(provider, entry);
        saved
    }

    /// The legacy plaintext key, if one is still in `providers.json`.
    pub fn file_key(&self, provider: Provider) -> Option<String> {
        let key = &self.entry(provider)?.api_key;
        (!key.is_empty()).then(|| key.clone())
    }
}

fn first_key(
    env: Option<String>,
    keychain: impl FnOnce() -> Option<String>,
    file: &str,
) -> Option<(KeySource, String)> {
    if let Some(value) = env.filter(|value| !value.is_empty()) {
        return Some((KeySource::Env, value));
    }
    if let Some(value) = keychain() {
        return Some((KeySource::Keychain, value));
    }
    (!file.is_empty()).then(|| (KeySource::File, file.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::super::keychain::memory::MemoryStore;
    use super::super::{ProviderEntry, ProviderSettings};
    use super::{KeySource, Saved, first_key};
    use crate::providers::{Compat, Provider};

    const CUSTOM: Provider = Provider::Compat(Compat::Custom);

    fn legacy(key: &str) -> ProviderSettings {
        let mut settings = ProviderSettings::default();
        let entry = ProviderEntry {
            api_key: key.to_owned(),
            ..ProviderEntry::default()
        };
        settings.set_entry(CUSTOM, entry);
        settings
    }

    #[test]
    fn env_wins_then_keychain_then_file() {
        let pick = |env: Option<&str>, chain: Option<&str>, file| {
            first_key(env.map(str::to_owned), || chain.map(str::to_owned), file)
        };
        let env = pick(Some("e"), Some("k"), "f");
        assert_eq!(env, Some((KeySource::Env, "e".to_owned())));
        let chain = pick(Some(""), Some("k"), "f");
        assert_eq!(chain, Some((KeySource::Keychain, "k".to_owned())));
        assert_eq!(
            pick(None, None, "f"),
            Some((KeySource::File, "f".to_owned()))
        );
        assert_eq!(pick(None, None, ""), None);
    }

    #[test]
    fn keychain_shadows_legacy_file_key() {
        let settings = legacy("file-key");
        let store = MemoryStore::default();
        let file = settings.resolve_key(CUSTOM, &store);
        assert_eq!(file.map(|(source, _)| source), Some(KeySource::File));
        store
            .items
            .borrow_mut()
            .insert("custom".into(), "chain".into());
        let chain = settings.resolve_key(CUSTOM, &store);
        assert_eq!(chain, Some((KeySource::Keychain, "chain".to_owned())));
    }

    #[test]
    fn store_key_moves_the_secret_out_of_the_file() {
        let mut settings = legacy("old");
        let store = MemoryStore::default();
        assert_eq!(
            settings.store_key(CUSTOM, "new".into(), &store),
            Saved::Keychain
        );
        assert_eq!(settings.file_key(CUSTOM), None);
        assert_eq!(
            store.items.borrow().get("custom").map(String::as_str),
            Some("new")
        );
    }

    #[test]
    fn store_key_falls_back_to_the_file_when_keychain_fails() {
        let mut settings = ProviderSettings::default();
        let store = MemoryStore {
            failing: true,
            ..MemoryStore::default()
        };
        let saved = settings.store_key(CUSTOM, "k".into(), &store);
        assert!(matches!(saved, Saved::File(_)));
        assert_eq!(settings.file_key(CUSTOM).as_deref(), Some("k"));
    }

    #[test]
    fn source_names_match_the_status_contract() {
        let names = [
            KeySource::Env,
            KeySource::Keychain,
            KeySource::File,
            KeySource::None,
        ]
        .map(KeySource::as_str);
        assert_eq!(names, ["env", "keychain", "file", "none"]);
    }
}
