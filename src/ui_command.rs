use std::ffi::OsString;
use std::process::Command;

pub fn run(args: impl IntoIterator<Item = OsString>) -> i32 {
    let args = args.into_iter().collect::<Vec<_>>();
    let executable = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.join("openterm-ui")));
    let Some(executable) = executable.filter(|path| path.is_file()) else {
        eprintln!("error: ai-ui is available inside OpenTerm.app; rebuild with 'make app'");
        return 127;
    };
    match Command::new(executable).args(args).spawn() {
        Ok(_) => 0,
        Err(error) => {
            eprintln!("error: could not open OpenTermUI: {error}");
            1
        }
    }
}
