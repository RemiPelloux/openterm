//! Claude through the installed `claude` CLI, so a Claude Pro/Max plan works
//! without this program ever handling the CLI's login tokens.
//! `--bare` is deliberately not passed: it skips the CLI's own login.

use super::AskRequest;
use std::io::Write;
use std::process::{Command, Stdio};

const PROGRAM: &str = "claude";

pub fn ask(request: &AskRequest) -> Result<String, String> {
    let mut command = Command::new(PROGRAM);
    command.args([
        "--print",
        "--output-format",
        "text",
        "--no-session-persistence",
        "--tools",
        "",
        "--append-system-prompt",
        super::ask_prompt::INSTRUCTIONS,
    ]);
    if let Some(model) = &request.model {
        command.args(["--model", model]);
    }
    let output = run_with_stdin(&mut command, &request.prompt)?;
    parse_output(&output)
}

/// True once `claude auth status` reports a usable login.
pub fn session_ready() -> bool {
    Command::new(PROGRAM)
        .args(["auth", "status"])
        .output()
        .is_ok_and(|output| output.status.success())
}

fn run_with_stdin(command: &mut Command, prompt: &str) -> Result<std::process::Output, String> {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("could not start {PROGRAM}: {error}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(prompt.as_bytes())
            .map_err(|error| format!("{PROGRAM} stdin failed: {error}"))?;
    }
    child
        .wait_with_output()
        .map_err(|error| format!("{PROGRAM} failed: {error}"))
}

fn parse_output(output: &std::process::Output) -> Result<String, String> {
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("claude-code failed: {}", error.trim()));
    }
    let answer = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    (!answer.is_empty())
        .then_some(answer)
        .ok_or_else(|| "claude-code returned no text".to_owned())
}

#[cfg(all(test, unix))]
mod tests {
    use super::parse_output;
    use std::os::unix::process::ExitStatusExt;
    use std::process::{ExitStatus, Output};

    fn output(code: i32, stdout: &str, stderr: &str) -> Output {
        Output {
            status: ExitStatus::from_raw(code << 8),
            stdout: stdout.as_bytes().to_vec(),
            stderr: stderr.as_bytes().to_vec(),
        }
    }

    #[test]
    fn returns_trimmed_answer() {
        assert_eq!(parse_output(&output(0, " ok \n", "")).as_deref(), Ok("ok"));
    }

    #[test]
    fn reports_cli_failure_and_empty_output() {
        let failed = parse_output(&output(1, "", "not logged in")).unwrap_err();
        assert!(failed.contains("not logged in"));
        assert!(parse_output(&output(0, "  ", "")).is_err());
    }
}
