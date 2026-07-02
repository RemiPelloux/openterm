use std::ffi::{OsStr, OsString};
use std::path::Path;

const CORE_COMMANDS: [&str; 19] = [
    "ai-ask",
    "ai-explain",
    "ai-panel",
    "ai-workspace",
    "ai-doctor",
    "ai-version",
    "ai-result",
    "ai-expand",
    "ai-page",
    "ai-export",
    "ai-pin",
    "ai-forget",
    "ai-run",
    "ai-history",
    "ai-search-history",
    "ai-clean-history",
    "ai-usage",
    "ai-context",
    "ai-ui",
];
const HELP: &str = "OpenTerm — AI-native terminal foundation

Commands:
  ai-ask      Ask an LLM for read-only terminal guidance
  ai-explain  Explain stored command output with your model
  ai-panel    Open the agent side panel
  ai-workspace Print cwd + Git context for agents
  ai-doctor   Check required tools and provider credentials
  ai-version  Print the OpenTerm version
  ai-result   Inspect stored result metadata
  ai-expand   Retrieve exact stored command evidence
  ai-page     Read a bounded page of stored evidence
  ai-export   Export stored evidence to a file
  ai-pin      Protect a result from cleanup
  ai-forget   Move a result into recoverable trash
  ai-run      Execute one program and preserve its raw output
  ai-history  Browse recent managed commands
  ai-search-history Search command history locally
  ai-clean-history  Rank safe cleanup suggestions
  ai-usage    Show local evidence-store statistics
  ai-context  Show Git branch and repository state
  ai-ui       Open native appearance settings
";

pub fn run(program: &OsStr, args: impl IntoIterator<Item = OsString>) -> i32 {
    let command = executable_name(program);
    if let Some(spec) = crate::rtk_commands::find(&command) {
        return spec.run(args);
    }
    match command.as_str() {
        "ai-ask" => crate::ask::run(args),
        "ai-explain" => crate::explain::run(args),
        "ai-panel" => crate::panel_command::run(args),
        "ai-workspace" => crate::workspace::run(args),
        "ai-doctor" => crate::doctor::run(),
        "ai-version" => print_version(),
        "ai-context" => crate::git_context::run(args),
        "ai-ui" => crate::ui_command::run(args),
        "ai-run" => crate::run_command::run(args),
        name if crate::results::is_command(name) => crate::results::run(name, args),
        "openterm" => run_launcher(args),
        unknown => print_unknown(unknown),
    }
}

fn run_launcher(args: impl IntoIterator<Item = OsString>) -> i32 {
    let mut args = args.into_iter();
    match args.next() {
        None => crate::ui::show_cockpit(),
        Some(value) if matches!(value.to_str(), Some("help" | "--help" | "-h")) => print_help(),
        Some(value) if value == "mascot" => crate::ui::animate_mascot(),
        Some(value) if value == "--print-command-names" => print_command_names(),
        Some(value) => print_unknown(&value.to_string_lossy()),
    }
}

fn executable_name(program: &OsStr) -> String {
    Path::new(program)
        .file_name()
        .unwrap_or(program)
        .to_string_lossy()
        .to_string()
}

fn print_help() -> i32 {
    print!("{HELP}");
    for command in crate::rtk_commands::all() {
        println!("  {:<11} {}", command.name, command.description);
    }
    println!("\nEvery OpenTerm tool command starts with ai-.");
    0
}

fn print_command_names() -> i32 {
    for command in CORE_COMMANDS {
        println!("{command}");
    }
    for command in crate::rtk_commands::all() {
        println!("{}", command.name);
    }
    0
}

fn print_version() -> i32 {
    println!("openterm {}", env!("CARGO_PKG_VERSION"));
    0
}

fn print_unknown(command: &str) -> i32 {
    eprintln!("error: unknown OpenTerm command '{command}'\n\n{HELP}");
    2
}

#[cfg(test)]
mod tests {
    use super::{CORE_COMMANDS, run};
    use std::ffi::OsString;

    #[test]
    fn all_tool_commands_use_ai_prefix() {
        assert!(
            CORE_COMMANDS
                .iter()
                .all(|command| command.starts_with("ai-"))
        );
        assert!(
            crate::rtk_commands::all()
                .iter()
                .all(|command| command.name.starts_with("ai-"))
        );
    }

    #[test]
    fn launcher_rejects_unprefixed_command() {
        assert_eq!(run("openterm".as_ref(), [OsString::from("doctor")]), 2);
    }

    #[test]
    fn unknown_ai_command_returns_usage_error() {
        assert_eq!(run("ai-unknown".as_ref(), []), 2);
    }
}
