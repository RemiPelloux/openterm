pub struct Entry {
    pub name: &'static str,
    pub summary: &'static str,
    pub usage: &'static str,
}

pub struct Section {
    pub title: &'static str,
    pub entries: &'static [Entry],
}

const AGENT: &[Entry] = &[
    Entry {
        name: "ai-help",
        summary: "List every OpenTerm command with usage notes",
        usage: "ai-help [--markdown|--plain] [COMMAND]",
    },
    Entry {
        name: "ai-ask",
        summary: "Ask an LLM for read-only terminal guidance",
        usage: "ai-ask [--provider codex|openai|claude|mistral] [--model MODEL] [--workspace] [--result REF] QUESTION",
    },
    Entry {
        name: "ai-connect",
        summary: "Connect to a provider (arrow keys) and pick a model",
        usage: "ai-connect [--status]",
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
    Entry {
        name: "ai-restore",
        summary: "Restore a result from recoverable trash",
        usage: "ai-restore RESULT",
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

pub const SECTIONS: &[Section] = &[
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
