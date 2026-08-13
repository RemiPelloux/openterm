use std::ffi::{OsStr, OsString};
use std::path::Path;

const CORE_COMMANDS: [&str; 23] = [
    "ai-help",
    "ai-ask",
    "ai-explain",
    "ai-panel",
    "ai-workspace",
    "ai-connect",
    "ai-doctor",
    "ai-version",
    "ai-result",
    "ai-expand",
    "ai-page",
    "ai-export",
    "ai-pin",
    "ai-forget",
    "ai-restore",
    "ai-run",
    "ai-history",
    "ai-search-history",
    "ai-clean-history",
    "ai-usage",
    "ai-context",
    "ai-ui",
    "ai-settings",
];

pub fn run(program: &OsStr, args: impl IntoIterator<Item = OsString>) -> i32 {
    let command = executable_name(program);
    if let Some(spec) = crate::rtk_commands::find(&command) {
        return spec.run(args);
    }
    match command.as_str() {
        "ai-help" => crate::help::run(args),
        "ai-ask" => crate::ask::run(args),
        "ai-explain" => crate::explain::run(args),
        "ai-panel" => crate::panel_command::run(args),
        "ai-workspace" => crate::workspace::run(args),
        "ai-connect" => crate::connect::run(args),
        "ai-doctor" => crate::doctor::run(),
        "ai-version" => print_version(),
        "ai-context" => crate::git_context::run(args),
        "ai-ui" => crate::ui_command::run(args),
        "ai-settings" => crate::ui_command::run_settings(args),
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
        Some(value) if value == "settings" => {
            crate::ui_command::run_settings(std::iter::empty::<OsString>())
        }
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
    crate::help::print_summary();
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
    eprintln!("error: unknown OpenTerm command '{command}'\n");
    crate::help::print_summary();
    2
}

#[cfg(test)]
mod tests {
    use super::{CORE_COMMANDS, run};
    use std::collections::HashSet;
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
    fn core_commands_match_help_catalog() {
        let commands = CORE_COMMANDS.into_iter().collect::<HashSet<_>>();
        let documented = crate::help_catalog::SECTIONS
            .iter()
            .flat_map(|section| section.entries.iter())
            .map(|entry| entry.name)
            .collect::<HashSet<_>>();

        assert_eq!(commands, documented);
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
