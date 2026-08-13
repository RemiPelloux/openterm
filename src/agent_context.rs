use std::path::Path;

const WORKSPACE_HEADER: &str = "OpenTerm workspace context (read-only):";
const SHELL_HEADER: &str = "OpenTerm shell files (local, redacted):";
const MAX_FILE_CHARS: usize = 8_192;
const SHELL_NAMES: [&str; 5] = [
    ".zshenv",
    ".zprofile",
    ".zshrc",
    ".zlogin",
    ".config/zsh/.zshrc",
];

pub fn workspace_block() -> String {
    let mut parts = vec![WORKSPACE_HEADER.to_owned()];
    parts.push(format!("cwd: {}", current_directory()));
    if let Some(context) = crate::git_context::prompt_line() {
        parts.push(format!("git: {context}"));
    }
    parts.join("\n")
}

pub fn result_block(id: &str) -> Result<String, String> {
    crate::results::preview_result(id).map_err(|error| error.to_string())
}

pub fn latest_result_block() -> Result<String, String> {
    crate::results::preview_latest_result().map_err(|error| error.to_string())
}

pub fn shell_block() -> Option<String> {
    let home = std::env::var_os("HOME")?;
    shell_block_from(Path::new(&home))
}

pub(crate) fn shell_block_from(home: &Path) -> Option<String> {
    let mut files = Vec::new();
    for name in SHELL_NAMES {
        let path = home.join(name);
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        files.push(file_section(&path, &text));
    }
    if files.is_empty() {
        return None;
    }
    let mut block = vec![SHELL_HEADER.to_owned()];
    block.extend(files);
    Some(block.join("\n\n"))
}

fn file_section(path: &Path, text: &str) -> String {
    let clipped = clip(text);
    let body = clipped
        .lines()
        .map(redact_line)
        .collect::<Vec<_>>()
        .join("\n");
    format!("### {}\n```zsh\n{body}\n```", path.display())
}

fn clip(text: &str) -> String {
    if text.len() <= MAX_FILE_CHARS {
        return text.to_owned();
    }
    let mut end = MAX_FILE_CHARS.min(text.len());
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}\n… [truncated]", &text[..end])
}

fn redact_line(line: &str) -> String {
    if !is_secret(line) {
        return line.to_owned();
    }
    line.split_once('=').map_or_else(
        || "[redacted]".to_owned(),
        |(name, _)| format!("{name}=[redacted]"),
    )
}

fn is_secret(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("secret")
        || lower.contains("password")
        || line.contains("sk-")
}

fn current_directory() -> String {
    std::env::current_dir().map_or_else(
        |_| ".".to_owned(),
        |path| path.to_string_lossy().into_owned(),
    )
}

#[cfg(test)]
mod tests {
    use super::workspace_block;

    #[test]
    fn workspace_block_includes_cwd() {
        let block = workspace_block();
        assert!(block.contains("cwd:"));
        assert!(block.contains("OpenTerm workspace context"));
    }

    #[test]
    fn shell_block_redacts_and_skips_missing() {
        let root = std::env::temp_dir().join(format!("openterm-shell-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&root);
        std::fs::write(
            root.join(".zshrc"),
            "export PATH=/usr/bin\nexport OPENAI_API_KEY=sk-secret\n",
        )
        .unwrap();
        let block = super::shell_block_from(&root).expect("zshrc");
        assert!(block.contains(".zshrc"));
        assert!(block.contains("PATH=/usr/bin"));
        assert!(block.contains("[redacted]"));
        assert!(!block.contains("sk-secret"));
        let _ = std::fs::remove_dir_all(root);
    }
}
