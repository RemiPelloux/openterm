//! Claude through the installed `claude` CLI, so a Claude Pro/Max plan works
//! without this program ever handling the CLI's login tokens.
//! `--bare` is deliberately not passed: it skips the CLI's own login.

use super::anthropic_events::MessageStream;
use super::{AskRequest, Provider};
use serde_json::Value;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, Command, Stdio};
use std::thread::JoinHandle;

const PROGRAM: &str = "claude";
const TEXT_OUTPUT: &[&str] = &["--output-format", "text"];
const STREAM_OUTPUT: &[&str] = &[
    "--output-format",
    "stream-json",
    "--include-partial-messages",
    "--verbose",
];

pub fn ask(request: &AskRequest) -> Result<String, String> {
    let child = spawn(&mut command(request, TEXT_OUTPUT), &request.prompt)?;
    let output = child
        .wait_with_output()
        .map_err(|error| format!("{PROGRAM} failed: {error}"))?;
    parse_output(&output)
}

/// Streams text from the CLI's NDJSON `stream_event` lines as they arrive.
pub fn ask_stream(request: &AskRequest, on_delta: &mut dyn FnMut(&str)) -> Result<String, String> {
    let mut child = spawn(&mut command(request, STREAM_OUTPUT), &request.prompt)?;
    let stderr = drain(child.stderr.take());
    let mut stream = MessageStream::new(Provider::ClaudeCode, on_delta);
    let read = match child.stdout.take() {
        Some(stdout) => read_lines(BufReader::new(stdout), &mut stream),
        None => Err(format!("{PROGRAM} stdout unavailable")),
    };
    finish_child(child, stderr, read)?;
    stream.finish()
}

/// True once `claude auth status` reports a usable login.
pub fn session_ready() -> bool {
    Command::new(PROGRAM)
        .args(["auth", "status"])
        .output()
        .is_ok_and(|output| output.status.success())
}

fn command(request: &AskRequest, output: &[&str]) -> Command {
    let mut command = Command::new(PROGRAM);
    command.arg("--print").args(output).args([
        "--no-session-persistence",
        "--tools",
        "",
        "--append-system-prompt",
        super::ask_prompt::INSTRUCTIONS,
    ]);
    if let Some(model) = &request.model {
        command.args(["--model", model]);
    }
    command
}

/// Starts the CLI with piped streams and hands it the prompt on stdin.
fn spawn(command: &mut Command, prompt: &str) -> Result<Child, String> {
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
    Ok(child)
}

/// Reads stderr on its own thread so a chatty CLI cannot block stdout.
fn drain(stderr: Option<impl Read + Send + 'static>) -> JoinHandle<String> {
    std::thread::spawn(move || {
        let mut text = String::new();
        if let Some(mut stderr) = stderr {
            let _ = stderr.read_to_string(&mut text);
        }
        text
    })
}

fn read_lines(stdout: impl BufRead, stream: &mut MessageStream) -> Result<(), String> {
    for line in stdout.lines() {
        let line = line.map_err(|error| format!("{PROGRAM} output failed: {error}"))?;
        if let Ok(value) = serde_json::from_str::<Value>(&line) {
            take_line(&value, stream)?;
        }
    }
    Ok(())
}

fn take_line(value: &Value, stream: &mut MessageStream) -> Result<(), String> {
    match value["type"].as_str() {
        Some("stream_event") => stream.take(&value["event"]),
        Some("result") if value["is_error"] == true => {
            let message = value["result"].as_str().unwrap_or("unknown error");
            Err(format!("claude-code failed: {message}"))
        }
        _ => Ok(()),
    }
}

/// The CLI's own error line wins; stderr explains other non-zero exits.
fn finish_child(
    mut child: Child,
    stderr: JoinHandle<String>,
    read: Result<(), String>,
) -> Result<(), String> {
    if read.is_err() {
        let _ = child.kill();
    }
    let status = child
        .wait()
        .map_err(|error| format!("{PROGRAM} failed: {error}"))?;
    let errors = stderr.join().unwrap_or_default();
    read?;
    if status.success() {
        return Ok(());
    }
    Err(format!("claude-code failed: {}", errors.trim()))
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
    use super::{MessageStream, Provider, parse_output, read_lines};
    use std::os::unix::process::ExitStatusExt;
    use std::process::{ExitStatus, Output};

    fn output(code: i32, stdout: &str, stderr: &str) -> Output {
        Output {
            status: ExitStatus::from_raw(code << 8),
            stdout: stdout.as_bytes().to_vec(),
            stderr: stderr.as_bytes().to_vec(),
        }
    }

    fn stream(ndjson: &str) -> Result<String, String> {
        let mut on_delta = |_: &str| {};
        let mut stream = MessageStream::new(Provider::ClaudeCode, &mut on_delta);
        read_lines(ndjson.as_bytes(), &mut stream)?;
        stream.finish()
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

    #[test]
    fn streams_partial_text_deltas() {
        let ndjson = [
            r#"{"type":"system","subtype":"init"}"#,
            r#"{"type":"stream_event","event":{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"po"}}}"#,
            r#"{"type":"stream_event","event":{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"ng"}}}"#,
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"pong"}]}}"#,
            r#"{"type":"result","subtype":"success","is_error":false,"result":"pong"}"#,
        ]
        .join("\n");
        assert_eq!(stream(&ndjson).as_deref(), Ok("pong"));
    }

    #[test]
    fn reports_error_result_line() {
        let ndjson =
            r#"{"type":"result","subtype":"success","is_error":true,"result":"bad model"}"#;
        assert_eq!(stream(ndjson).unwrap_err(), "claude-code failed: bad model");
    }
}
