use std::ffi::OsString;
use std::process::Command;

pub fn run(args: impl IntoIterator<Item = OsString>) -> i32 {
    launch(
        args,
        "Usage: ai-ui\n\nOpen OpenTerm appearance settings.\n",
        "ai-ui is available inside OpenTerm.app; rebuild with 'make app'",
    )
}

pub fn run_settings(args: impl IntoIterator<Item = OsString>) -> i32 {
    launch(
        args,
        "Usage: ai-settings\n\nOpen OpenTerm settings (appearance, AI, token status).\nAlso available as: openterm settings\n",
        "ai-settings is available inside OpenTerm.app; rebuild with 'make app'",
    )
}

pub fn spawn(binary: &str, args: impl IntoIterator<Item = OsString>, missing: &str) -> i32 {
    let executable = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.join(binary)));
    let Some(executable) = executable.filter(|path| path.is_file()) else {
        eprintln!("error: {missing}");
        return 127;
    };
    match Command::new(executable).args(args).spawn() {
        Ok(_) => 0,
        Err(error) => {
            eprintln!("error: could not open {binary}: {error}");
            1
        }
    }
}

fn launch(args: impl IntoIterator<Item = OsString>, help: &str, missing: &str) -> i32 {
    let args: Vec<_> = args.into_iter().collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print!("{help}");
        return 0;
    }
    spawn("openterm-ui", args, missing)
}
