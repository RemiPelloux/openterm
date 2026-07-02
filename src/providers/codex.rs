use super::AskRequest;
use std::process::Command;

const SAFETY_PROMPT: &str = "You are the read-only OpenTerm ai-ask assistant. Answer the user's terminal question concisely. You may inspect the current workspace, but do not modify files or execute commands with side effects. If asked to change or organize something, return a proposed plan and exact commands without applying them.";

pub fn ask(request: &AskRequest) -> Result<String, String> {
    let prompt = format!("{SAFETY_PROMPT}\n\nUser question: {}", request.prompt);
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
    let output = command
        .arg(prompt)
        .output()
        .map_err(|error| codex_error(&error))?;
    parse_output(&output)
}

fn parse_output(output: &std::process::Output) -> Result<String, String> {
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("codex failed: {}", error.trim()));
    }
    let answer = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    (!answer.is_empty())
        .then_some(answer)
        .ok_or_else(|| "codex returned no text".to_owned())
}

fn codex_error(error: &std::io::Error) -> String {
    format!("could not start codex: {error}")
}
