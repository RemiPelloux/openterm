use std::ffi::OsString;

const HELP: &str = "Usage: ai-workspace [--json]

Print read-only workspace context for agents (cwd, Git state).
";

pub fn run(args: impl IntoIterator<Item = OsString>) -> i32 {
    let args: Vec<_> = args.into_iter().collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print!("{HELP}");
        return 0;
    }
    if args.iter().any(|arg| arg == "--json") {
        return print_json();
    }
    if !args.is_empty() {
        eprintln!("error: unknown option\n\n{HELP}");
        return 2;
    }
    println!("{}", crate::agent_context::workspace_block());
    0
}

fn print_json() -> i32 {
    let cwd = std::env::current_dir().map_or_else(
        |_| ".".to_owned(),
        |path| path.to_string_lossy().into_owned(),
    );
    let git = crate::git_context::prompt_line();
    let payload = serde_json::json!({
        "cwd": cwd,
        "git": git,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&payload).unwrap_or_default()
    );
    0
}

#[cfg(test)]
mod tests {
    use super::run;
    use std::ffi::OsString;

    #[test]
    fn prints_workspace_block() {
        let code = run(Vec::<OsString>::new());
        assert_eq!(code, 0);
    }
}
