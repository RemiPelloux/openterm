use super::AskRequest;
use std::process::{Command, Stdio};

pub fn ask(request: &AskRequest) -> Result<String, String> {
    let prompt = format!(
        "{}\n\nUser question: {}",
        super::ask_prompt::INSTRUCTIONS,
        request.prompt
    );
    let mut command = Command::new("codex");
    command.args([
        "exec",
        "--ignore-user-config",
        "--ephemeral",
        "--sandbox",
        "read-only",
        "--skip-git-repo-check",
        "--color",
        "never",
    ]);
    if let Some(model) = &request.model {
        command.args(["--model", model]);
    }
    // The prompt is an argument; an inherited non-TTY stdin would make
    // `codex exec` wait to read more input (e.g. from the macOS panel).
    let output = command
        .arg(prompt)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| codex_error(&error))?;
    parse_output(&output)
}

/// True once `codex login status` reports a usable login.
pub fn session_ready() -> bool {
    Command::new("codex")
        .args(["login", "status"])
        .output()
        .is_ok_and(|output| output.status.success())
}

const STDIN_NOTICE: &str = "Reading additional input from stdin";
const UNAUTHORIZED: &str = "401 Unauthorized";

fn parse_output(output: &std::process::Output) -> Result<String, String> {
    if !output.status.success() {
        return Err(failure(&String::from_utf8_lossy(&output.stderr)));
    }
    let answer = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    (!answer.is_empty())
        .then_some(answer)
        .ok_or_else(|| "codex returned no text".to_owned())
}

/// Drops codex's stdin notice and turns a rejected login into a next step.
fn failure(stderr: &str) -> String {
    if stderr.contains(UNAUTHORIZED) {
        return "codex login was rejected (401 Unauthorized); run 'codex login' and \
                choose Sign in with ChatGPT to use your subscription"
            .to_owned();
    }
    let detail = stderr
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with(STDIN_NOTICE))
        .collect::<Vec<_>>()
        .join("\n");
    format!("codex failed: {detail}")
}

fn codex_error(error: &std::io::Error) -> String {
    format!("could not start codex: {error}")
}

#[cfg(test)]
mod tests {
    use super::failure;

    #[test]
    fn rejected_login_points_to_codex_login() {
        let stderr = "Reading additional input from stdin...\nERROR x: 401 Unauthorized: bad key";
        let message = failure(stderr);
        assert!(message.contains("run 'codex login'"));
        assert!(!message.contains("bad key"), "never echo key details");
    }

    #[test]
    fn stdin_notice_is_not_reported_as_the_error() {
        let message = failure("Reading additional input from stdin...\nmodel not found");
        assert_eq!(message, "codex failed: model not found");
    }
}
