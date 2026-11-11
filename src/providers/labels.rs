//! Human-readable provider names for menus, tables, and the macOS app.

use super::{Compat, Provider};

impl Provider {
    /// Short display name for lists and sidebars; `label` adds setup context.
    pub const fn short_label(self) -> &'static str {
        match self {
            Self::Codex => "ChatGPT (Codex CLI)",
            Self::ClaudeCode => "Claude (Claude Code)",
            Self::OpenAi => "OpenAI",
            Self::Claude => "Anthropic",
            Self::Mistral => "Mistral",
            Self::Ollama => "Ollama",
            Self::LmStudio => "LM Studio",
            Self::Compat(Compat::Custom) => "Custom URL",
            Self::Compat(preset) => preset.label(),
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
