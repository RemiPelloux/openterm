use serde::Serialize;
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

#[derive(Default, Serialize)]
struct GitContext {
    repo: String,
    root: String,
    branch: String,
    ahead: u32,
    behind: u32,
    staged: u32,
    modified: u32,
    renamed: u32,
    untracked: u32,
    conflicts: u32,
    stashes: u32,
}

pub fn prompt_line() -> Option<String> {
    load_status().map(|context| prompt(&context))
}

pub fn run(args: impl IntoIterator<Item = OsString>) -> i32 {
    let args = args.into_iter().collect::<Vec<_>>();
    let mode = match parse_mode(&args) {
        Ok(mode) => mode,
        Err(error) => {
            eprintln!("error: {error}");
            return 2;
        }
    };
    let loaded = match mode {
        Mode::Prompt | Mode::Ansi => load_status(),
        Mode::Human | Mode::Json => load(),
    };
    let Some(context) = loaded else {
        if !matches!(mode, Mode::Prompt | Mode::Ansi) {
            println!("Not inside a Git worktree.");
        }
        return 0;
    };
    print_context(&context, mode);
    0
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Human,
    Prompt,
    Ansi,
    Json,
}

fn parse_mode(args: &[OsString]) -> Result<Mode, &'static str> {
    match args {
        [] => Ok(Mode::Human),
        [value] if value == "--prompt" => Ok(Mode::Prompt),
        [value] if value == "--ansi" => Ok(Mode::Ansi),
        [value] if value == "--json" => Ok(Mode::Json),
        _ => Err("usage: ai-context [--prompt|--ansi|--json]"),
    }
}

/// Everything the prompt needs, in one `git` process. This runs on every prompt
/// draw, so it must not grow a second call. `--no-optional-locks` keeps a prompt
/// redraw from contending with a concurrent git command for the index lock.
fn load_status() -> Option<GitContext> {
    let status = git_output(&[
        "--no-optional-locks",
        "status",
        "--porcelain=v2",
        "--branch",
        "--show-stash",
    ])?;
    Some(parse_status(&status))
}

fn load() -> Option<GitContext> {
    let mut context = load_status()?;
    let root = git_output(&["rev-parse", "--show-toplevel"])?;
    context.repo = Path::new(&root)
        .file_name()
        .map_or_else(|| root.clone(), |name| name.to_string_lossy().into_owned());
    context.root = root;
    Some(context)
}

fn git_output(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn parse_status(status: &str) -> GitContext {
    let mut context = GitContext::default();
    for line in status.lines() {
        if line.starts_with("# ") {
            parse_header(&mut context, line);
        } else {
            parse_change(&mut context, line);
        }
    }
    context
}

fn parse_header(context: &mut GitContext, line: &str) {
    if let Some(branch) = line.strip_prefix("# branch.head ") {
        branch.clone_into(&mut context.branch);
    } else if let Some(value) = line.strip_prefix("# branch.ab ") {
        parse_divergence(context, value);
    } else if let Some(value) = line.strip_prefix("# stash ") {
        context.stashes = value.parse().unwrap_or_default();
    }
}

fn parse_divergence(context: &mut GitContext, value: &str) {
    for part in value.split_whitespace() {
        if let Some(ahead) = part.strip_prefix('+') {
            context.ahead = ahead.parse().unwrap_or_default();
        } else if let Some(behind) = part.strip_prefix('-') {
            context.behind = behind.parse().unwrap_or_default();
        }
    }
}

fn parse_change(context: &mut GitContext, line: &str) {
    if line.starts_with("? ") {
        context.untracked += 1;
        return;
    }
    if line.starts_with("u ") {
        context.conflicts += 1;
        return;
    }
    let Some(state) = line.split_whitespace().nth(1) else {
        return;
    };
    let mut chars = state.chars();
    context.staged += u32::from(chars.next().is_some_and(|value| value != '.'));
    context.modified += u32::from(chars.next().is_some_and(|value| value != '.'));
    context.renamed += u32::from(line.starts_with("2 "));
}

fn print_context(context: &GitContext, mode: Mode) {
    match mode {
        Mode::Prompt => println!("{}", prompt(context)),
        Mode::Ansi => print!("{}", prompt_ansi(context)),
        Mode::Json => println!(
            "{}",
            serde_json::to_string_pretty(context).unwrap_or_default()
        ),
        Mode::Human => print_human(context),
    }
}

fn print_human(context: &GitContext) {
    println!("{} · {}", context.repo, context.branch);
    println!("  root       {}", context.root);
    println!("  staged     {}", context.staged);
    println!("  modified   {}", context.modified);
    println!("  renamed    {}", context.renamed);
    println!("  untracked  {}", context.untracked);
    println!("  conflicts  {}", context.conflicts);
    println!("  ahead      {}", context.ahead);
    println!("  behind     {}", context.behind);
    println!("  stashes    {}", context.stashes);
}

fn prompt(context: &GitContext) -> String {
    let mut parts = vec![format!("git:{}", context.branch)];
    push_count(&mut parts, "+", context.staged);
    push_count(&mut parts, "~", context.modified);
    push_count(&mut parts, "?", context.untracked);
    push_count(&mut parts, "!", context.conflicts);
    push_count(&mut parts, "↑", context.ahead);
    push_count(&mut parts, "↓", context.behind);
    push_count(&mut parts, "≡", context.stashes);
    parts.join(" ")
}

fn push_count(parts: &mut Vec<String>, symbol: &str, count: u32) {
    if count > 0 {
        parts.push(format!("{symbol}{count}"));
    }
}

fn prompt_ansi(context: &GitContext) -> String {
    let mut out = String::from("\x1b[48;5;236m \x1b[1;38;5;255m⎇ \x1b[38;5;81m");
    out.push_str(&context.branch);
    out.push_str("\x1b[0m");
    push_ansi_count(&mut out, "+", context.staged, "82");
    push_ansi_count(&mut out, "~", context.modified, "214");
    push_ansi_count(&mut out, "?", context.untracked, "228");
    push_ansi_count(&mut out, "!", context.conflicts, "196");
    push_ansi_count(&mut out, "↑", context.ahead, "45");
    push_ansi_count(&mut out, "↓", context.behind, "208");
    push_ansi_count(&mut out, "≡", context.stashes, "141");
    out.push_str(" \x1b[0m");
    out
}

fn push_ansi_count(out: &mut String, symbol: &str, count: u32, color: &str) {
    if count > 0 {
        out.push_str("\x1b[38;5;236m \x1b[1;38;5;");
        out.push_str(color);
        out.push('m');
        out.push_str(symbol);
        out.push_str(&count.to_string());
        out.push_str("\x1b[0m");
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_status, prompt, prompt_ansi};

    #[test]
    fn ansi_prompt_highlights_branch() {
        let status = "# branch.head main\n1 M. x";
        let context = parse_status(status);
        let rendered = prompt_ansi(&context);
        assert!(rendered.contains("main"));
        assert!(rendered.contains("\x1b["));
    }

    #[test]
    fn parses_porcelain_v2_state() {
        let status = "# branch.head feature/ui\n# branch.ab +2 -1\n# stash 3\n1 M. x\n1 .M y\n2 R. z\n? new\nu UU conflict";
        let context = parse_status(status);
        assert_eq!(context.branch, "feature/ui");
        assert_eq!((context.ahead, context.behind, context.stashes), (2, 1, 3));
        assert_eq!((context.staged, context.modified), (2, 1));
        assert_eq!(
            (context.renamed, context.untracked, context.conflicts),
            (1, 1, 1)
        );
        assert!(prompt(&context).contains("git:feature/ui"));
    }
}
