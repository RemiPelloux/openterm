const WORKSPACE_HEADER: &str = "OpenTerm workspace context (read-only):";

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
}
