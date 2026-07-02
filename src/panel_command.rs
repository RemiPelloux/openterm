use std::ffi::OsString;
use std::process::Command;

const BINARY: &str = "openterm-panel";

pub fn run(args: impl IntoIterator<Item = OsString>) -> i32 {
    let args: Vec<_> = args.into_iter().collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print!("Usage: ai-panel\n\nOpen the OpenTerm agent side panel.\n");
        return 0;
    }
    if !args.is_empty() {
        eprintln!("error: ai-panel takes no arguments\n\nUsage: ai-panel");
        return 2;
    }
    let executable = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.join(BINARY)));
    let Some(executable) = executable.filter(|path| path.is_file()) else {
        eprintln!("error: ai-panel is available inside OpenTerm.app; rebuild with 'make app'");
        return 127;
    };
    match Command::new(executable).spawn() {
        Ok(_) => 0,
        Err(error) => {
            eprintln!("error: could not open agent panel: {error}");
            1
        }
    }
}
