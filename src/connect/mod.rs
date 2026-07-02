mod codex_auth;
mod menu;
mod models;

use crate::providers::config::{self, CodexEntry, ProviderEntry};
use crate::providers::Provider;
use std::ffi::OsString;
use std::io::{self, IsTerminal};

const HELP: &str =
    "Usage: ai-connect [--status]

Interactive provider setup for OpenTerm.

Use arrow keys to pick a provider, then a model (sorted newest first).
API keys are stored in ~/.config/openterm/providers.json (mode 600).
Codex uses the same device-code browser login as OpenAgents.
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
        other => connect_api_provider(other)?,
    };
    println!("\nConnected to {provider} · {model}");
    if let Ok(path) = config::config_path() {
        println!("Saved to {}", path.display());
    }
    Ok(())
}

fn connect_codex() -> Result<String, String> {
    let tokens = codex_auth::ensure_codex_session()?;
    let choices = models::list_codex(&tokens.access_token);
    let model = menu::select_model(&choices, "Select Codex model")?;
    let mut settings = config::load();
    settings.codex = Some(CodexEntry {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        model: Some(model.clone()),
        source: tokens.source,
    });
    settings.default_provider = Some("codex".to_owned());
    config::save(&settings)?;
    Ok(model)
}

fn connect_api_provider(provider: Provider) -> Result<String, String> {
    let api_key = menu::read_api_key(provider)?;
    let choices = models::list_for_provider(provider, &api_key);
    let model = menu::select_model(&choices, "Select model")?;
    let mut settings = config::load();
    let entry = ProviderEntry {
        api_key,
        model: Some(model.clone()),
    };
    match provider {
        Provider::OpenAi => settings.openai = Some(entry),
        Provider::Claude => settings.claude = Some(entry),
        Provider::Mistral => settings.mistral = Some(entry),
        Provider::Codex => {}
    }
    settings.default_provider = Some(provider.to_string());
    config::save(&settings)?;
    Ok(model)
}

fn print_status() -> i32 {
    println!("OpenTerm provider status\n");
    for provider in [
        Provider::Codex,
        Provider::OpenAi,
        Provider::Claude,
        Provider::Mistral,
    ] {
        println!("  {provider:<8} {}", describe(provider));
    }
    if let Ok(path) = config::config_path() {
        println!("\nConfig: {}", path.display());
    }
    if let Some(default) = config::load().default_provider.as_deref() {
        println!("Default provider: {default}");
    }
    println!("\nRun ai-connect to connect with arrow-key selection.");
    0
}

fn describe(provider: Provider) -> String {
    if provider == Provider::Codex {
        if config::codex_connected() {
            let model = config::stored_model(provider)
                .map(|value| format!(", model {value}"))
                .unwrap_or_default();
            return format!("connected (openterm{model})");
        }
        if codex_auth::import_cli_tokens().is_some() {
            return "ready (codex CLI session)".to_owned();
        }
        return "not connected — run ai-connect".to_owned();
    }
    let env = config::env_var(provider).expect("api provider");
    if std::env::var_os(env).is_some() {
        return "configured (environment)".to_owned();
    }
    if config::is_stored(provider) {
        let model = config::stored_model(provider)
            .map(|value| format!(", model {value}"))
            .unwrap_or_default();
        return format!("connected (openterm{model})");
    }
    "not connected — run ai-connect".to_owned()
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
