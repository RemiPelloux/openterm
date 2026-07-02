use std::ffi::OsString;
use std::io::Write;
use std::process::Command;
use std::time::Instant;

pub fn run(args: impl IntoIterator<Item = OsString>) -> i32 {
    let args = args.into_iter().collect::<Vec<_>>();
    let Some(program) = args.first() else {
        eprintln!("error: usage: ai-run PROGRAM [ARG ...]");
        return 2;
    };
    let started = Instant::now();
    match Command::new(program).args(&args[1..]).output() {
        Ok(output) => finish(&args, started, &output),
        Err(error) => spawn_error(program, &error),
    }
}

fn finish(args: &[OsString], started: Instant, output: &std::process::Output) -> i32 {
    let _ = std::io::stdout().write_all(&output.stdout);
    let _ = std::io::stderr().write_all(&output.stderr);
    let code = crate::rtk_commands::output_exit_code(output);
    let request = crate::results::SaveRequest::raw(args, started, output);
    crate::results::save_and_announce(&request);
    code
}

fn spawn_error(program: &OsString, error: &std::io::Error) -> i32 {
    if error.kind() == std::io::ErrorKind::NotFound {
        eprintln!("error: '{}' was not found", program.to_string_lossy());
        return 127;
    }
    eprintln!(
        "error: could not run '{}': {error}",
        program.to_string_lossy()
    );
    1
}
