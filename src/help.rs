use crate::help_catalog::SECTIONS;
use crate::help_render::{
    EntryView, Output, default_output, render_entry, render_footer, render_header, render_one_rtk,
    render_rtk, render_rtk_header, render_section, render_summary_footer,
};
use std::ffi::OsString;

struct Options {
    output: Output,
    topic: Option<String>,
}

pub fn run(args: impl IntoIterator<Item = OsString>) -> i32 {
    let options = match parse_options(args.into_iter().collect()) {
        Ok(options) => options,
        Err(code) => return code,
    };
    if let Some(topic) = options.topic {
        if print_one(&topic, options.output) {
            return 0;
        }
        eprintln!("error: unknown topic '{topic}'\n\nRun ai-help for the full catalog.");
        return 2;
    }
    print_full(options.output);
    0
}

pub fn print_summary() {
    print!("{}", build_summary(default_output()));
}

fn parse_options(args: Vec<OsString>) -> Result<Options, i32> {
    let mut output = default_output();
    let mut topic = None;
    for arg in args {
        let value = arg.to_string_lossy();
        match value.as_ref() {
            "--help" | "-h" => {}
            "--markdown" | "-m" => output = Output::Markdown,
            "--plain" => output = Output::Plain,
            flag if flag.starts_with('-') => {
                eprintln!(
                    "error: unknown option '{flag}'\n\nUsage: ai-help [--markdown|--plain] [COMMAND]"
                );
                return Err(2);
            }
            name => topic = Some(name.to_owned()),
        }
    }
    Ok(Options { output, topic })
}

fn print_full(output: Output) {
    print!("{}", build_full(output));
}

fn build_full(output: Output) -> String {
    let mut text = render_header(true, output);
    for section in SECTIONS {
        text.push_str(&render_section(section.title, output));
        for entry in section.entries {
            text.push_str(&render_entry(&entry_view(entry, true), output));
        }
    }
    text.push_str(&render_rtk_header(output));
    for command in crate::rtk_commands::all() {
        text.push_str(&render_rtk(command.name, command.description, output));
    }
    text.push_str(&render_footer(output));
    text
}

fn build_summary(output: Output) -> String {
    let mut text = render_header(false, output);
    append_summary_entries(&mut text, output);
    text.push_str(&render_summary_footer(output));
    text
}

fn append_summary_entries(text: &mut String, output: Output) {
    for section in SECTIONS {
        for entry in section.entries {
            text.push_str(&render_entry(&entry_view(entry, false), output));
        }
    }
    for command in crate::rtk_commands::all() {
        let usage = format!("{} [ARGS...]", command.name);
        text.push_str(&render_entry(
            &EntryView {
                name: command.name,
                summary: command.description,
                usage: &usage,
                detailed: false,
            },
            output,
        ));
    }
}

fn entry_view(entry: &crate::help_catalog::Entry, detailed: bool) -> EntryView<'_> {
    EntryView {
        name: entry.name,
        summary: entry.summary,
        usage: entry.usage,
        detailed,
    }
}

fn print_one(name: &str, output: Output) -> bool {
    for section in SECTIONS {
        if let Some(entry) = section.entries.iter().find(|entry| entry.name == name) {
            print!("{}", render_entry(&entry_view(entry, true), output));
            return true;
        }
    }
    if let Some(command) = crate::rtk_commands::find(name) {
        print!(
            "{}",
            render_one_rtk(command.name, command.description, command.prefix, output)
        );
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{build_full, parse_options, run};
    use crate::help_render::Output;
    use std::ffi::OsString;

    #[test]
    fn full_help_exits_zero() {
        assert_eq!(run([]), 0);
    }

    #[test]
    fn markdown_flag_sets_output() {
        let options = parse_options(vec![OsString::from("--markdown")]).expect("options");
        assert_eq!(options.output, Output::Markdown);
    }

    #[test]
    fn markdown_output_uses_headings() {
        let text = build_full(Output::Markdown);
        assert!(text.contains("## Agent & guidance"));
        assert!(text.contains("```bash"));
    }

    #[test]
    fn single_command_help_exits_zero() {
        assert_eq!(run([OsString::from("ai-ask")]), 0);
    }

    #[test]
    fn unknown_topic_returns_error() {
        assert_eq!(run([OsString::from("missing")]), 2);
    }

    #[test]
    fn sections_cover_core_commands() {
        for name in ["ai-help", "ai-ask", "ai-run"] {
            assert!(
                crate::help_catalog::SECTIONS
                    .iter()
                    .any(|section| { section.entries.iter().any(|entry| entry.name == name) })
            );
        }
    }
}
