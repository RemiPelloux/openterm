use crate::markdown;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const SHELL_FILES: [&str; 4] = [".zshenv", ".zprofile", ".zshrc", ".zlogin"];

pub fn run(prompt: &str) -> Result<(), String> {
    if !std::io::stdin().is_terminal() {
        eprintln!("openterm: skipped OpenCode apply (not a TTY)");
        return Ok(());
    }
    let snapshots = snapshot_shell();
    eprintln!("openterm: applying with OpenCode…");
    let output = invoke(prompt)?;
    print_opencode_output(&output);
    print_shell_diffs(&snapshots);
    print_git_diff();
    Ok(())
}

fn invoke(prompt: &str) -> Result<std::process::Output, String> {
    let mut command = Command::new(opencode_program());
    command
        .args([
            "--dangerously-skip-permissions",
            "--print",
            "--output-format",
            "text",
            "--no-session-persistence",
        ])
        .stderr(Stdio::inherit());
    if let Some(home) = std::env::var_os("HOME") {
        command.arg("--add-dir").arg(home);
    }
    command
        .arg(prompt)
        .output()
        .map_err(|error| format!("opencode failed to start: {error}"))
}

fn print_opencode_output(output: &std::process::Output) {
    let text = String::from_utf8_lossy(&output.stdout);
    if !text.trim().is_empty() {
        print!("{}", markdown::render(text.trim_end()));
        println!();
    }
    if !output.status.success() {
        eprintln!("openterm: OpenCode exited {}", status_code(output));
    }
}

fn opencode_program() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        let candidate = PathBuf::from(home).join(".local/bin/opencode");
        if candidate.is_file() {
            return candidate;
        }
    }
    PathBuf::from("opencode")
}

fn snapshot_shell() -> Vec<(PathBuf, String)> {
    let Some(home) = std::env::var_os("HOME") else {
        return Vec::new();
    };
    let home = PathBuf::from(home);
    SHELL_FILES
        .iter()
        .filter_map(|name| read_pair(home.join(name)))
        .collect()
}

fn read_pair(path: PathBuf) -> Option<(PathBuf, String)> {
    let text = std::fs::read_to_string(&path).ok()?;
    Some((path, text))
}

fn print_shell_diffs(before: &[(PathBuf, String)]) {
    for (path, old) in before {
        let new = std::fs::read_to_string(path).unwrap_or_default();
        if let Some(diff) = file_diff(path, old, &new) {
            print!("{}", markdown::render(&format!("```diff\n{diff}\n```\n")));
        }
    }
}

fn file_diff(path: &Path, before: &str, after: &str) -> Option<String> {
    if before == after {
        return None;
    }
    let old = temp_name("old");
    let new = temp_name("new");
    std::fs::write(&old, before).ok()?;
    std::fs::write(&new, after).ok()?;
    let output = Command::new("diff")
        .args([
            "-u",
            "-L",
            &format!("a/{}", path.display()),
            "-L",
            &format!("b/{}", path.display()),
            old.to_str()?,
            new.to_str()?,
        ])
        .output()
        .ok();
    let _ = std::fs::remove_file(&old);
    let _ = std::fs::remove_file(&new);
    let stdout = String::from_utf8_lossy(&output?.stdout).trim().to_owned();
    (!stdout.is_empty()).then_some(stdout)
}

fn print_git_diff() {
    let output = Command::new("git").args(["diff", "--no-ext-diff"]).output();
    let Ok(output) = output else {
        return;
    };
    let text = String::from_utf8_lossy(&output.stdout);
    if text.trim().is_empty() {
        return;
    }
    println!(
        "{}",
        markdown::render(&format!(
            "## git diff\n\n```diff\n{}\n```\n",
            text.trim_end()
        ))
    );
}

fn temp_name(side: &str) -> PathBuf {
    std::env::temp_dir().join(format!("openterm-ask-{side}-{}", std::process::id()))
}

fn status_code(output: &std::process::Output) -> i32 {
    output.status.code().unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::file_diff;
    use std::path::Path;

    #[test]
    fn file_diff_skips_unchanged() {
        assert_eq!(file_diff(Path::new("/tmp/x"), "same\n", "same\n"), None);
    }

    #[test]
    fn file_diff_emits_unified() {
        let diff = file_diff(Path::new("/tmp/zshrc"), "a\n", "b\n").expect("diff");
        assert!(diff.contains("-a"));
        assert!(diff.contains("+b"));
    }
}
