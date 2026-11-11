use super::config::ProviderSettings;
use super::{Provider, local};
use std::fmt::{Debug, Formatter};

const OPENAI_DEFAULT: &str = "https://api.openai.com";
const ANTHROPIC_DEFAULT: &str = "https://api.anthropic.com";
const MISTRAL_DEFAULT: &str = "https://api.mistral.ai/v1";
const OPENAI_BASE_ENV: &str = "OPENAI_BASE_URL";

/// Where a request goes and the credential bound to that origin.
///
/// `base_url` is an origin for the Responses and Messages APIs, and a
/// versioned base (`.../v1`) for Chat Completions providers.
#[derive(Clone, Default)]
pub struct Endpoint {
    pub base_url: String,
    api_key: Option<String>,
}

impl Endpoint {
    pub fn resolve(provider: Provider, settings: &ProviderSettings) -> Self {
        Self::with_key(provider, settings, settings.api_key(provider))
    }

    /// Uses `api_key` as given, so callers that only need the origin (or
    /// hold a key not yet saved) skip the keychain lookup.
    pub fn with_key(
        provider: Provider,
        settings: &ProviderSettings,
        api_key: Option<String>,
    ) -> Self {
        Self {
            base_url: base_url(provider, settings),
            api_key,
        }
    }

    #[cfg(test)]
    pub fn new(base_url: String, api_key: Option<String>) -> Self {
        Self { base_url, api_key }
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}/{path}", self.base_url)
    }

    pub fn api_key(&self, provider: Provider) -> Result<&str, String> {
        self.api_key.as_deref().ok_or_else(|| {
            let variable = super::config::env_var(provider).unwrap_or("API key");
            format!("{variable} is not configured (run ai-connect)")
        })
    }

    pub fn optional_api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }
}

impl Debug for Endpoint {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let key = if self.api_key.is_some() {
            "<set>"
        } else {
            "<none>"
        };
        formatter
            .debug_struct("Endpoint")
            .field("base_url", &self.base_url)
            .field("api_key", &key)
            .finish()
    }
}

fn base_url(provider: Provider, settings: &ProviderSettings) -> String {
    let stored = settings.base_url(provider);
    match provider {
        Provider::OpenAi => openai_origin(std::env::var(OPENAI_BASE_ENV).ok().as_deref(), stored),
        Provider::Claude => origin(stored).unwrap_or_else(|| ANTHROPIC_DEFAULT.to_owned()),
        Provider::Mistral => origin(stored).unwrap_or_else(|| MISTRAL_DEFAULT.to_owned()),
        Provider::Ollama | Provider::LmStudio => format!("{}/v1", local::base_url(provider)),
        Provider::Compat(preset) => preset.base_url(stored),
        Provider::Codex | Provider::ClaudeCode => String::new(),
    }
}

fn origin(value: Option<&str>) -> Option<String> {
    value
        .map(|value| value.trim().trim_end_matches('/'))
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

/// Responses paths already carry `v1/`, so a pasted `.../v1` base is trimmed.
fn openai_origin(env: Option<&str>, stored: Option<&str>) -> String {
    let Some(value) = origin(env).or_else(|| origin(stored)) else {
        return OPENAI_DEFAULT.to_owned();
    };
    value
        .strip_suffix("/v1")
        .map_or(value.clone(), |base| base.trim_end_matches('/').to_owned())
}

#[cfg(test)]
mod tests {
    use super::{Endpoint, openai_origin};

    #[test]
    fn openai_prefers_env_then_stored_then_default() {
        assert_eq!(openai_origin(None, None), "https://api.openai.com");
        assert_eq!(
            openai_origin(None, Some("https://llm.example.com/v1/")),
            "https://llm.example.com"
        );
        assert_eq!(
            openai_origin(Some("https://gateway.example/"), Some("https://ignored")),
            "https://gateway.example"
        );
    }

    #[test]
    fn debug_never_prints_the_key() {
        let endpoint = Endpoint::new("https://x".into(), Some("sk-secret".into()));
        let text = format!("{endpoint:?}");
        assert!(!text.contains("sk-secret"));
        assert!(text.contains("<set>"));
    }
}
