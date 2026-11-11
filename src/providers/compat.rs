//! OpenAI-compatible Chat Completions gateways. Each preset is only an origin
//! and a credential variable; the wire protocol is shared in `chat.rs`.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Compat {
    OpenRouter,
    Groq,
    DeepSeek,
    Xai,
    Gemini,
    Together,
    Custom,
}

struct Spec {
    name: &'static str,
    label: &'static str,
    base_url: &'static str,
    env_key: &'static str,
}

const CUSTOM_BASE_ENV: &str = "OPENTERM_CUSTOM_BASE_URL";

impl Compat {
    pub const ALL: [Self; 7] = [
        Self::OpenRouter,
        Self::Groq,
        Self::DeepSeek,
        Self::Xai,
        Self::Gemini,
        Self::Together,
        Self::Custom,
    ];

    pub fn name(self) -> &'static str {
        self.spec().name
    }

    pub const fn label(self) -> &'static str {
        self.spec().label
    }

    pub fn env_key(self) -> &'static str {
        self.spec().env_key
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|preset| preset.name() == name)
    }

    /// Chat base including its version segment, e.g. `https://host/v1`.
    pub fn base_url(self, stored: Option<&str>) -> String {
        if self == Self::Custom {
            let env = std::env::var(CUSTOM_BASE_ENV).ok();
            return first_non_empty(env.as_deref(), stored).unwrap_or_default();
        }
        first_non_empty(stored, None).unwrap_or_else(|| self.spec().base_url.to_owned())
    }

    const fn spec(self) -> &'static Spec {
        &SPECS[self as usize]
    }
}

/// Indexed by `Compat` discriminant; order must match the enum.
const SPECS: [Spec; 7] = [
    spec(
        "openrouter",
        "OpenRouter",
        "https://openrouter.ai/api/v1",
        "OPENROUTER_API_KEY",
    ),
    spec(
        "groq",
        "Groq",
        "https://api.groq.com/openai/v1",
        "GROQ_API_KEY",
    ),
    spec(
        "deepseek",
        "DeepSeek",
        "https://api.deepseek.com/v1",
        "DEEPSEEK_API_KEY",
    ),
    spec("xai", "xAI Grok", "https://api.x.ai/v1", "XAI_API_KEY"),
    spec(
        "gemini",
        "Google Gemini",
        "https://generativelanguage.googleapis.com/v1beta/openai",
        "GEMINI_API_KEY",
    ),
    spec(
        "together",
        "Together AI",
        "https://api.together.xyz/v1",
        "TOGETHER_API_KEY",
    ),
    spec(
        "custom",
        "Custom OpenAI-compatible URL",
        "",
        "OPENTERM_CUSTOM_API_KEY",
    ),
];

const fn spec(
    name: &'static str,
    label: &'static str,
    base_url: &'static str,
    env_key: &'static str,
) -> Spec {
    Spec {
        name,
        label,
        base_url,
        env_key,
    }
}

fn first_non_empty(first: Option<&str>, second: Option<&str>) -> Option<String> {
    first
        .into_iter()
        .chain(second)
        .map(|value| value.trim().trim_end_matches('/'))
        .find(|value| !value.is_empty())
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::Compat;

    #[test]
    fn names_roundtrip_and_are_unique() {
        for preset in Compat::ALL {
            assert_eq!(Compat::from_name(preset.name()), Some(preset));
        }
        assert_eq!(Compat::from_name("nope"), None);
    }

    #[test]
    fn stored_base_overrides_preset_and_trims_slash() {
        assert_eq!(
            Compat::Groq.base_url(None),
            "https://api.groq.com/openai/v1"
        );
        assert_eq!(
            Compat::Groq.base_url(Some("https://proxy.example/v1/")),
            "https://proxy.example/v1"
        );
    }

    #[test]
    fn every_preset_but_custom_has_a_base_url() {
        for preset in Compat::ALL {
            let has_base = !preset.spec().base_url.is_empty();
            assert_eq!(has_base, preset != Compat::Custom, "{}", preset.name());
        }
    }
}
