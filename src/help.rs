use std::ffi::OsString;

struct Entry {
    name: &'static str,
    summary: &'static str,
    usage: &'static str,
}

struct Section {
    title: &'static str,
    entries: &'static [Entry],
}

const AGENT: &[Entry] = &[
    Entry {
        name: "ai-help",
        summary: "List every OpenTerm command with usage notes",
        usage: "ai-help",
    },
    Entry {
        name: "ai-ask",
        summary: "Ask an LLM for read-only terminal guidance",
        usage: "ai-ask [--provider codex|openai|claude|mistral] [--model MODEL] [--workspace] [--result REF] QUESTION",
    },
    Entry {
        name: "ai-explain",
        summary: "Explain stored command output with your model",
        usage: "ai-explain [--last|--result REF] [--provider PROVIDER] [--model MODEL] [QUESTION]",
    },
    Entry {
        name: "ai-panel",
        summary: "Open the native agent side panel (Ask, Workspace, Gain)",
        usage: "ai-panel",
    },
    Entry {
        name: "ai-workspace",
        summary: "Print cwd and Git context for agents",
        usage: "ai-workspace [--json]",
    },
];

const SYSTEM: &[Entry] = &[
    Entry {
        name: "ai-doctor",
        summary: "Check required tools and provider credentials",
        usage: "ai-doctor",
    },
    Entry {
        name: "ai-context",
        summary: "Show Git branch and working-tree state",
        usage: "ai-context [--prompt|--ansi|--json]",
    },
    Entry {
        name: "ai-ui",
        summary: "Open native appearance settings",
        usage: "ai-ui",
    },
    Entry {
        name: "ai-version",
        summary: "Print the OpenTerm version",
        usage: "ai-version",
    },
];

const EVIDENCE: &[Entry] = &[
    Entry {
        name: "ai-run",
        summary: "Run one program and store raw stdout/stderr evidence",
        usage: "ai-run PROGRAM [ARG ...]",
    },
    Entry {
        name: "ai-result",
        summary: "Inspect stored result metadata",
        usage: "ai-result RESULT [--json]",
    },
    Entry {
        name: "ai-expand",
        summary: "Retrieve exact stored command evidence",
        usage: "ai-expand RESULT [--stream stdout|stderr|all]",
    },
    Entry {
        name: "ai-page",
        summary: "Read a bounded page of stored evidence",
        usage: "ai-page RESULT [--stream STREAM] [--offset N] [--limit N]",
    },
    Entry {
        name: "ai-export",
        summary: "Export stored evidence to a file",
        usage: "ai-export RESULT PATH [--stream STREAM]",
    },
    Entry {
        name: "ai-pin",
        summary: "Protect a result from cleanup suggestions",
        usage: "ai-pin RESULT",
    },
    Entry {
        name: "ai-forget",
        summary: "Move a result into recoverable trash",
        usage: "ai-forget RESULT",
    },
];

const HISTORY: &[Entry] = &[
    Entry {
        name: "ai-history",
        summary: "Browse recent managed commands",
        usage: "ai-history [--limit N]",
    },
    Entry {
        name: "ai-search-history",
        summary: "Search command history locally",
        usage: "ai-search-history QUERY [--limit N]",
    },
    Entry {
        name: "ai-clean-history",
        summary: "Rank safe cleanup suggestions (does not delete)",
        usage: "ai-clean-history [--limit N]",
    },
    Entry {
        name: "ai-usage",
        summary: "Show local evidence-store statistics",
        usage: "ai-usage [--json]",
    },
];

const SECTIONS: &[Section] = &[
    Section {
        title: "Agent & guidance",
        entries: AGENT,
    },
    Section {
        title: "System & environment",
        entries: SYSTEM,
    },
    Section {
        title: "Evidence store",
        entries: EVIDENCE,
    },
    Section {
        title: "History & retention",
        entries: HISTORY,
    },
];

pub fn run(args: impl IntoIterator<Item = OsString>) -> i32 {
    let args: Vec<_> = args.into_iter().collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") || args.is_empty() {
        print_full();
        return 0;
    }
    if let Some(name) = args.first().and_then(|arg| arg.to_str()) {
        if print_one(name) {
            return 0;
        }
    }
    eprintln!(
        "error: unknown topic '{:?}'\n\nRun ai-help for the full catalog.",
        args[0]
    );
    2
}

pub fn print_summary() {
    print_header(false);
    for section in SECTIONS {
        for entry in section.entries {
            print_line(entry.name, entry.summary);
        }
    }
    for command in crate::rtk_commands::all() {
        print_line(command.name, command.description);
    }
    println!("\nEvery OpenTerm tool command starts with ai-.");
    println!("Run ai-help for full usage notes.");
}

fn print_full() {
    print_header(true);
    for section in SECTIONS {
        print_section(section);
    }
    print_rtk_section();
    print_footer();
}

fn print_one(name: &str) -> bool {
    for section in SECTIONS {
        if let Some(entry) = section.entries.iter().find(|entry| entry.name == name) {
            print_entry(entry, true);
            return true;
        }
    }
    if let Some(command) = crate::rtk_commands::find(name) {
        println!("{} — {}", command.name, command.description);
        println!("  wraps: rtk {}", command.prefix.join(" "));
        println!("  stores compressed evidence when run inside OpenTerm");
        return true;
    }
    false
}

fn print_header(detailed: bool) {
    if detailed {
        println!("OpenTerm command reference\n");
        println!("OpenTerm wraps everyday terminal work with ai-* tools.");
        println!("RTK-backed commands compress output and store encrypted evidence.\n");
    } else {
        print!("OpenTerm — AI-native terminal foundation\n\nCommands:\n");
    }
}

fn print_section(section: &Section) {
    println!("{}", section.title);
    for entry in section.entries {
        print_entry(entry, true);
    }
    println!();
}

fn print_rtk_section() {
    println!("RTK output tools (token-efficient wrappers)");
    println!("Prefer ai-rg over ai-grep. Each command preserves exit codes.\n");
    for command in crate::rtk_commands::all() {
        let usage = format!("{} [ARGS...]", command.name);
        print_line_detail(command.name, command.description, &usage);
    }
    println!();
}

fn print_entry(entry: &Entry, detailed: bool) {
    if detailed {
        print_line_detail(entry.name, entry.summary, entry.usage);
    } else {
        print_line(entry.name, entry.summary);
    }
}

fn print_line(name: &str, summary: &str) {
    println!("  {name:<18} {summary}");
}

fn print_line_detail(name: &str, summary: &str, usage: &str) {
    println!("  {name:<18} {summary}");
    println!("    {usage}");
}

fn print_footer() {
    println!("Notes:");
    println!("  • Every user-facing tool starts with ai-.");
    println!("  • ai-ask and ai-explain are read-only — they never modify files.");
    println!(
        "  • Managed commands (ai-run, RTK tools) store evidence in ~/Library/Application Support/OpenTerm/data"
    );
    println!("  • Use ai-help COMMAND for a single entry, e.g. ai-help ai-ask");
}

#[cfg(test)]
mod tests {
    use super::run;
    use std::ffi::OsString;

    #[test]
    fn full_help_exits_zero() {
        assert_eq!(run([]), 0);
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
                super::SECTIONS
                    .iter()
                    .any(|section| { section.entries.iter().any(|entry| entry.name == name) })
            );
        }
    }
}
