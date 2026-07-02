use std::process::Command;

const REQUIRED_COMMANDS: [(&str, &[&str]); 3] = [
    ("git", &["--version"]),
    ("rtk", &["--version"]),
    ("codex", &["--version"]),
];
const API_KEYS: [(&str, &str); 3] = [
    ("openai", "OPENAI_API_KEY"),
    ("claude", "ANTHROPIC_API_KEY"),
    ("mistral", "MISTRAL_API_KEY"),
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
    i32::from(!healthy)
}

fn print_credentials() {
    for (provider, variable) in API_KEYS {
        let state = if std::env::var_os(variable).is_some() {
            "configured"
        } else {
            "not configured"
        };
        println!("  api {provider}: {state} ({variable})");
    }
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
    use super::command_version;

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
