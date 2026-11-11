use crate::providers::{Compat, Provider};
use dialoguer::{Input, Select, theme::ColorfulTheme};

pub fn select_provider() -> Result<Provider, String> {
    let labels: Vec<String> = Provider::ALL
        .iter()
        .map(|provider| format!("{provider} — {}", label(*provider)))
        .collect();
    let index = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select provider")
        .items(&labels)
        .default(0)
        .interact()
        .map_err(|error| error.to_string())?;
    Ok(Provider::ALL[index])
}

pub fn select_model(models: &[String], prompt: &str) -> Result<String, String> {
    if models.is_empty() {
        return Err("no models available for this provider".to_owned());
    }
    let index = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(models)
        .default(0)
        .interact()
        .map_err(|error| error.to_string())?;
    Ok(models[index].clone())
}

pub fn read_api_key(provider: Provider) -> Result<String, String> {
    let prompt = format!("Enter {provider} API key");
    dialoguer::Password::new()
        .with_prompt(prompt)
        .interact()
        .map_err(|error| error.to_string())
}

pub fn read_base_url(current: Option<&str>) -> Result<String, String> {
    let theme = ColorfulTheme::default();
    let mut input = Input::<String>::with_theme(&theme)
        .with_prompt("Base URL including version (e.g. https://host/v1)");
    if let Some(value) = current {
        input = input.with_initial_text(value);
    }
    let value = input.interact_text().map_err(|error| error.to_string())?;
    Ok(value.trim().trim_end_matches('/').to_owned())
}

fn label(provider: Provider) -> &'static str {
    match provider {
        Provider::Codex => "ChatGPT subscription via Codex CLI (codex login)",
        Provider::ClaudeCode => "Claude Pro/Max via Claude Code CLI (claude auth login)",
        Provider::OpenAi => "OpenAI API key",
        Provider::Claude => "Anthropic API key",
        Provider::Mistral => "Mistral API key",
        Provider::Ollama => "Ollama (local, no API key)",
        Provider::LmStudio => "LM Studio (local, no API key)",
        Provider::Compat(Compat::Custom) => "Any OpenAI-compatible URL + key",
        Provider::Compat(preset) => preset.label(),
    }
}
