mod menu;
mod models;

use crate::providers::Provider;
use crate::providers::config::{self, ProviderEntry};
use std::ffi::OsString;
use std::io::{self, IsTerminal};
use std::process::Command;

const HELP: &str = "Usage: ai-connect [--status]

Interactive provider setup for OpenTerm.

Use arrow keys to pick a provider, then a model (sorted newest first).
API keys are stored in ~/.config/openterm/providers.json (mode 600).
Codex authentication is delegated to the installed Codex CLI.
Ollama and LM Studio need no key; start the runtime, then pick a pulled model.
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
    let provider = menu::select_provider()?;
    let model = match provider {
        Provider::Codex => connect_codex()?,
        other if other.is_local() => connect_local_provider(other)?,
        other if other.uses_api_key() => connect_api_provider(other)?,
        other => return Err(format!("provider '{other}' cannot be connected this way")),
    };
    println!("\nConnected to {provider} · {model}");
    if let Ok(path) = config::config_path() {
        println!("Saved to {}", path.display());
    }
    Ok(())
}

fn connect_codex() -> Result<String, String> {
    if !codex_session_ready() {
        return Err("Codex CLI is unavailable or not authenticated; run 'codex login'".to_owned());
    }
    let mut settings = config::load();
    settings.default_provider = Some("codex".to_owned());
    config::save(&settings)?;
    Ok("Codex CLI default model".to_owned())
}

fn connect_api_provider(provider: Provider) -> Result<String, String> {
    let api_key = menu::read_api_key(provider)?;
    let choices = models::list_for_provider(provider, &api_key);
    persist_model(
        provider,
        api_key,
        menu::select_model(&choices, "Select model")?,
    )
}

fn connect_local_provider(provider: Provider) -> Result<String, String> {
    let choices = models::list_for_provider(provider, "");
    if choices.is_empty() {
        return Err(format!(
            "{provider} is not reachable at {}; start it, pull a model, then retry",
            crate::providers::local_base_url(provider)
        ));
    }
    persist_model(
        provider,
        String::new(),
        menu::select_model(&choices, "Select model")?,
    )
}

fn persist_model(provider: Provider, api_key: String, model: String) -> Result<String, String> {
    let mut settings = config::load();
    let base_url = kept_openai_base(provider, &settings);
    config::store_entry(
        &mut settings,
        provider,
        ProviderEntry {
            api_key,
            model: Some(model.clone()),
            base_url,
        },
    );
    settings.default_provider = Some(provider.to_string());
    config::save(&settings)?;
    crate::hud::sync_from_config();
    Ok(model)
}

fn kept_openai_base(provider: Provider, settings: &config::ProviderSettings) -> Option<String> {
    if provider != Provider::OpenAi {
        return None;
    }
    std::env::var("OPENAI_BASE_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            settings
                .openai
                .as_ref()
                .and_then(|entry| entry.base_url.clone())
        })
}

fn print_status() -> i32 {
    println!("OpenTerm provider status\n");
    for provider in Provider::ALL {
        println!("  {provider:<10} {}", describe(provider));
    }
    if let Ok(path) = config::config_path() {
        println!("\nConfig: {}", path.display());
    }
    if let Some(default) = config::load().default_provider.as_deref() {
        println!("Default provider: {default}");
    }
    println!("\nRun ai-connect to connect with arrow-key selection.");
    crate::hud::sync_from_config();
    0
}

fn describe(provider: Provider) -> String {
    if provider == Provider::Codex {
        if codex_session_ready() {
            return "connected (Codex CLI session)".to_owned();
        }
        return "not connected — run codex login".to_owned();
    }
    if provider.is_local() {
        return describe_local(provider);
    }
    describe_api(provider)
}

fn describe_local(provider: Provider) -> String {
    let host = crate::providers::local_base_url(provider);
    if config::is_stored(provider) {
        let model = config::stored_model(provider)
            .map(|value| format!(", model {value}"))
            .unwrap_or_default();
        return format!("connected (local at {host}{model})");
    }
    format!("not connected — start {provider} at {host}, then run ai-connect")
}

fn describe_api(provider: Provider) -> String {
    if let Some(env) = config::env_var(provider) {
        if std::env::var_os(env).is_some() {
            return "configured (environment)".to_owned();
        }
    }
    if config::is_stored(provider) {
        let model = config::stored_model(provider)
            .map(|value| format!(", model {value}"))
            .unwrap_or_default();
        return format!("connected (openterm{model})");
    }
    "not connected — run ai-connect".to_owned()
}

fn codex_session_ready() -> bool {
    Command::new("codex")
        .args(["login", "status"])
        .output()
        .is_ok_and(|output| output.status.success())
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
