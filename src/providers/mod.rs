mod claude;
mod codex;
mod mistral;
mod openai;

use std::fmt::{Display, Formatter};
use std::str::FromStr;

const MAX_RESPONSE_BYTES: u64 = 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Provider {
    Codex,
    OpenAi,
    Claude,
    Mistral,
}

#[derive(Debug)]
pub struct AskRequest {
    pub provider: Provider,
    pub model: Option<String>,
    pub prompt: String,
}

impl Provider {
    pub fn ask(self, request: &AskRequest) -> Result<String, String> {
        match self {
            Self::Codex => codex::ask(request),
            Self::OpenAi => openai::ask(request),
            Self::Claude => claude::ask(request),
            Self::Mistral => mistral::ask(request),
        }
    }

    pub const fn requires_model(self) -> bool {
        !matches!(self, Self::Codex)
    }
}

impl Display for Provider {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Codex => "codex",
            Self::OpenAi => "openai",
            Self::Claude => "claude",
            Self::Mistral => "mistral",
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

pub(super) fn api_key(variable: &str) -> Result<String, String> {
    std::env::var(variable).map_err(|_| format!("{variable} is not configured"))
}

pub(super) fn http_error(provider: Provider, error: &ureq::Error) -> String {
    format!("{provider} request failed: {error}")
}

pub(super) fn http_agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .max_redirects(0)
        .build()
        .into()
}

pub(super) fn read_json(
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
