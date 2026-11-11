use crate::providers::config;
use std::process::Command;

const REQUIRED_COMMANDS: [(&str, &[&str]); 3] = [
    ("git", &["--version"]),
    ("rtk", &["--version"]),
    ("codex", &["--version"]),
];
pub fn run() -> i32 {
    println!("OpenTerm doctor");
    let mut healthy = true;

    for (command, args) in REQUIRED_COMMANDS {
        match command_version(command, args) {
            Ok(version) => println!("  ok  {command}: {version}"),
            Err(error) => {
                eprintln!("  err {command}: {error}");
                healthy = false;
            }
        }
    }

    print_credentials();
    print_local_runtimes();
    if let Ok(path) = config::config_path() {
        if path.exists() {
            println!("  cfg providers: {}", path.display());
        }
    }
    i32::from(!healthy)
}

fn print_credentials() {
    let settings = config::load();
    for provider in crate::providers::Provider::ALL {
        let Some(variable) = config::env_var(provider) else {
            continue;
        };
        let state = credential_state(settings.key_source(provider));
        println!("  api {provider}: {state} ({variable})");
    }
}

fn print_local_runtimes() {
    for provider in [
        crate::providers::Provider::Ollama,
        crate::providers::Provider::LmStudio,
    ] {
        let host = crate::providers::local_base_url(provider);
        let models = crate::providers::local_models(provider);
        let state = if models.is_empty() {
            "not running".to_owned()
        } else {
            format!("reachable, {} model(s)", models.len())
        };
        println!("  loc {provider}: {state} ({host})");
    }
}

/// Names the source only; the key itself is never read into output.
fn credential_state(source: config::KeySource) -> String {
    if source == config::KeySource::None {
        return "not configured".to_owned();
    }
    format!("configured ({})", source.as_str())
}

fn command_version(command: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(command)
        .args(args)
        .output()
        .map_err(|error| error.to_string())?;

    if !output.status.success() {
        return Err(format!("exited with {}", output.status));
    }

    let version = String::from_utf8_lossy(&output.stdout);
    Ok(version.trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::{command_version, credential_state};
    use crate::providers::config::KeySource;

    #[test]
    fn credential_state_names_the_source() {
        assert_eq!(
            credential_state(KeySource::Keychain),
            "configured (keychain)"
        );
        assert_eq!(credential_state(KeySource::None), "not configured");
    }

    #[test]
    fn missing_command_is_reported() {
        let result = command_version("openterm-command-that-does-not-exist", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn successful_command_returns_trimmed_output() {
        let result = command_version("printf", &["  ready  "]);
        assert_eq!(result.as_deref(), Ok("ready"));
    }
}
