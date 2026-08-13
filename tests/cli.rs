use std::path::PathBuf;
use std::process::Command;

fn openterm_as(name: &str) -> Command {
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_openterm"));
    if name == "openterm" {
        return Command::new(binary);
    }
    let directory = binary.parent().expect("binary directory");
    let link = directory.join(name);
    create_link(&binary, &link);
    Command::new(link)
}

#[cfg(unix)]
fn create_link(binary: &std::path::Path, link: &std::path::Path) {
    use std::io::ErrorKind;
    let _ = std::fs::remove_file(link);
    match std::os::unix::fs::symlink(binary, link) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
        Err(error) => panic!("create command link: {error}"),
    }
}

#[cfg(windows)]
fn create_link(binary: &std::path::Path, link: &std::path::Path) {
    std::fs::copy(binary, link.with_extension("exe")).expect("copy command binary");
}

#[test]
fn ai_version_reports_package_version() {
    let output = openterm_as("ai-version").output().expect("run ai-version");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert_eq!(
        stdout.trim(),
        concat!("openterm ", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn ai_explain_rejects_last_and_result_together() {
    let output = openterm_as("ai-explain")
        .arg("--last")
        .arg("--result")
        .arg("res_test")
        .output()
        .expect("run ai-explain");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("not both"));
}

#[test]
fn ai_help_supports_markdown_output() {
    let output = openterm_as("ai-help")
        .arg("--markdown")
        .output()
        .expect("run ai-help --markdown");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.starts_with("# OpenTerm command reference"));
    assert!(stdout.contains("```bash"));
}

#[test]
fn ai_help_lists_commands() {
    let output = openterm_as("ai-help").output().expect("run ai-help");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("ai-help"));
    assert!(stdout.contains("ai-ask"));
    assert!(stdout.contains("ai-rg"));
    assert!(stdout.contains("Agent & guidance"));
}

#[test]
fn launcher_lists_new_rtk_commands() {
    let output = openterm_as("openterm")
        .arg("help")
        .output()
        .expect("run help");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("ai-rg"));
    assert!(stdout.contains("ai-panel"));
    assert!(stdout.contains("ai-workspace"));
    assert!(stdout.contains("ai-json"));
    assert!(stdout.contains("ai-pytest"));
    assert!(stdout.contains("ai-cargo"));
}

#[test]
fn ai_ask_rejects_missing_question() {
    let output = openterm_as("ai-ask").output().expect("run ai-ask");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("a question is required"));
}

#[test]
fn launcher_rejects_unprefixed_command() {
    let output = openterm_as("openterm")
        .arg("doctor")
        .output()
        .expect("run openterm");
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn ai_run_missing_program_returns_command_not_found() {
    let output = openterm_as("ai-run")
        .arg("openterm-program-that-does-not-exist")
        .output()
        .expect("run ai-run");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(127));
    assert!(stderr.contains("was not found"));
}

#[cfg(unix)]
#[test]
fn managed_command_stores_expandable_evidence() {
    let root = test_root();
    std::fs::create_dir_all(&root).expect("create test root");
    let fake_rtk = root.join("rtk");
    write_executable(&fake_rtk, "#!/bin/sh\nprintf 'stored evidence\\n'\n");
    let path = format!("{}:{}", root.display(), std::env::var("PATH").unwrap());

    let output = openterm_as("ai-grep")
        .arg("needle")
        .env("PATH", path)
        .env("OPENTERM_DATA_DIR", &root)
        .output()
        .expect("run managed command");
    let result_id = parse_result_id(&output.stderr);
    let forgotten = result_command("ai-forget", &result_id, &root);
    let restored = result_command("ai-restore", &result_id, &root);
    let expanded = result_command("ai-expand", &result_id, &root);

    assert_eq!(output.stdout, b"stored evidence\n");
    assert!(forgotten.status.success());
    assert!(restored.status.success());
    assert_eq!(expanded.stdout, b"stored evidence\n");
    std::fs::remove_dir_all(root).expect("remove test root");
}

fn result_command(name: &str, result_id: &str, root: &std::path::Path) -> std::process::Output {
    openterm_as(name)
        .arg(result_id)
        .env("OPENTERM_DATA_DIR", root)
        .output()
        .unwrap_or_else(|error| panic!("run {name}: {error}"))
}

#[cfg(unix)]
fn write_executable(path: &std::path::Path, contents: &str) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::write(path, contents).expect("write executable");
    let permissions = std::fs::Permissions::from_mode(0o700);
    std::fs::set_permissions(path, permissions).expect("set executable permissions");
}

fn parse_result_id(stderr: &[u8]) -> String {
    String::from_utf8_lossy(stderr)
        .lines()
        .find_map(|line| line.strip_prefix("openterm: result "))
        .and_then(|line| line.split_whitespace().next())
        .expect("result reference")
        .to_owned()
}

fn test_root() -> PathBuf {
    let mut bytes = [0_u8; 8];
    getrandom::fill(&mut bytes).expect("random test suffix");
    std::env::temp_dir().join(format!("openterm-cli-test-{}", hex::encode(bytes)))
}
