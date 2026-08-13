use crate::ask;
use std::ffi::OsString;
use std::io::{self, IsTerminal};
use std::path::PathBuf;
use std::process::Command;

const HELP: &str = "Usage: ai-panel

Open the agent panel inside this terminal. The shell keeps focus.

Type a question, or:
  :workspace   show cwd and Git context
  :gain        show RTK token savings
  q            quit
";

pub fn run(args: impl IntoIterator<Item = OsString>) -> i32 {
    match parse_args(args) {
        ParseOutcome::Help => {
            print!("{HELP}");
            0
        }
        ParseOutcome::Error(error) => print_error(&error, 2),
        ParseOutcome::Interactive => run_interactive(),
    }
}

enum ParseOutcome {
    Help,
    Interactive,
    Error(String),
}

enum Action {
    Quit,
    Workspace,
    Gain,
    Ask(String),
}

fn parse_args(args: impl IntoIterator<Item = OsString>) -> ParseOutcome {
    let Some(arg) = args.into_iter().next() else {
        return ParseOutcome::Interactive;
    };
    match arg.to_string_lossy().as_ref() {
        "--help" | "-h" => ParseOutcome::Help,
        flag if flag.starts_with('-') => ParseOutcome::Error(format!("unknown option '{flag}'")),
        _ => ParseOutcome::Error("ai-panel takes no arguments".to_owned()),
    }
}

fn run_interactive() -> i32 {
    if !io::stdin().is_terminal() {
        return print_error("ai-panel requires an interactive terminal", 2);
    }
    println!("OpenTerm agent panel  ·  this terminal  ·  q to quit");
    match run_loop() {
        Ok(()) => 0,
        Err(error) => print_error(&error, 1),
    }
}

fn run_loop() -> Result<(), String> {
    loop {
        let line = dialoguer::Input::<String>::new()
            .with_prompt("ask")
            .allow_empty(true)
            .interact_text()
            .map_err(|error| error.to_string())?;
        match parse_line(&line) {
            None => {}
            Some(Action::Quit) => return Ok(()),
            Some(action) => println!("{}", handle(&action)?),
        }
    }
}

fn parse_line(line: &str) -> Option<Action> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(match trimmed {
        "q" | "quit" | "exit" | ":q" => Action::Quit,
        ":workspace" | ":w" => Action::Workspace,
        ":gain" => Action::Gain,
        other => Action::Ask(other.to_owned()),
    })
}

fn handle(action: &Action) -> Result<String, String> {
    match action {
        Action::Quit => Ok(String::new()),
        Action::Workspace => Ok(crate::agent_context::workspace_block()),
        Action::Gain => run_named("ai-gain"),
        Action::Ask(question) => ask::ask_workspace(question),
    }
}

fn run_named(name: &str) -> Result<String, String> {
    let output = Command::new(command_path(name))
        .output()
        .map_err(|error| error.to_string())?;
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    if text.trim().is_empty() {
        text = String::from_utf8_lossy(&output.stderr).into_owned();
    }
    if text.trim().is_empty() {
        return Err(format!("{name} returned no output"));
    }
    Ok(text.trim_end().to_owned())
}

fn command_path(name: &str) -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.join(name)))
        .filter(|path| path.is_file())
        .unwrap_or_else(|| PathBuf::from(name))
}

fn print_error(error: &str, code: i32) -> i32 {
    eprintln!("error: {error}\n\n{HELP}");
    code
}

#[cfg(test)]
mod tests {
    use super::{Action, ParseOutcome, parse_args, parse_line};
    use std::ffi::OsString;

    #[test]
    fn help_flag_parses() {
        assert!(matches!(
            parse_args([OsString::from("--help")]),
            ParseOutcome::Help
        ));
    }

    #[test]
    fn rejects_positional_args() {
        assert!(matches!(
            parse_args([OsString::from("now")]),
            ParseOutcome::Error(_)
        ));
    }

    #[test]
    fn parse_line_commands() {
        assert!(matches!(parse_line("q"), Some(Action::Quit)));
        assert!(matches!(parse_line(":workspace"), Some(Action::Workspace)));
        assert!(matches!(parse_line(":gain"), Some(Action::Gain)));
        match parse_line("  df -h?  ") {
            Some(Action::Ask(question)) => assert_eq!(question, "df -h?"),
            _ => panic!("expected ask"),
        }
        assert!(parse_line("  ").is_none());
    }
}
