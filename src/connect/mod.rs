mod args;
mod direct;
mod menu;
mod models;
mod status;

use crate::providers::config::keychain::Keychain;
use crate::providers::config::{self, ProviderEntry, ProviderSettings};
use crate::providers::{Compat, Endpoint, Provider};
use args::Action;
use std::ffi::OsString;
use std::io::{self, IsTerminal};
use std::process::Command;

const HELP: &str = "Usage: ai-connect
       ai-connect PROVIDER [--model MODEL] [--key-stdin] [--base-url URL] [--no-default]
       ai-connect --default PROVIDER
       ai-connect --status [--json]

Connect OpenTerm to an AI provider.

Without arguments, pick a provider and model with arrow keys.
With PROVIDER, connect without prompts (scripts and the macOS app):
  --model MODEL    model id; defaults to the saved one (list: ai-models PROVIDER)
  --key-stdin      read the API key from stdin, e.g.
                   pbpaste | ai-connect groq --model MODEL --key-stdin
                   keys are never accepted as command-line arguments
  --base-url URL   OpenAI-compatible base URL; required for custom
  --no-default     keep the current default provider
--default PROVIDER switches the default provider only.
--status [--json] shows each provider's connection and key source.

API keys are stored in the macOS Keychain (service ai.openterm.providers).
Where it is unavailable they fall back to ~/.config/openterm/providers.json
(mode 600). Environment variables such as OPENAI_API_KEY take precedence.
Subscriptions: ChatGPT through the Codex CLI (codex login), Claude Pro/Max
through the Claude Code CLI (claude auth login). OpenTerm never stores their tokens.
Ollama and LM Studio need no key; start the runtime, then pick a pulled model.
";

pub fn run(args: impl IntoIterator<Item = OsString>) -> i32 {
    match args::parse(args) {
        Ok(Action::Help) => print_help(),
        Ok(Action::Status { json }) => {
            status::print(json);
            0
        }
        Ok(Action::SetDefault(provider)) => finish(direct::set_default(provider), |()| {
            format!("Default provider: {provider}")
        }),
        Ok(Action::Connect(connect)) => finish(direct::run(&connect), |model| {
            format!("Connected to {} · {model}", connect.provider)
        }),
        Ok(Action::Interactive) => run_interactive(),
        Err(error) => usage_error(&error),
    }
}

fn finish<T>(result: Result<T, String>, line: impl FnOnce(T) -> String) -> i32 {
    match result {
        Ok(value) => {
            println!("{}", line(value));
            0
        }
        Err(error) => runtime_error(&error),
    }
}

fn run_interactive() -> i32 {
    if !io::stdin().is_terminal() {
        return usage_error("ai-connect without PROVIDER requires an interactive terminal");
    }
    match run_wizard() {
        Ok(()) => 0,
        Err(error) => runtime_error(&error),
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
    ensure_cli_ready(provider)?;
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
    let key = if provider.uses_api_key() {
        Some(menu::read_api_key(provider)?)
    } else {
        None
    };
    let model = choose_model(settings, provider, &entry, key.clone())?;
    entry.model = Some(model.clone());
    settings.set_entry(provider, entry);
    if let Some(key) = key {
        direct::report_saved(provider, &settings.store_key(provider, key, &Keychain));
    }
    settings.default_provider = Some(provider.to_string());
    config::save(settings)?;
    crate::hud::sync_from_config(settings);
    Ok(model)
}

/// Lists models with the key just typed; hosted providers that list nothing
/// fall back to a typed model id.
fn choose_model(
    settings: &ProviderSettings,
    provider: Provider,
    entry: &ProviderEntry,
    key: Option<String>,
) -> Result<String, String> {
    let mut draft = settings.clone();
    draft.set_entry(provider, entry.clone());
    let endpoint = Endpoint::with_key(provider, &draft, key);
    let choices = models::list_for_provider(provider, &endpoint);
    if !choices.is_empty() {
        return menu::select_model(&choices, "Select model");
    }
    if provider.is_local() {
        return Err(format!(
            "{provider} is not reachable at {}; start it, pull a model, then retry",
            crate::providers::local_base_url(provider)
        ));
    }
    menu::read_model(provider)
}

fn ensure_cli_ready(provider: Provider) -> Result<(), String> {
    if cli_session_ready(provider) {
        return Ok(());
    }
    Err(format!(
        "{provider} CLI is unavailable or not signed in; run '{}'",
        login_hint(provider)
    ))
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

fn usage_error(error: &str) -> i32 {
    eprintln!("error: {error}\n\n{HELP}");
    2
}

fn runtime_error(error: &str) -> i32 {
    eprintln!("error: {error}");
    1
}
