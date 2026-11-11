mod anthropic_events;
mod ask_prompt;
mod chat;
mod claude;
mod claude_code;
mod codex;
pub mod compat;
pub mod config;
mod endpoint;
mod local;
mod openai;
mod openai_stream;
mod sse;

pub use compat::Compat;
pub use endpoint::Endpoint;
pub use local::base_url as local_base_url;
pub use local::models as local_models;

use config::ProviderSettings;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::sync::OnceLock;

const MAX_RESPONSE_BYTES: u64 = 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Provider {
    Codex,
    ClaudeCode,
    OpenAi,
    Claude,
    Mistral,
    Ollama,
    LmStudio,
    Compat(Compat),
}

#[derive(Debug)]
pub struct AskRequest {
    pub provider: Provider,
    pub model: Option<String>,
    pub prompt: String,
    pub endpoint: Endpoint,
}

impl AskRequest {
    /// Resolves model, origin, and credential from one settings load.
    pub fn build(
        settings: &ProviderSettings,
        provider: Provider,
        model: Option<String>,
        prompt: String,
    ) -> Result<Self, String> {
        Ok(Self {
            provider,
            model: resolve_model(settings, provider, model)?,
            prompt,
            endpoint: Endpoint::resolve(provider, settings),
        })
    }
}

impl Provider {
    pub const ALL: [Self; 14] = [
        Self::Codex,
        Self::ClaudeCode,
        Self::OpenAi,
        Self::Claude,
        Self::Mistral,
        Self::Ollama,
        Self::LmStudio,
        Self::Compat(Compat::OpenRouter),
        Self::Compat(Compat::Groq),
        Self::Compat(Compat::DeepSeek),
        Self::Compat(Compat::Xai),
        Self::Compat(Compat::Gemini),
        Self::Compat(Compat::Together),
        Self::Compat(Compat::Custom),
    ];

    pub fn ask(self, request: &AskRequest) -> Result<String, String> {
        match self {
            Self::Codex => codex::ask(request),
            Self::ClaudeCode => claude_code::ask(request),
            Self::OpenAi => openai::ask(request),
            Self::Claude => claude::ask(request),
            Self::Mistral | Self::Ollama | Self::LmStudio | Self::Compat(_) => chat::ask(request),
        }
    }

    pub fn ask_stream(
        self,
        request: &AskRequest,
        on_delta: &mut dyn FnMut(&str),
    ) -> Result<String, String> {
        match self {
            Self::OpenAi => openai_stream::ask_stream(request, on_delta),
            Self::Claude => claude::ask_stream(request, on_delta),
            Self::ClaudeCode => claude_code::ask_stream(request, on_delta),
            Self::Mistral | Self::Ollama | Self::LmStudio | Self::Compat(_) => {
                chat::ask_stream(request, on_delta)
            }
            Self::Codex => {
                let answer = codex::ask(request)?;
                if !answer.is_empty() {
                    on_delta(&answer);
                }
                Ok(answer)
            }
        }
    }

    /// Delegated CLIs pick their own default model; everything else needs one.
    pub const fn requires_model(self) -> bool {
        !self.is_cli()
    }

    pub const fn is_cli(self) -> bool {
        matches!(self, Self::Codex | Self::ClaudeCode)
    }

    pub const fn uses_api_key(self) -> bool {
        matches!(
            self,
            Self::OpenAi | Self::Claude | Self::Mistral | Self::Compat(_)
        )
    }

    pub const fn is_local(self) -> bool {
        matches!(self, Self::Ollama | Self::LmStudio)
    }

    /// True when a delegated CLI reports a usable login; false for others.
    pub fn cli_session_ready(self) -> bool {
        match self {
            Self::Codex => codex::session_ready(),
            Self::ClaudeCode => claude_code::session_ready(),
            _ => false,
        }
    }

    /// The command a person runs to sign a delegated CLI in.
    pub const fn login_command(self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude auth login",
            _ => "codex login",
        }
    }

    /// Wire name of the provider family in `ai-models --json`.
    pub const fn kind(self) -> &'static str {
        if self.is_cli() {
            "cli"
        } else if self.is_local() {
            "local"
        } else {
            "api"
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Codex => "ChatGPT subscription via Codex CLI (codex login)",
            Self::ClaudeCode => "Claude Pro/Max via Claude Code CLI (claude auth login)",
            Self::OpenAi => "OpenAI API key",
            Self::Claude => "Anthropic API key",
            Self::Mistral => "Mistral API key",
            Self::Ollama => "Ollama (local, no API key)",
            Self::LmStudio => "LM Studio (local, no API key)",
            Self::Compat(preset) => preset.label(),
        }
    }
}

impl Display for Provider {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Codex => "codex",
            Self::ClaudeCode => "claude-code",
            Self::OpenAi => "openai",
            Self::Claude => "claude",
            Self::Mistral => "mistral",
            Self::Ollama => "ollama",
            Self::LmStudio => "lmstudio",
            Self::Compat(preset) => preset.name(),
        };
        formatter.pad(name)
    }
}

impl FromStr for Provider {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "codex" | "chatgpt" => Ok(Self::Codex),
            "claude-code" | "claudecode" => Ok(Self::ClaudeCode),
            "openai" => Ok(Self::OpenAi),
            "claude" | "anthropic" => Ok(Self::Claude),
            "mistral" => Ok(Self::Mistral),
            "ollama" => Ok(Self::Ollama),
            "lmstudio" | "lm-studio" => Ok(Self::LmStudio),
            other => Compat::from_name(other)
                .map(Self::Compat)
                .ok_or_else(|| format!("unknown provider '{value}'")),
        }
    }
}

pub(super) fn model(request: &AskRequest) -> Result<&str, String> {
    request
        .model
        .as_deref()
        .ok_or_else(|| format!("provider '{}' requires --model", request.provider))
}

fn resolve_model(
    settings: &ProviderSettings,
    provider: Provider,
    explicit: Option<String>,
) -> Result<Option<String>, String> {
    if let Some(model) = explicit.or_else(|| settings.model(provider).map(str::to_owned)) {
        return Ok(Some(model));
    }
    if !provider.requires_model() {
        return Ok(None);
    }
    Err(format!(
        "provider '{provider}' requires --model (run ai-connect)"
    ))
}

pub(super) fn http_error(provider: Provider, error: &ureq::Error) -> String {
    format!("{provider} request failed: {error}")
}

/// One process-wide agent so repeated calls reuse pooled TLS connections.
pub(crate) fn http_agent() -> &'static ureq::Agent {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT.get_or_init(|| {
        ureq::Agent::config_builder()
            .max_redirects(0)
            .build()
            .into()
    })
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
    use super::{AskRequest, Provider, config::ProviderSettings};

    #[test]
    fn provider_names_roundtrip() {
        for provider in Provider::ALL {
            let parsed: Provider = provider.to_string().parse().expect("name");
            assert_eq!(parsed, provider);
        }
        assert_eq!("lm-studio".parse::<Provider>().unwrap(), Provider::LmStudio);
        assert_eq!("anthropic".parse::<Provider>().unwrap(), Provider::Claude);
    }

    #[test]
    fn cli_providers_need_no_model() {
        let settings = ProviderSettings::default();
        let request = AskRequest::build(&settings, Provider::ClaudeCode, None, "hi".into());
        assert_eq!(request.expect("request").model, None);
        let missing = AskRequest::build(&settings, Provider::Mistral, None, "hi".into());
        assert!(missing.unwrap_err().contains("requires --model"));
    }
}
