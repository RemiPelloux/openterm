//! `ai-models use [PROVIDER [MODEL]]`: pick the model (and default provider)
//! that ai-ask, ai-explain, and ai-chat use, with arrow keys or directly.

use super::{cache, refresh};
use crate::providers::config::{self, ProviderSettings};
use crate::providers::{Endpoint, Provider};
use dialoguer::{Input, Select, theme::ColorfulTheme};
use std::io::IsTerminal;

/// Claude Code resolves these aliases to the newest model of each family.
const CLAUDE_CODE_ALIASES: [&str; 3] = ["opus", "sonnet", "haiku"];
const TYPE_OWN: &str = "Type a model id…";
const CLI_DEFAULT: &str = "Let the CLI choose (its default)";

#[derive(Debug, PartialEq, Eq)]
enum Choice {
    Model(String),
    CliDefault,
}

pub fn run(args: &[String]) -> i32 {
    match choose(args) {
        Ok(line) => {
            println!("{line}");
            0
        }
        Err(error) => {
            eprintln!("error: {error}");
            1
        }
    }
}

fn choose(args: &[String]) -> Result<String, String> {
    let mut settings = config::load();
    let (provider, choice) = match args {
        [] => {
            let provider = pick_provider(&settings)?;
            (provider, pick_model(&settings, provider)?)
        }
        [name] => {
            let provider = name.parse()?;
            (provider, pick_model(&settings, provider)?)
        }
        [name, model] => (name.parse()?, Choice::Model(model.trim().to_owned())),
        _ => return Err("usage: ai-models use [PROVIDER [MODEL]]".to_owned()),
    };
    ensure_usable(&settings, provider)?;
    apply(&mut settings, provider, choice)
}

fn ensure_usable(settings: &ProviderSettings, provider: Provider) -> Result<(), String> {
    let usable = if provider.is_cli() {
        provider.cli_session_ready()
    } else if provider.is_local() {
        settings.is_stored(provider) || !crate::providers::local_models(provider).is_empty()
    } else {
        refresh::has_discovery(provider, settings)
    };
    if usable {
        return Ok(());
    }
    Err(format!(
        "{provider} is not connected yet; run 'ai-connect {provider}' first"
    ))
}

fn apply(
    settings: &mut ProviderSettings,
    provider: Provider,
    choice: Choice,
) -> Result<String, String> {
    let model = match choice {
        Choice::Model(model) if model.is_empty() => return Err("model id is empty".to_owned()),
        Choice::Model(model) => Some(model),
        Choice::CliDefault if provider.requires_model() => {
            return Err(format!("{provider} needs a model id"));
        }
        Choice::CliDefault => None,
    };
    let mut entry = settings.entry(provider).cloned().unwrap_or_default();
    entry.model.clone_from(&model);
    settings.set_entry(provider, entry);
    settings.default_provider = Some(provider.to_string());
    config::save(settings)?;
    crate::hud::sync_from_config(settings);
    let shown = model.unwrap_or_else(|| "CLI default model".to_owned());
    Ok(format!("Using {provider} · {shown}"))
}

fn pick_provider(settings: &ProviderSettings) -> Result<Provider, String> {
    require_terminal()?;
    let choices: Vec<Provider> = Provider::ALL
        .into_iter()
        .filter(|provider| ensure_usable(settings, *provider).is_ok())
        .collect();
    if choices.is_empty() {
        return Err("no connected providers; run ai-connect first".to_owned());
    }
    let labels: Vec<String> = choices
        .iter()
        .map(|provider| provider_label(settings, *provider))
        .collect();
    let current = settings.default_provider();
    let start = choices.iter().position(|p| *p == current).unwrap_or(0);
    Ok(choices[select("Provider", &labels, start)?])
}

fn provider_label(settings: &ProviderSettings, provider: Provider) -> String {
    let model = settings.model(provider).unwrap_or("default model");
    format!("{:<22} {model}", provider.short_label())
}

fn pick_model(settings: &ProviderSettings, provider: Provider) -> Result<Choice, String> {
    require_terminal()?;
    let models = candidates(settings, provider);
    let mut labels = models.clone();
    labels.push(TYPE_OWN.to_owned());
    if provider.is_cli() {
        labels.push(CLI_DEFAULT.to_owned());
    }
    let current = settings.model(provider);
    let start = current
        .and_then(|model| models.iter().position(|m| m == model))
        .unwrap_or(0);
    let index = select(&format!("{} model", provider.short_label()), &labels, start)?;
    match labels[index].as_str() {
        TYPE_OWN => type_model(current),
        CLI_DEFAULT => Ok(Choice::CliDefault),
        model => Ok(Choice::Model(model.to_owned())),
    }
}

/// Cached discovery for hosted and local providers (refreshed when empty);
/// `claude-code` aliases; `codex` reuses the cached `openai` list.
fn candidates(settings: &ProviderSettings, provider: Provider) -> Vec<String> {
    match provider {
        Provider::ClaudeCode => CLAUDE_CODE_ALIASES.map(str::to_owned).to_vec(),
        Provider::Codex => cached(settings, Provider::OpenAi)
            .into_iter()
            .filter(|model| !model.contains("image"))
            .collect(),
        _ => {
            let models = cached(settings, provider);
            if !models.is_empty() {
                return models;
            }
            refresh::run(settings, Some(provider), true);
            cached(settings, provider)
        }
    }
}

fn cached(settings: &ProviderSettings, provider: Provider) -> Vec<String> {
    let base_url = Endpoint::resolve(provider, settings).base_url;
    cache::load()
        .for_endpoint(provider, &base_url)
        .map(|entry| entry.models.clone())
        .unwrap_or_default()
}

fn type_model(current: Option<&str>) -> Result<Choice, String> {
    let theme = ColorfulTheme::default();
    let mut input = Input::<String>::with_theme(&theme).with_prompt("Model id");
    if let Some(model) = current {
        input = input.with_initial_text(model);
    }
    let model = input.interact_text().map_err(|error| error.to_string())?;
    Ok(Choice::Model(model.trim().to_owned()))
}

fn select(prompt: &str, items: &[String], start: usize) -> Result<usize, String> {
    Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(items)
        .default(start)
        .interact()
        .map_err(|error| error.to_string())
}

fn require_terminal() -> Result<(), String> {
    if std::io::stdin().is_terminal() {
        return Ok(());
    }
    Err("pick interactively in a terminal, or pass both: ai-models use PROVIDER MODEL".to_owned())
}

#[cfg(test)]
mod tests {
    use super::{Choice, apply};
    use crate::providers::Provider;
    use crate::providers::config::ProviderSettings;

    #[test]
    fn rejects_empty_and_cli_default_for_api_providers() {
        let mut settings = ProviderSettings::default();
        let empty = apply(
            &mut settings,
            Provider::OpenAi,
            Choice::Model(String::new()),
        );
        assert_eq!(empty.unwrap_err(), "model id is empty");
        let cli = apply(&mut settings, Provider::OpenAi, Choice::CliDefault);
        assert_eq!(cli.unwrap_err(), "openai needs a model id");
        assert_eq!(
            settings,
            ProviderSettings::default(),
            "nothing changed on error"
        );
    }
}
