use std::io::{self, IsTerminal, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const FRAME_MS: u64 = 80;
const FRAMES: [&str; 8] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];
const TEAL: &str = "\x1b[38;5;45m";
const CYAN: &str = "\x1b[38;5;81m";
const MUTED: &str = "\x1b[38;5;238m";
const RESET: &str = "\x1b[0m";

pub struct FirstWordWait {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
    shown: bool,
}

impl FirstWordWait {
    pub fn start() -> Self {
        if !io::stdout().is_terminal() {
            return Self::idle();
        }
        if reduced_motion() {
            print_frame("•");
            return Self {
                stop: Arc::new(AtomicBool::new(true)),
                handle: None,
                shown: true,
            };
        }
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let handle = thread::spawn(move || spin(flag.as_ref()));
        Self {
            stop,
            handle: Some(handle),
            shown: true,
        }
    }

    pub fn clear(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        if !self.shown {
            return;
        }
        self.shown = false;
        print!("\r\x1b[K");
        let _ = io::stdout().flush();
    }

    fn idle() -> Self {
        Self {
            stop: Arc::new(AtomicBool::new(true)),
            handle: None,
            shown: false,
        }
    }
}

impl Drop for FirstWordWait {
    fn drop(&mut self) {
        self.clear();
    }
}

fn spin(stop: &AtomicBool) {
    let mut index = 0;
    while !stop.load(Ordering::Relaxed) {
        print_frame(FRAMES[index]);
        index = (index + 1) % FRAMES.len();
        thread::sleep(Duration::from_millis(FRAME_MS));
    }
}

fn print_frame(frame: &str) {
    let color = crate::markdown::color_enabled();
    print!(
        "\r  {}▐●‿●▌{}  {}{frame}{} {}thinking…{}",
        paint(color, TEAL),
        paint(color, RESET),
        paint(color, CYAN),
        paint(color, RESET),
        paint(color, MUTED),
        paint(color, RESET)
    );
    let _ = io::stdout().flush();
}

fn reduced_motion() -> bool {
    std::env::var("OPENTERM_REDUCED_MOTION").is_ok_and(|value| value != "0")
}

fn paint(enabled: bool, value: &'static str) -> &'static str {
    if enabled { value } else { "" }
}

#[cfg(test)]
mod tests {
    use super::FirstWordWait;

    #[test]
    fn clear_is_idempotent() {
        let mut wait = FirstWordWait::idle();
        wait.clear();
        wait.clear();
    }
}
