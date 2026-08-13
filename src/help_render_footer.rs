use crate::help_render::{Output, Style};

pub fn render_summary_footer(output: Output) -> String {
    match output {
        Output::Markdown => {
            "\nEvery OpenTerm tool command starts with `ai-`.\nRun `ai-help` for full usage notes.\n"
                .to_owned()
        }
        Output::Ansi => {
            let style = Style::new(output);
            format!(
                "\nEvery OpenTerm tool command starts with ai-.\nRun {} for full usage notes.\n",
                style.code("ai-help")
            )
        }
        Output::Plain => {
            "\nEvery OpenTerm tool command starts with ai-.\nRun ai-help for full usage notes.\n"
                .to_owned()
        }
    }
}

pub fn render_footer(output: Output) -> String {
    match output {
        Output::Markdown => markdown_footer(),
        Output::Ansi => ansi_footer(),
        Output::Plain => plain_footer(),
    }
}

fn markdown_footer() -> String {
    "\n## Notes\n\n\
        - Every user-facing tool starts with `ai-`.\n\
        - `ai-ask` inspects local files and may apply edits through OpenCode.\n\
        - `ai-chat` and `ai-explain` are read-only — they never modify files.\n\
        - Managed commands store evidence in `~/Library/Application Support/OpenTerm/data`.\n\
        - Run `ai-help --markdown | glow` for a rendered page if `glow` is installed.\n\
        - Run `ai-help COMMAND` for one entry.\n"
        .to_owned()
}

fn ansi_footer() -> String {
    let style = Style::new(Output::Ansi);
    format!(
        "{}\n  • Every user-facing tool starts with ai-.\n\
           • ai-ask inspects local files and may apply edits through OpenCode.\n\
           • ai-chat and ai-explain are read-only — they never modify files.\n\
           • Managed commands store evidence locally under Application Support/OpenTerm/data.\n\
           • Try {} for raw Markdown, or pipe it to glow.\n\
           • Use ai-help COMMAND for a single entry.\n",
        style.heading("Notes"),
        style.code("ai-help --markdown")
    )
}

fn plain_footer() -> String {
    "\nNotes:\n  • Every user-facing tool starts with ai-.\n\
        • ai-ask inspects local files and may apply edits through OpenCode.\n\
        • ai-chat and ai-explain are read-only — they never modify files.\n\
        • Managed commands store evidence under Application Support/OpenTerm/data.\n\
        • ai-help --markdown emits Markdown; pipe to glow to render it.\n\
        • Use ai-help COMMAND for a single entry.\n"
        .to_owned()
}

pub fn render_one_rtk(name: &str, summary: &str, prefix: &[&str], output: Output) -> String {
    match output {
        Output::Markdown => format!(
            "### `{name}`\n\n{summary}\n\nWraps: `rtk {}`\n\nStores compressed evidence when run inside OpenTerm.\n",
            prefix.join(" ")
        ),
        Output::Ansi => {
            let style = Style::new(output);
            format!(
                "{} — {}\n  wraps: {}\n  stores compressed evidence when run inside OpenTerm\n",
                style.command(name),
                summary,
                style.code(&format!("rtk {}", prefix.join(" ")))
            )
        }
        Output::Plain => format!(
            "{name} — {summary}\n  wraps: rtk {}\n  stores compressed evidence when run inside OpenTerm\n",
            prefix.join(" ")
        ),
    }
}
