use crate::ui_command;
use std::ffi::OsString;

const HELP: &str = "Usage: ai-panel

Open the agent side panel (Ask, Workspace, Gain) beside the terminal.
The panel does not steal keyboard focus from the shell.
";

pub fn run(args: impl IntoIterator<Item = OsString>) -> i32 {
    let args: Vec<_> = args.into_iter().collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print!("{HELP}");
        return 0;
    }
    if !args.is_empty() {
        eprintln!("error: ai-panel takes no arguments\n\n{HELP}");
        return 2;
    }
    ui_command::spawn(
        "openterm-panel",
        None::<OsString>,
        "ai-panel is available inside OpenTerm.app; rebuild with 'make app'",
    )
}

#[cfg(test)]
mod tests {
    use super::run;
    use std::ffi::OsString;

    #[test]
    fn help_flag_prints_side_panel() {
        assert_eq!(run([OsString::from("--help")]), 0);
    }

    #[test]
    fn rejects_positional_args() {
        assert_eq!(run([OsString::from("now")]), 2);
    }
}
