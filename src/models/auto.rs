//! Keeps model lists fresh without making anyone wait for them.

use super::cache;
use crate::providers::AskRequest;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Minimum gap between background refresh attempts per provider, so a
/// provider that keeps failing (offline, bad key) is not hammered on every ask.
const RETRY_AFTER: Duration = Duration::from_secs(15 * 60);

/// Called from `ai-ask` / `ai-explain`: when this provider's cached list is
/// stale or missing, refresh it in a detached child so the next model picker
/// is current and this answer is not delayed.
pub fn refresh_if_stale(request: &AskRequest) {
    let provider = request.provider;
    if provider.is_cli() {
        return;
    }
    if !cache::load().is_stale(provider, &request.endpoint.base_url, cache::now_ms()) {
        return;
    }
    let Some(marker) = marker_path(&provider.to_string()) else {
        return;
    };
    if attempted_recently(&marker, SystemTime::now()) {
        return;
    }
    let _ = std::fs::write(&marker, b"");
    spawn_refresh(&provider.to_string());
}

fn marker_path(provider: &str) -> Option<PathBuf> {
    let dir = crate::results::data_root().ok()?.join("model-refresh");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir.join(provider))
}

fn attempted_recently(marker: &Path, now: SystemTime) -> bool {
    std::fs::metadata(marker)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|modified| now.duration_since(modified).ok())
        .is_some_and(|age| age < RETRY_AFTER)
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

#[cfg(test)]
mod tests {
    use super::{RETRY_AFTER, attempted_recently};
    use std::time::{Duration, SystemTime};

    #[test]
    fn backs_off_only_within_the_retry_window() {
        let marker = std::env::temp_dir().join(format!("openterm-refresh-{}", std::process::id()));
        let now = SystemTime::now();
        assert!(
            !attempted_recently(&marker, now),
            "missing marker never backs off"
        );
        std::fs::write(&marker, b"").expect("marker");
        assert!(attempted_recently(&marker, SystemTime::now()));
        let later = SystemTime::now() + RETRY_AFTER + Duration::from_secs(1);
        assert!(!attempted_recently(&marker, later));
        let _ = std::fs::remove_file(marker);
    }
}
