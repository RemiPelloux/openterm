mod agent_context;
mod ask;
mod cli;
mod doctor;
mod explain;
mod git_context;
mod panel_command;
mod providers;
mod results;
mod rtk_commands;
mod run_command;
mod ui;
mod ui_command;
mod workspace;

fn main() {
    let mut args = std::env::args_os();
    let program = args.next().unwrap_or_else(|| "openterm".into());
    let exit_code = cli::run(&program, args);
    std::process::exit(exit_code);
}
