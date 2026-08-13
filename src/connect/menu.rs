use crate::providers::Provider;
use dialoguer::{Select, theme::ColorfulTheme};

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

fn label(provider: Provider) -> &'static str {
    match provider {
        Provider::Codex => "Codex CLI session (run codex login first)",
        Provider::OpenAi => "OpenAI API key",
        Provider::Claude => "Anthropic API key",
        Provider::Mistral => "Mistral API key",
        Provider::Ollama => "Ollama (local, no API key)",
        Provider::LmStudio => "LM Studio (local, no API key)",
    }
}
