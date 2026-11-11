use crate::ask;
use crate::ui;
use dialoguer::Input;
use dialoguer::theme::Theme;
use std::ffi::OsString;
use std::fmt;
use std::io::{self, IsTerminal, Write};

const HELP: &str = "Usage: ai-chat

In-terminal mascot chat. Read-only with workspace context.

  q leave · :clear redraw · :hud status · :help
";

const CYAN: &str = "\x1b[38;5;81m";
const TEAL: &str = "\x1b[38;5;45m";
const MUTED: &str = "\x1b[38;5;238m";
const TEXT: &str = "\x1b[38;5;252m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

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
    Clear,
    Hud,
    Help,
    Ask(String),
}

struct ChatTheme {
    color: bool,
}

impl Theme for ChatTheme {
    fn format_input_prompt(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        _default: Option<&str>,
    ) -> fmt::Result {
        write!(
            f,
            "  {}{prompt}{}  {}❯{} ",
            paint(self.color, MUTED),
            paint(self.color, RESET),
            paint(self.color, CYAN),
            paint(self.color, RESET)
        )
    }

    fn format_input_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        sel: &str,
    ) -> fmt::Result {
        write!(
            f,
            "  {}{prompt}{}  {}{sel}{}",
            paint(self.color, MUTED),
            paint(self.color, RESET),
            paint(self.color, TEXT),
            paint(self.color, RESET)
        )
    }
}

fn parse_args(args: impl IntoIterator<Item = OsString>) -> ParseOutcome {
    let Some(arg) = args.into_iter().next() else {
        return ParseOutcome::Interactive;
    };
    match arg.to_string_lossy().as_ref() {
        "--help" | "-h" => ParseOutcome::Help,
        flag if flag.starts_with('-') => ParseOutcome::Error(format!("unknown option '{flag}'")),
        _ => ParseOutcome::Error("ai-chat takes no arguments".to_owned()),
    }
}

fn run_interactive() -> i32 {
    if !io::stdin().is_terminal() {
        return print_error("ai-chat requires an interactive terminal", 2);
    }
    crate::hud::sync_from_config(&crate::providers::config::load());
    print_banner();
    match run_loop() {
        Ok(()) => 0,
        Err(error) => print_error(&error, 1),
    }
}

fn run_loop() -> Result<(), String> {
    let theme = ChatTheme {
        color: color_enabled(),
    };
    loop {
        let line = Input::<String>::with_theme(&theme)
            .with_prompt("you")
            .allow_empty(true)
            .interact_text()
            .map_err(|error| error.to_string())?;
        match parse_line(&line) {
            None => {}
            Some(Action::Quit) => return Ok(()),
            Some(Action::Clear) => {
                print!("\x1b[2J\x1b[H");
                print_banner();
            }
            Some(Action::Hud) => println!("{}", paint_status()),
            Some(Action::Help) => print!("{HELP}"),
            Some(Action::Ask(question)) => ask_turn(&question)?,
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
        ":clear" => Action::Clear,
        ":hud" => Action::Hud,
        ":help" | "-h" | "--help" => Action::Help,
        other => Action::Ask(other.to_owned()),
    })
}

fn ask_turn(question: &str) -> Result<(), String> {
    print_thinking();
    let answer = ask::ask_text(question, true)?;
    print!("\r\x1b[K");
    let color = color_enabled();
    println!("  {}▐●‿●▌{}", paint(color, TEAL), paint(color, RESET));
    if ask::present(&answer) != 0 {
        return Err("ask failed".to_owned());
    }
    Ok(())
}

fn print_banner() {
    for line in banner_lines(&crate::hud::current_line(), color_enabled()) {
        println!("{line}");
    }
}

fn banner_lines(status: &str, color: bool) -> Vec<String> {
    let mut lines = vec![String::new(), box_head(color)];
    for (art, note) in ui::mascot_art().iter().zip(notes(status, color)) {
        lines.push(box_row(color, art, &note));
    }
    lines.push(box_foot(color));
    lines.push(String::new());
    lines
}

fn notes(status: &str, color: bool) -> [String; 7] {
    [
        format!(
            "{}{}OpenTerm chat{}",
            paint(color, BOLD),
            paint(color, TEAL),
            paint(color, RESET)
        ),
        format!("{}{status}{}", paint(color, CYAN), paint(color, RESET)),
        tint(color, MUTED, "workspace on · OpenCode apply"),
        tint(color, MUTED, "q  ·  :clear  ·  :hud  ·  :help"),
        String::new(),
        String::new(),
        String::new(),
    ]
}

fn box_head(color: bool) -> String {
    format!(
        "{}╭─{} {}◆{} {}OpenTerm{}",
        paint(color, MUTED),
        paint(color, RESET),
        paint(color, TEAL),
        paint(color, RESET),
        paint(color, CYAN),
        paint(color, RESET)
    )
}

fn box_row(color: bool, art: &str, note: &str) -> String {
    format!(
        "{}│{} {}{art}{}  {note}",
        paint(color, MUTED),
        paint(color, RESET),
        paint(color, CYAN),
        paint(color, RESET)
    )
}

fn box_foot(color: bool) -> String {
    format!("{}╰─{}", paint(color, MUTED), paint(color, RESET))
}

fn print_thinking() {
    print!("{}\r", mascot_line(MUTED, "thinking…"));
    let _ = io::stdout().flush();
}

fn paint_status() -> String {
    mascot_line(CYAN, &crate::hud::current_line())
}

fn mascot_line(body_color: &'static str, body: &str) -> String {
    let color = color_enabled();
    format!(
        "  {}▐●‿●▌{}  {}{body}{}",
        paint(color, TEAL),
        paint(color, RESET),
        paint(color, body_color),
        paint(color, RESET)
    )
}

fn tint(color: bool, code: &'static str, text: &str) -> String {
    format!("{}{text}{}", paint(color, code), paint(color, RESET))
}

fn color_enabled() -> bool {
    io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none()
}

fn paint(enabled: bool, value: &'static str) -> &'static str {
    if enabled { value } else { "" }
}

fn print_error(error: &str, code: i32) -> i32 {
    eprintln!("error: {error}\n\n{HELP}");
    code
}

#[cfg(test)]
mod tests {
    use super::{Action, banner_lines, parse_line};

    #[test]
    fn parse_line_commands() {
        assert!(matches!(parse_line("q"), Some(Action::Quit)));
        assert!(matches!(parse_line(":clear"), Some(Action::Clear)));
        assert!(matches!(parse_line(":hud"), Some(Action::Hud)));
        assert!(matches!(parse_line(":help"), Some(Action::Help)));
        match parse_line("  list disks  ") {
            Some(Action::Ask(question)) => assert_eq!(question, "list disks"),
            _ => panic!("expected ask"),
        }
        assert!(parse_line("").is_none());
    }

    #[test]
    fn banner_includes_canonical_mascot() {
        let text = banner_lines("openai gpt-5.6-terra · 0 tok · 0 cached · idle", false).join("\n");
        assert!(text.contains("▗▄▄▄▄▄██▄▄▄▄▄▖"));
        assert!(text.contains("●    ●"));
        assert!(text.contains("OpenTerm chat"));
        assert!(text.contains("openai gpt-5.6-terra"));
        assert!(!text.contains("\x1b["));
    }
}
