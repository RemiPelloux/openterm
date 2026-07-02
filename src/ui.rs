use crate::results::StoreStats;
use std::io::{IsTerminal, Write};
use std::time::Duration;

const DEFAULT_ART: [&str; 7] = [
    "           ▄▄           ",
    "     ▗▄▄▄▄▄██▄▄▄▄▄▖     ",
    "     ▐ ────────── ▌     ",
    "     ▐   ●    ●   ▌     ",
    "     ▐    ╰──╯    ▌     ",
    "     ▝▀▀▀▀▀▀▀▀▀▀▀▀▘     ",
    "       ▝▘      ▝▘       ",
];
const ARMS_UP_ART: [&str; 7] = [
    "           ▄▄           ",
    "  ╲  ▗▄▄▄▄▄██▄▄▄▄▄▖  ╱  ",
    "   ╲ ▐ ────────── ▌ ╱   ",
    "     ▐   ●    ●   ▌     ",
    "     ▐    ╰──╯    ▌     ",
    "     ▝▀▀▀▀▀▀▀▀▀▀▀▀▘     ",
    "       ▝▘      ▝▘       ",
];
const FRAME_MS: u64 = 60;
const JUMP_WAVE: [(Pose, bool, u8); 6] = [
    (Pose::Default, true, 2),
    (Pose::ArmsUp, false, 3),
    (Pose::Default, false, 1),
    (Pose::Default, true, 2),
    (Pose::ArmsUp, false, 3),
    (Pose::Default, false, 1),
];

#[derive(Clone, Copy)]
enum Pose {
    Default,
    ArmsUp,
}

pub fn show_cockpit() -> i32 {
    let color = color_enabled();
    let stats = crate::results::cockpit_stats().unwrap_or_default();
    print!("{}", cockpit(color, &stats));
    0
}

pub fn animate_mascot() -> i32 {
    if !std::io::stdout().is_terminal() || reduced_motion() {
        print_art(Pose::Default, false);
        return 0;
    }
    let mut first_frame = true;
    for (pose, crouched, count) in JUMP_WAVE {
        for _ in 0..count {
            if !first_frame {
                print!("\x1b[8A\r\x1b[J");
            }
            print_art(pose, crouched);
            let _ = std::io::stdout().flush();
            std::thread::sleep(Duration::from_millis(FRAME_MS));
            first_frame = false;
        }
    }
    print!("\x1b[8A\r\x1b[J");
    print_art(Pose::Default, false);
    0
}

fn cockpit(color: bool, stats: &StoreStats) -> String {
    let accent = ansi(color, "\x1b[38;5;81m");
    let glow = ansi(color, "\x1b[38;5;45m");
    let strong = ansi(color, "\x1b[1m");
    let muted = ansi(color, "\x1b[38;5;238m");
    let reset = ansi(color, "\x1b[0m");
    format!(
        "\n{accent}{}{reset}\n\n{strong}{glow}OpenTerm{reset} {muted}agent-first · token-aware · local-first{reset}\n\n  {strong}{:<5}{reset} results   {strong}{:<5}{reset} pinned   {strong}{:<9}{reset} evidence\n\n{accent}Agent{reset}\n  ai-panel                     agent side panel\n  ai-ask --workspace \"…\"       cwd + Git aware guidance\n  ai-explain --last            explain latest output\n  ai-workspace                 print agent context pack\n\n{accent}Search{reset}\n  ai-rg PATTERN                fast ripgrep (prefer over ai-grep)\n  ai-read · ai-ls · ai-json    compact inspection\n  ai-gain                      RTK token savings\n\n{accent}Recall{reset}\n  ai-history · ai-search-history · ai-clean-history\n\n{muted}openterm help · ai-ui · ai-doctor{reset}\n\n",
        DEFAULT_ART.join("\n"),
        stats.active_results,
        stats.pinned_results,
        human_bytes(stats.evidence_bytes)
    )
}

fn print_art(pose: Pose, crouched: bool) {
    let art = match pose {
        Pose::Default => &DEFAULT_ART,
        Pose::ArmsUp => &ARMS_UP_ART,
    };
    if crouched {
        println!();
    }
    println!("{}", art.join("\n"));
    if !crouched {
        println!();
    }
}

fn color_enabled() -> bool {
    std::io::stdout().is_terminal()
        && std::env::var_os("NO_COLOR").is_none()
        && std::env::var("TERM").is_ok_and(|term| term != "dumb")
}

fn reduced_motion() -> bool {
    std::env::var("OPENTERM_REDUCED_MOTION").is_ok_and(|value| value != "0")
}

fn ansi(enabled: bool, value: &'static str) -> &'static str {
    if enabled { value } else { "" }
}

fn human_bytes(bytes: u64) -> String {
    const UNITS: [(&str, u64); 4] = [
        ("GiB", 1_u64 << 30),
        ("MiB", 1_u64 << 20),
        ("KiB", 1_u64 << 10),
        ("B", 1),
    ];
    for (name, size) in UNITS {
        if bytes >= size || size == 1 {
            let whole = bytes / size;
            let decimal = bytes % size * 10 / size;
            return format!("{whole}.{decimal} {name}");
        }
    }
    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::cockpit;
    use crate::results::StoreStats;

    #[test]
    fn cockpit_is_accessible_without_color() {
        let rendered = cockpit(false, &StoreStats::default());
        assert!(rendered.contains("OpenTerm"));
        assert!(rendered.contains("ai-panel"));
        assert!(!rendered.contains("ai-history") || rendered.contains("ai-search-history"));
        assert!(!rendered.contains("\x1b["));
    }
}
