use std::io::IsTerminal;

const CYAN: &str = "\x1b[38;5;81m";
const TEAL: &str = "\x1b[38;5;45m";
const MUTED: &str = "\x1b[38;5;238m";
const TEXT: &str = "\x1b[38;5;252m";
const GREEN: &str = "\x1b[38;5;114m";
const RED: &str = "\x1b[38;5;203m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";
const FENCE: &str = "```";
const OPENCODE: &str = "```opencode";

pub fn render(text: &str) -> String {
    let color = color_enabled();
    let mut out = String::new();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if let Some(lang) = fence_lang(line) {
            let body = take_fence(&mut lines);
            out.push_str(&render_fence(lang, &body, color));
            continue;
        }
        out.push_str(&render_line(line, color));
        out.push('\n');
    }
    out
}

pub fn split_opencode(text: &str) -> (String, Option<String>) {
    let Some(start) = text.rfind(OPENCODE) else {
        return (text.to_owned(), None);
    };
    let after = &text[start + OPENCODE.len()..];
    let rest = after.strip_prefix('\n').unwrap_or(after);
    let Some(end) = rest.find(FENCE) else {
        return (text.to_owned(), None);
    };
    let prompt = rest[..end].trim().to_owned();
    if prompt.is_empty() {
        return (text.to_owned(), None);
    }
    let mut visible = text[..start].trim_end().to_owned();
    let tail = rest[end + FENCE.len()..].trim();
    if !tail.is_empty() {
        visible.push('\n');
        visible.push_str(tail);
    }
    (visible, Some(prompt))
}

fn take_fence(lines: &mut std::str::Lines<'_>) -> String {
    let mut body = String::new();
    for line in lines.by_ref() {
        if line.starts_with(FENCE) {
            break;
        }
        if !body.is_empty() {
            body.push('\n');
        }
        body.push_str(line);
    }
    body
}

pub(crate) fn fence_lang(line: &str) -> Option<&str> {
    let rest = line.strip_prefix(FENCE)?;
    if rest.contains(FENCE) {
        return None;
    }
    Some(rest.trim())
}

fn render_fence(lang: &str, body: &str, color: bool) -> String {
    let mut out = open_fence(lang, color);
    for line in body.lines() {
        out.push_str(&fence_body_line(lang, line, color));
        out.push('\n');
    }
    out.push_str(&close_fence(color));
    out
}

pub(crate) fn open_fence(lang: &str, color: bool) -> String {
    format!(
        "{}╭─ {}{}{}\n",
        paint(color, MUTED),
        paint(color, CYAN),
        if lang.is_empty() { "code" } else { lang },
        paint(color, RESET)
    )
}

pub(crate) fn close_fence(color: bool) -> String {
    format!("{}╰─{}\n", paint(color, MUTED), paint(color, RESET))
}

pub(crate) fn fence_body_line(lang: &str, line: &str, color: bool) -> String {
    let tint = if lang == "diff" || looks_diff(line) {
        match line.chars().next() {
            Some('+') => GREEN,
            Some('-') => RED,
            Some('@') => CYAN,
            _ => TEXT,
        }
    } else {
        TEXT
    };
    format!(
        "{}│{} {}{line}{}",
        paint(color, MUTED),
        paint(color, RESET),
        paint(color, tint),
        paint(color, RESET)
    )
}

fn looks_diff(line: &str) -> bool {
    matches!(line.chars().next(), Some('+' | '-' | '@')) && line.len() > 1
}

pub(crate) fn render_line(line: &str, color: bool) -> String {
    if let Some(title) = line.strip_prefix("### ") {
        return heading(title, CYAN, color);
    }
    if let Some(title) = line.strip_prefix("## ") {
        return heading(title, TEAL, color);
    }
    if let Some(title) = line.strip_prefix("# ") {
        return heading(title, TEAL, color);
    }
    if matches!(line, "Result" | "Plan" | "Notes") {
        return heading(line, TEAL, color);
    }
    if line == "---" || line == "***" {
        return format!("{}────────{}", paint(color, MUTED), paint(color, RESET));
    }
    if let Some(item) = list_item(line) {
        return format!(
            "  {}•{} {}",
            paint(color, TEAL),
            paint(color, RESET),
            inlines(item, color)
        );
    }
    inlines(line, color)
}

fn heading(title: &str, tint: &'static str, color: bool) -> String {
    format!(
        "{}{}{}{}",
        paint(color, BOLD),
        paint(color, tint),
        inlines(title, color),
        paint(color, RESET)
    )
}

fn list_item(line: &str) -> Option<&str> {
    line.strip_prefix("- ")
        .or_else(|| line.strip_prefix("* "))
        .or_else(|| line.strip_prefix("+ "))
        .or_else(|| numbered_item(line))
}

fn numbered_item(line: &str) -> Option<&str> {
    let digits = line.bytes().take_while(u8::is_ascii_digit).count();
    if digits == 0 {
        return None;
    }
    line[digits..].strip_prefix(". ")
}

fn inlines(line: &str, color: bool) -> String {
    let mut out = String::new();
    let mut rest = line;
    while !rest.is_empty() {
        let tick = rest.find('`');
        let bold = rest.find("**");
        let next = match (tick, bold) {
            (None, None) => {
                out.push_str(rest);
                break;
            }
            (Some(tick), Some(bold)) if tick <= bold => tick,
            (Some(tick), None) => tick,
            (_, Some(bold)) => bold,
        };
        out.push_str(&rest[..next]);
        rest = &rest[next..];
        rest = if rest.starts_with('`') {
            take_code(&mut out, rest, color)
        } else {
            take_bold(&mut out, rest, color)
        };
    }
    out
}

fn take_code<'a>(out: &mut String, rest: &'a str, color: bool) -> &'a str {
    let inner = &rest[1..];
    let Some(end) = inner.find('`') else {
        out.push('`');
        return inner;
    };
    out.push_str(paint(color, CYAN));
    out.push_str(&inner[..end]);
    out.push_str(paint(color, RESET));
    &inner[end + 1..]
}

fn take_bold<'a>(out: &mut String, rest: &'a str, color: bool) -> &'a str {
    let inner = &rest[2..];
    let Some(end) = inner.find("**") else {
        out.push_str("**");
        return inner;
    };
    out.push_str(paint(color, BOLD));
    out.push_str(&inner[..end]);
    out.push_str(paint(color, RESET));
    &inner[end + 2..]
}

pub(crate) fn color_enabled() -> bool {
    std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none()
}

fn paint(enabled: bool, value: &'static str) -> &'static str {
    if enabled { value } else { "" }
}

#[cfg(test)]
mod tests {
    use super::{render, split_opencode};

    #[test]
    fn split_extracts_trailing_opencode_prompt() {
        let (visible, prompt) = split_opencode(
            "## Plan\nDo the work.\n\n```opencode\nEdit /tmp/zshrc: drop nvm from startup.\n```\n",
        );
        assert!(visible.contains("## Plan"));
        assert!(!visible.contains("```opencode"));
        assert_eq!(
            prompt.as_deref(),
            Some("Edit /tmp/zshrc: drop nvm from startup.")
        );
    }

    #[test]
    fn render_plain_headings_and_code() {
        let text = render("# Title\nUse `df -h`.\n\n```diff\n-slow\n+fast\n```\n");
        assert!(text.contains("Title"));
        assert!(text.contains("df -h"));
        assert!(text.contains("-slow"));
        assert!(text.contains("+fast"));
        assert!(!text.contains("\x1b["));
    }

    #[test]
    fn render_strips_bold_and_numbered_lists() {
        let text = render("1. **Keep .zshenv minimal** — it runs for every zsh.\n");
        assert!(text.contains("Keep .zshenv minimal"));
        assert!(!text.contains("**"));
        assert!(text.contains("•"));
    }
}
