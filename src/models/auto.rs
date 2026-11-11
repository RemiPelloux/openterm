//! Keeps model lists fresh without making anyone wait for them.

use super::cache;
use crate::providers::AskRequest;

/// Called from `ai-ask` / `ai-explain`: when this provider's cached list is
/// stale or missing, refresh it in a detached child so the next model picker
/// is current and this answer is not delayed.
pub fn refresh_if_stale(request: &AskRequest) {
    let provider = request.provider;
    if provider.is_cli() {
        return;
    }
    if cache::load().is_stale(provider, &request.endpoint.base_url, cache::now_ms()) {
        spawn_refresh(&provider.to_string());
    }
}

/// The binary dispatches on `argv[0]`, so the child is named `ai-models`
/// and put in its own process group to survive Ctrl-C on the parent.
#[cfg(unix)]
fn spawn_refresh(provider: &str) {
    use std::os::unix::process::CommandExt;
    use std::process::{Command, Stdio};
    const COMMAND: &str = "ai-models";
    let Ok(program) = std::env::current_exe() else {
        return;
    };
    let _ = Command::new(program)
        .arg0(COMMAND)
        .args([provider, "--refresh", "--quiet"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn();
}

#[cfg(not(unix))]
fn spawn_refresh(_provider: &str) {}
