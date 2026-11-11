mod menu;
mod models;

use crate::providers::config::{self, ProviderEntry, ProviderSettings};
use crate::providers::{Compat, Endpoint, Provider};
use std::ffi::OsString;
use std::io::{self, IsTerminal};
use std::process::Command;

const HELP: &str = "Usage: ai-connect [--status]

Interactive provider setup for OpenTerm.

Use arrow keys to pick a provider, then a model (sorted newest first).
API keys are stored in ~/.config/openterm/providers.json (mode 600).
Subscriptions: ChatGPT through the Codex CLI (codex login), Claude Pro/Max
through the Claude Code CLI (claude auth login). OpenTerm never stores their tokens.
Ollama and LM Studio need no key; start the runtime, then pick a pulled model.
OpenRouter, Groq, DeepSeek, xAI, Gemini, Together, or any custom
OpenAI-compatible URL connect with an API key.
";

pub fn run(args: impl IntoIterator<Item = OsString>) -> i32 {
    match parse_args(args) {
        ParseOutcome::Help => print_help(),
        ParseOutcome::Status => print_status(),
        ParseOutcome::Interactive => run_interactive(),
        ParseOutcome::Error(error) => print_error(&error, 2),
    }
}

enum ParseOutcome {
    Help,
    Status,
    Interactive,
    Error(String),
}

fn parse_args(args: impl IntoIterator<Item = OsString>) -> ParseOutcome {
    let mut status = false;
    for arg in args {
        match arg.to_string_lossy().as_ref() {
            "--help" | "-h" => return ParseOutcome::Help,
            "--status" => status = true,
            flag if flag.starts_with('-') => {
                return ParseOutcome::Error(format!("unknown option '{flag}'"));
            }
            _ => {
                return ParseOutcome::Error(
                    "ai-connect takes no arguments (use --status)".to_owned(),
                );
            }
        }
    }
    if status {
        ParseOutcome::Status
    } else {
        ParseOutcome::Interactive
    }
}

fn run_interactive() -> i32 {
    if !io::stdin().is_terminal() {
        return print_error("ai-connect requires an interactive terminal", 2);
    }
    match run_wizard() {
        Ok(()) => 0,
        Err(error) => print_error(&error, 1),
    }
}

fn run_wizard() -> Result<(), String> {
    let mut settings = config::load();
    let provider = menu::select_provider()?;
    let model = if provider.is_cli() {
        connect_cli(&mut settings, provider)?
    } else {
        connect_hosted(&mut settings, provider)?
    };
    println!("\nConnected to {provider} · {model}");
    if let Ok(path) = config::config_path() {
        println!("Saved to {}", path.display());
    }
    Ok(())
}

/// Delegated CLIs own their login (a GPT plan via `codex`, a Claude plan
/// via `claude`); only the default-provider choice is recorded here.
fn connect_cli(settings: &mut ProviderSettings, provider: Provider) -> Result<String, String> {
    if !cli_session_ready(provider) {
        return Err(format!(
            "{provider} CLI is unavailable or not signed in; run '{}'",
            login_hint(provider)
        ));
    }
    settings.default_provider = Some(provider.to_string());
    config::save(settings)?;
    crate::hud::sync_from_config(settings);
    Ok(format!("{provider} CLI default model"))
}

fn connect_hosted(settings: &mut ProviderSettings, provider: Provider) -> Result<String, String> {
    let mut entry = settings.entry(provider).cloned().unwrap_or_default();
    if provider == Provider::Compat(Compat::Custom) {
        entry.base_url = Some(menu::read_base_url(entry.base_url.as_deref())?);
    }
    if provider.uses_api_key() {
        entry.api_key = menu::read_api_key(provider)?;
    }
    let choices = list_models(settings, provider, &entry);
    if choices.is_empty() && provider.is_local() {
        return Err(format!(
            "{provider} is not reachable at {}; start it, pull a model, then retry",
            crate::providers::local_base_url(provider)
        ));
    }
    let model = menu::select_model(&choices, "Select model")?;
    entry.model = Some(model.clone());
    persist(settings, provider, entry)?;
    Ok(model)
}

fn list_models(
    settings: &ProviderSettings,
    provider: Provider,
    entry: &ProviderEntry,
) -> Vec<String> {
    let mut draft = settings.clone();
    draft.set_entry(provider, entry.clone());
    models::list_for_provider(provider, &Endpoint::resolve(provider, &draft))
}

fn persist(
    settings: &mut ProviderSettings,
    provider: Provider,
    entry: ProviderEntry,
) -> Result<(), String> {
    settings.set_entry(provider, entry);
    settings.default_provider = Some(provider.to_string());
    config::save(settings)?;
    crate::hud::sync_from_config(settings);
    Ok(())
}

fn print_status() -> i32 {
    let settings = config::load();
    println!("OpenTerm provider status\n");
    for provider in Provider::ALL {
        println!("  {provider:<12} {}", describe(provider, &settings));
    }
    if let Ok(path) = config::config_path() {
        println!("\nConfig: {}", path.display());
    }
    if let Some(default) = settings.default_provider.as_deref() {
        println!("Default provider: {default}");
    }
    println!("\nRun ai-connect to connect with arrow-key selection.");
    crate::hud::sync_from_config(&settings);
    0
}

fn describe(provider: Provider, settings: &ProviderSettings) -> String {
    if provider.is_cli() {
        if cli_session_ready(provider) {
            return format!("connected ({provider} CLI session)");
        }
        return format!("not connected — run {}", login_hint(provider));
    }
    let model = settings
        .model(provider)
        .map(|value| format!(", model {value}"))
        .unwrap_or_default();
    if provider.is_local() {
        let host = crate::providers::local_base_url(provider);
        if settings.is_stored(provider) {
            return format!("connected (local at {host}{model})");
        }
        return format!("not connected — start {provider} at {host}, then run ai-connect");
    }
    describe_api(provider, settings, &model)
}

fn describe_api(provider: Provider, settings: &ProviderSettings, model: &str) -> String {
    let from_env = config::env_var(provider).is_some_and(|env| std::env::var_os(env).is_some());
    if from_env {
        return format!("configured (environment{model})");
    }
    if settings.is_stored(provider) {
        return format!("connected (openterm{model})");
    }
    "not connected — run ai-connect".to_owned()
}

fn cli_session_ready(provider: Provider) -> bool {
    match provider {
        Provider::ClaudeCode => crate::providers::claude_code_ready(),
        _ => Command::new("codex")
            .args(["login", "status"])
            .output()
            .is_ok_and(|output| output.status.success()),
    }
}

fn login_hint(provider: Provider) -> &'static str {
    match provider {
        Provider::ClaudeCode => "claude auth login",
        _ => "codex login",
    }
}

fn print_help() -> i32 {
    print!("{HELP}");
    0
}

fn print_error(error: &str, code: i32) -> i32 {
    eprintln!("error: {error}\n\n{HELP}");
    code
}

#[cfg(test)]
mod tests {
    use super::{ParseOutcome, parse_args};
    use std::ffi::OsString;

    #[test]
    fn status_flag_parses() {
        assert!(matches!(
            parse_args([OsString::from("--status")]),
            ParseOutcome::Status
        ));
    }

    #[test]
    fn rejects_positional_args() {
        assert!(matches!(
            parse_args([OsString::from("openai")]),
            ParseOutcome::Error(_)
        ));
    }
}
