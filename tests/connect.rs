//! `ai-connect` non-interactive contract, run against a throwaway config dir
//! with the keychain switched off so tests never touch the login keychain.
#![cfg(unix)]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn root() -> PathBuf {
    let mut bytes = [0_u8; 8];
    getrandom::fill(&mut bytes).expect("random suffix");
    let root = std::env::temp_dir().join(format!("openterm-connect-test-{}", hex::encode(bytes)));
    std::fs::create_dir_all(root.join("bin")).expect("create root");
    root
}

fn ai_connect(root: &Path, args: &[&str], stdin: &str) -> Output {
    let link = root.join("ai-connect");
    if !link.exists() {
        std::os::unix::fs::symlink(env!("CARGO_BIN_EXE_openterm"), &link).expect("link");
    }
    let mut child = Command::new(link)
        .args(args)
        .env("OPENTERM_CONFIG_DIR", root.join("config"))
        .env("OPENTERM_DATA_DIR", root.join("data"))
        .env("OPENTERM_KEYCHAIN", "off")
        .env("PATH", root.join("bin"))
        .env_remove("GROQ_API_KEY")
        .env_remove("OPENTERM_CUSTOM_BASE_URL")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn ai-connect");
    let mut input = child.stdin.take().expect("stdin");
    input.write_all(stdin.as_bytes()).expect("write stdin");
    drop(input);
    child.wait_with_output().expect("wait")
}

fn provider_row<'a>(status: &'a serde_json::Value, id: &str) -> &'a serde_json::Value {
    status["providers"]
        .as_array()
        .expect("providers")
        .iter()
        .find(|entry| entry["id"] == id)
        .expect("provider row")
}

#[test]
fn key_stdin_connects_and_status_reports_the_source() {
    let root = root();
    let output = ai_connect(&root, &["groq", "--model", "x", "--key-stdin"], "sk-test\n");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "Connected to groq · x\n"
    );
    let stored = std::fs::read_to_string(root.join("config/providers.json")).expect("config");
    assert!(
        stored.contains("\"api_key\": \"sk-test\""),
        "keychain off falls back to file"
    );

    let status = ai_connect(&root, &["--status", "--json"], "");
    let value: serde_json::Value = serde_json::from_slice(&status.stdout).expect("json");
    assert_eq!(value["default_provider"], "groq");
    let groq = provider_row(&value, "groq");
    assert_eq!(groq["connected"], true);
    assert_eq!(groq["source"], "file");
    assert_eq!(groq["model"], "x");
    assert!(groq["label"].is_string());
    assert!(!String::from_utf8_lossy(&status.stdout).contains("sk-test"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn key_on_argv_is_a_usage_error() {
    let root = root();
    let output = ai_connect(&root, &["groq", "--key", "sk-test"], "");
    assert_eq!(output.status.code(), Some(2));
    assert!(!root.join("config/providers.json").exists());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn custom_without_base_url_fails() {
    let root = root();
    let output = ai_connect(&root, &["custom", "--model", "m", "--key-stdin"], "k");
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("--base-url"));
    let _ = std::fs::remove_dir_all(root);
}
