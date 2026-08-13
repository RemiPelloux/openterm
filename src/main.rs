mod agent_context;
mod ask;
mod ask_apply;
mod ask_cache;
mod chat;
mod cli;
mod connect;
mod doctor;
mod explain;
mod git_context;
mod help;
mod help_catalog;
mod help_render;
mod help_render_footer;
mod hud;
mod markdown;
mod markdown_live;
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
