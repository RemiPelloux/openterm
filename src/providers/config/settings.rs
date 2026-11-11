//! Lookups over one loaded `ProviderSettings`. Load the file once per command
//! and pass it down; every accessor here is a pure read.

use super::{ProviderEntry, ProviderSettings};
use crate::providers::Provider;

const DEFAULT_PROVIDER: Provider = Provider::Codex;

impl ProviderSettings {
    pub fn entry(&self, provider: Provider) -> Option<&ProviderEntry> {
        self.entries.get(&provider.to_string())
    }

    pub fn set_entry(&mut self, provider: Provider, entry: ProviderEntry) {
        self.entries.insert(provider.to_string(), entry);
    }

    pub fn model(&self, provider: Provider) -> Option<&str> {
        self.entry(provider)?.model.as_deref()
    }

    pub fn base_url(&self, provider: Provider) -> Option<&str> {
        self.entry(provider)?.base_url.as_deref()
    }

    pub fn default_provider(&self) -> Provider {
        parse_default(self.default_provider.as_deref())
    }

    /// Local runtimes are connected once a model is chosen; they take no key.
    pub fn is_stored(&self, provider: Provider) -> bool {
        provider.is_local() && self.model(provider).is_some()
    }
}

fn parse_default(name: Option<&str>) -> Provider {
    name.and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_PROVIDER)
}

pub fn env_var(provider: Provider) -> Option<&'static str> {
    match provider {
        Provider::OpenAi => Some("OPENAI_API_KEY"),
        Provider::Claude => Some("ANTHROPIC_API_KEY"),
        Provider::Mistral => Some("MISTRAL_API_KEY"),
        Provider::Compat(preset) => Some(preset.env_key()),
        Provider::Codex | Provider::ClaudeCode | Provider::Ollama | Provider::LmStudio => None,
    }
}

#[cfg(test)]
mod tests {
    use super::super::{ProviderEntry, ProviderSettings};
    use crate::providers::{Compat, Provider};

    fn with_entry(provider: Provider, entry: ProviderEntry) -> ProviderSettings {
        let mut settings = ProviderSettings::default();
        settings.set_entry(provider, entry);
        settings
    }

    #[test]
    fn stored_key_and_model_are_read_from_one_load() {
        let provider = Provider::Compat(Compat::Custom);
        let settings = with_entry(
            provider,
            ProviderEntry {
                api_key: "k".to_owned(),
                model: Some("m".to_owned()),
                base_url: Some("https://x.example/v1".to_owned()),
            },
        );
        assert_eq!(settings.file_key(provider).as_deref(), Some("k"));
        assert_eq!(settings.model(provider), Some("m"));
        assert_eq!(settings.base_url(provider), Some("https://x.example/v1"));
        assert_eq!(settings.file_key(Provider::Compat(Compat::Groq)), None);
        assert!(!settings.is_stored(provider));
    }

    #[test]
    fn keyless_providers_have_no_api_key() {
        let settings = ProviderSettings::default();
        assert_eq!(settings.api_key(Provider::Ollama), None);
        assert_eq!(settings.api_key(Provider::ClaudeCode), None);
    }

    #[test]
    fn parse_default_falls_back_to_codex() {
        assert_eq!(super::parse_default(None), Provider::Codex);
        assert_eq!(
            super::parse_default(Some("groq")),
            Provider::Compat(Compat::Groq)
        );
        assert_eq!(super::parse_default(Some("unknown")), Provider::Codex);
    }
}
