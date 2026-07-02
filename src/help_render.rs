use std::io::IsTerminal;

pub use crate::help_render_footer::{render_footer, render_one_rtk, render_summary_footer};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Output {
    Ansi,
    Markdown,
    Plain,
}

pub struct EntryView<'a> {
    pub name: &'a str,
    pub summary: &'a str,
    pub usage: &'a str,
    pub detailed: bool,
}

pub struct Style {
    enabled: bool,
}

pub fn default_output() -> Output {
    if stdout_color() {
        Output::Ansi
    } else {
        Output::Plain
    }
}

pub fn stdout_color() -> bool {
    std::io::stdout().is_terminal()
        && std::env::var_os("NO_COLOR").is_none()
        && std::env::var("TERM").is_ok_and(|term| term != "dumb")
}

impl Style {
    pub fn new(output: Output) -> Self {
        Self {
            enabled: output == Output::Ansi,
        }
    }

    fn wrap(&self, code: &str, text: &str) -> String {
        if self.enabled {
            format!("{code}{text}\x1b[0m")
        } else {
            text.to_owned()
        }
    }

    pub fn title(&self, text: &str) -> String {
        self.wrap("\x1b[1m\x1b[38;5;81m", text)
    }

    pub fn heading(&self, text: &str) -> String {
        self.wrap("\x1b[1m", text)
    }

    pub fn command(&self, text: &str) -> String {
        self.wrap("\x1b[1m\x1b[38;5;45m", text)
    }

    pub fn muted(&self, text: &str) -> String {
        self.wrap("\x1b[38;5;238m", text)
    }

    pub fn code(&self, text: &str) -> String {
        if self.enabled {
            self.wrap("\x1b[38;5;252m", text)
        } else {
            text.to_owned()
        }
    }
}

pub fn render_header(detailed: bool, output: Output) -> String {
    match output {
        Output::Markdown => render_markdown_header(detailed),
        Output::Plain => render_plain_header(detailed),
        Output::Ansi => render_ansi_header(detailed),
    }
}

fn render_markdown_header(detailed: bool) -> String {
    if detailed {
        "# OpenTerm command reference\n\n\
         OpenTerm wraps everyday terminal work with **ai-\\*** tools.\n\
         RTK-backed commands compress output and store encrypted evidence.\n"
            .to_owned()
    } else {
        "# OpenTerm\n\nShort command list. Run `ai-help` for full usage.\n\n".to_owned()
    }
}

fn render_plain_header(detailed: bool) -> String {
    if detailed {
        "OpenTerm command reference\n\n\
         OpenTerm wraps everyday terminal work with ai-* tools.\n\
         RTK-backed commands compress output and store encrypted evidence.\n\n"
            .to_owned()
    } else {
        "OpenTerm — AI-native terminal foundation\n\nCommands:\n".to_owned()
    }
}

fn render_ansi_header(detailed: bool) -> String {
    let style = Style::new(Output::Ansi);
    if detailed {
        format!(
            "{}\n\nOpenTerm wraps everyday terminal work with {} tools.\n\
             RTK-backed commands compress output and store encrypted evidence.\n\n",
            style.title("OpenTerm command reference"),
            style.command("ai-*")
        )
    } else {
        format!(
            "{} — AI-native terminal foundation\n\nCommands:\n",
            style.title("OpenTerm")
        )
    }
}

pub fn render_section(title: &str, output: Output) -> String {
    match output {
        Output::Markdown => format!("\n## {title}\n\n"),
        Output::Ansi => format!("\n{}\n", Style::new(output).heading(title)),
        Output::Plain => format!("\n{title}\n"),
    }
}

pub fn render_entry(entry: &EntryView<'_>, output: Output) -> String {
    match output {
        Output::Markdown => render_markdown_entry(entry),
        Output::Ansi => render_ansi_entry(entry),
        Output::Plain => render_plain_entry(entry),
    }
}

fn render_markdown_entry(entry: &EntryView<'_>) -> String {
    if entry.detailed {
        format!(
            "### `{name}`\n\n{summary}\n\n```bash\n{usage}\n```\n\n",
            name = entry.name,
            summary = entry.summary,
            usage = entry.usage
        )
    } else {
        format!("- **`{}`** — {}\n", entry.name, entry.summary)
    }
}

fn render_ansi_entry(entry: &EntryView<'_>) -> String {
    let style = Style::new(Output::Ansi);
    if entry.detailed {
        format!(
            "  {:<18} {}\n    {}\n",
            style.command(entry.name),
            entry.summary,
            style.muted(entry.usage)
        )
    } else {
        format!("  {:<18} {}\n", style.command(entry.name), entry.summary)
    }
}

fn render_plain_entry(entry: &EntryView<'_>) -> String {
    if entry.detailed {
        format!(
            "  {name:<18} {summary}\n    {usage}\n",
            name = entry.name,
            summary = entry.summary,
            usage = entry.usage
        )
    } else {
        format!(
            "  {name:<18} {summary}\n",
            name = entry.name,
            summary = entry.summary
        )
    }
}

pub fn render_rtk_header(output: Output) -> String {
    match output {
        Output::Markdown => {
            "\n## RTK output tools\n\nPrefer **ai-rg** over ai-grep. Each command preserves exit codes.\n\n"
                .to_owned()
        }
        Output::Ansi => {
            let style = Style::new(output);
            format!(
                "\n{}\nPrefer {} over ai-grep. Each command preserves exit codes.\n\n",
                style.heading("RTK output tools (token-efficient wrappers)"),
                style.command("ai-rg")
            )
        }
        Output::Plain => {
            "\nRTK output tools (token-efficient wrappers)\n\
             Prefer ai-rg over ai-grep. Each command preserves exit codes.\n\n"
                .to_owned()
        }
    }
}

pub fn render_rtk(name: &str, summary: &str, output: Output) -> String {
    let usage = format!("{name} [ARGS...]");
    render_entry(
        &EntryView {
            name,
            summary,
            usage: &usage,
            detailed: true,
        },
        output,
    )
}

#[cfg(test)]
mod tests {
    use super::{EntryView, Output, render_entry, render_header};

    #[test]
    fn markdown_header_uses_hash_syntax() {
        let text = render_header(true, Output::Markdown);
        assert!(text.starts_with("# OpenTerm command reference"));
    }

    #[test]
    fn ansi_entry_includes_escape_codes() {
        let entry = EntryView {
            name: "ai-ask",
            summary: "Ask the model",
            usage: "ai-ask QUESTION",
            detailed: true,
        };
        let text = render_entry(&entry, Output::Ansi);
        assert!(text.contains("\x1b["));
        assert!(text.contains("ai-ask"));
    }

    #[test]
    fn plain_entry_has_no_escape_codes() {
        let entry = EntryView {
            name: "ai-ask",
            summary: "Ask the model",
            usage: "ai-ask QUESTION",
            detailed: true,
        };
        let text = render_entry(&entry, Output::Plain);
        assert!(!text.contains("\x1b["));
    }
}
