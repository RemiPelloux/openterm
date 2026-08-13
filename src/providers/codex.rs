use super::AskRequest;
use std::process::Command;

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
