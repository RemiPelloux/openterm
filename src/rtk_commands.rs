use std::ffi::OsString;
use std::process::{Command, ExitStatus};
use std::time::Instant;

pub struct RtkCommand {
    pub name: &'static str,
    pub description: &'static str,
    pub(crate) prefix: &'static [&'static str],
}

const COMMANDS: [RtkCommand; 19] = [
    RtkCommand::new(
        "ai-rg",
        "Fast ripgrep search (prefer over ai-grep)",
        &["rg"],
    ),
    RtkCommand::new("ai-grep", "Grouped grep (legacy; prefer ai-rg)", &["grep"]),
    RtkCommand::new("ai-find", "Compact file discovery", &["find"]),
    RtkCommand::new("ai-tree", "Token-efficient directory tree", &["tree"]),
    RtkCommand::new("ai-read", "Bounded intelligent file reading", &["read"]),
    RtkCommand::new("ai-ls", "Compact directory listing", &["ls"]),
    RtkCommand::new("ai-json", "Compact JSON output", &["json"]),
    RtkCommand::new("ai-wc", "Compact word and line counts", &["wc"]),
    RtkCommand::new("ai-err", "Errors and warnings only", &["err"]),
    RtkCommand::new("ai-summary", "Heuristic command summary", &["summary"]),
    RtkCommand::new("ai-smart", "Two-line technical summary", &["smart"]),
    RtkCommand::new("ai-env", "Filtered environment variables", &["env"]),
    RtkCommand::new("ai-deps", "Project dependency summary", &["deps"]),
    RtkCommand::new("ai-gain", "Token savings statistics", &["gain"]),
    RtkCommand::new("ai-status", "Compact Git status", &["git", "status"]),
    RtkCommand::new("ai-diff", "Condensed Git diff", &["git", "diff"]),
    RtkCommand::new("ai-test", "Failure-focused test execution", &["test"]),
    RtkCommand::new("ai-log", "Deduplicated log inspection", &["log"]),
    RtkCommand::new("ai-docker", "Compact Docker output", &["docker"]),
];

impl RtkCommand {
    const fn new(
        name: &'static str,
        description: &'static str,
        prefix: &'static [&'static str],
    ) -> Self {
        Self {
            name,
            description,
            prefix,
        }
    }

    pub fn run(&self, args: impl IntoIterator<Item = OsString>) -> i32 {
        let args = args.into_iter().collect::<Vec<_>>();
        run_managed(self, &args)
    }
}

fn run_managed(spec: &RtkCommand, args: &[OsString]) -> i32 {
    let started = Instant::now();
    let output = Command::new("rtk").args(spec.prefix).args(args).output();
    match output {
        Ok(output) => {
            print_output(&output.stdout, &output.stderr);
            let code = exit_code(output.status);
            let request = crate::results::SaveRequest::rtk(spec, args, started, &output);
            crate::results::save_and_announce(&request);
            code
        }
        Err(error) => print_spawn_error("rtk", &error),
    }
}

fn print_output(stdout: &[u8], stderr: &[u8]) {
    use std::io::Write;
    let _ = std::io::stdout().write_all(stdout);
    let _ = std::io::stderr().write_all(stderr);
}

fn print_spawn_error(binary: &str, error: &std::io::Error) -> i32 {
    if error.kind() == std::io::ErrorKind::NotFound {
        eprintln!("error: '{binary}' is required but was not found");
        return 127;
    }
    eprintln!("error: could not run '{binary}': {error}");
    1
}

pub fn all() -> &'static [RtkCommand] {
    &COMMANDS
}

pub fn find(name: &str) -> Option<&'static RtkCommand> {
    COMMANDS.iter().find(|command| command.name == name)
}

#[cfg(test)]
fn run_program(binary: &str, prefix: &[&str], args: impl IntoIterator<Item = OsString>) -> i32 {
    match Command::new(binary).args(prefix).args(args).status() {
        Ok(status) => exit_code(status),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("error: '{binary}' is required but was not found");
            127
        }
        Err(error) => {
            eprintln!("error: could not run '{binary}': {error}");
            1
        }
    }
}

fn exit_code(status: ExitStatus) -> i32 {
    if let Some(code) = status.code() {
        return code;
    }
    signal_exit_code(status)
}

pub(crate) fn output_exit_code(output: &std::process::Output) -> i32 {
    exit_code(output.status)
}

#[cfg(unix)]
fn signal_exit_code(status: ExitStatus) -> i32 {
    use std::os::unix::process::ExitStatusExt;
    status.signal().map_or(1, |signal| 128 + signal)
}

#[cfg(not(unix))]
const fn signal_exit_code(_status: ExitStatus) -> i32 {
    1
}

#[cfg(test)]
mod tests {
    use super::{find, run_program};
    use std::ffi::OsString;

    #[test]
    fn maps_git_status_to_rtk_subcommand() {
        assert_eq!(
            find("ai-status").expect("command").prefix,
            ["git", "status"]
        );
    }

    #[test]
    fn unknown_command_has_no_mapping() {
        assert!(find("ai-missing").is_none());
    }

    #[cfg(unix)]
    #[test]
    fn preserves_success_and_failure_codes() {
        let args = Vec::<OsString>::new();
        assert_eq!(run_program("true", &[], args.clone()), 0);
        assert_eq!(run_program("false", &[], args), 1);
    }

    #[test]
    fn missing_program_uses_command_not_found_code() {
        let code = run_program("openterm-missing-program", &[], Vec::<OsString>::new());
        assert_eq!(code, 127);
    }
}
