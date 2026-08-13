mod claude;
mod codex;
pub mod config;
mod local;
mod mistral;
mod openai;

pub use local::base_url as local_base_url;
pub use local::models as local_models;
pub(crate) use openai::url as openai_url;

use std::fmt::{Display, Formatter};
use std::str::FromStr;

const MAX_RESPONSE_BYTES: u64 = 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Provider {
    Codex,
    OpenAi,
    Claude,
    Mistral,
    Ollama,
    LmStudio,
}

#[derive(Debug)]
pub struct AskRequest {
    pub provider: Provider,
    pub model: Option<String>,
    pub prompt: String,
}

impl Provider {
    pub const ALL: [Self; 6] = [
        Self::Codex,
        Self::OpenAi,
        Self::Claude,
        Self::Mistral,
        Self::Ollama,
        Self::LmStudio,
    ];

    pub fn ask(self, request: &AskRequest) -> Result<String, String> {
        match self {
            Self::Codex => codex::ask(request),
            Self::OpenAi => openai::ask(request),
            Self::Claude => claude::ask(request),
            Self::Mistral => mistral::ask(request),
            Self::Ollama | Self::LmStudio => local::ask(self, request),
        }
    }

    pub const fn requires_model(self) -> bool {
        !matches!(self, Self::Codex)
    }

    /// Hosted providers authenticate with a key; Codex delegates to its CLI and
    /// locally hosted runtimes need no credential at all.
    pub const fn uses_api_key(self) -> bool {
        matches!(self, Self::OpenAi | Self::Claude | Self::Mistral)
    }

    pub const fn is_local(self) -> bool {
        matches!(self, Self::Ollama | Self::LmStudio)
    }
}

impl Display for Provider {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Codex => "codex",
            Self::OpenAi => "openai",
            Self::Claude => "claude",
            Self::Mistral => "mistral",
            Self::Ollama => "ollama",
            Self::LmStudio => "lmstudio",
        };
        formatter.write_str(name)
    }
}

impl FromStr for Provider {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "codex" => Ok(Self::Codex),
            "openai" => Ok(Self::OpenAi),
            "claude" | "anthropic" => Ok(Self::Claude),
            "mistral" => Ok(Self::Mistral),
            "ollama" => Ok(Self::Ollama),
            "lmstudio" | "lm-studio" => Ok(Self::LmStudio),
            _ => Err(format!("unknown provider '{value}'")),
        }
    }
}

pub(super) fn model(request: &AskRequest) -> Result<&str, String> {
    request
        .model
        .as_deref()
        .ok_or_else(|| format!("provider '{}' requires --model", request.provider))
}

pub fn resolve_model(
    provider: Provider,
    explicit: Option<String>,
) -> Result<Option<String>, String> {
    if explicit.is_some() {
        return Ok(explicit);
    }
    if !provider.requires_model() {
        return Ok(None);
    }
    config::stored_model(provider)
        .map(Some)
        .ok_or_else(|| format!("provider '{provider}' requires --model (run ai-connect)"))
}

pub(super) fn api_key(variable: &str) -> Result<String, String> {
    let provider = match variable {
        "OPENAI_API_KEY" => Provider::OpenAi,
        "ANTHROPIC_API_KEY" => Provider::Claude,
        "MISTRAL_API_KEY" => Provider::Mistral,
        other => return Err(format!("{other} is not configured")),
    };
    config::resolve_api_key(provider)
}

pub(super) fn http_error(provider: Provider, error: &ureq::Error) -> String {
    format!("{provider} request failed: {error}")
}

pub(crate) fn http_agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .max_redirects(0)
        .build()
        .into()
}

pub(crate) fn read_json(
    provider: Provider,
    response: &mut ureq::http::Response<ureq::Body>,
) -> Result<serde_json::Value, String> {
    response
        .body_mut()
        .with_config()
        .limit(MAX_RESPONSE_BYTES)
        .read_json()
        .map_err(|error| http_error(provider, &error))
}

pub(super) fn non_empty(provider: Provider, text: String) -> Result<String, String> {
    (!text.is_empty())
        .then_some(text)
        .ok_or_else(|| format!("{provider} returned no text"))
}

#[cfg(test)]
mod tests {
    use super::Provider;

    #[test]
    fn provider_names_roundtrip() {
        for provider in Provider::ALL {
            let parsed: Provider = provider.to_string().parse().expect("name");
            assert_eq!(parsed, provider);
        }
        assert_eq!("lm-studio".parse::<Provider>().unwrap(), Provider::LmStudio);
        assert_eq!("anthropic".parse::<Provider>().unwrap(), Provider::Claude);
    }
}
