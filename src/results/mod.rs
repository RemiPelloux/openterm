mod commands;
mod crypto;
mod history_commands;
mod retention;
mod schema;
mod store;
mod store_paths;
#[cfg(test)]
mod store_tests;
mod types;

use std::error::Error;
use std::ffi::OsString;
use std::process::Output;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub(crate) use store_paths::{create_private_directory, data_root};
pub use types::SaveRequest;
pub(crate) use types::StoreStats;

type AppResult<T> = Result<T, Box<dyn Error>>;

const COMMANDS: [&str; 11] = [
    "ai-result",
    "ai-expand",
    "ai-page",
    "ai-export",
    "ai-pin",
    "ai-forget",
    "ai-restore",
    "ai-history",
    "ai-search-history",
    "ai-clean-history",
    "ai-usage",
];

pub fn is_command(name: &str) -> bool {
    COMMANDS.contains(&name)
}

pub fn run(name: &str, args: impl IntoIterator<Item = OsString>) -> i32 {
    let args = args.into_iter().collect::<Vec<_>>();
    if history_commands::is_command(name) {
        return history_commands::run(name, &args);
    }
    commands::run(name, &args)
}

pub(crate) fn cockpit_stats() -> Option<StoreStats> {
    store::Store::open().ok()?.stats().ok()
}

pub fn preview_latest_result() -> AppResult<String> {
    let store = store::Store::open()?;
    let record = store
        .recent(1)?
        .into_iter()
        .next()
        .ok_or("no stored results yet — run an ai-* command first")?;
    Ok(preview_record(&store, &record))
}

pub fn preview_result(id: &str) -> AppResult<String> {
    let store = store::Store::open()?;
    let record = store
        .get(id)?
        .ok_or_else(|| format!("result '{id}' was not found"))?;
    Ok(preview_record(&store, &record))
}

fn preview_record(store: &store::Store, record: &types::ResultRecord) -> String {
    let evidence = store
        .evidence(record, types::Stream::All)
        .unwrap_or_default();
    let end = evidence.len().min(RESULT_PREVIEW_BYTES);
    let text = String::from_utf8_lossy(&evidence[..end]);
    let preview = if evidence.len() > RESULT_PREVIEW_BYTES {
        format!("{text}\n… ({RESULT_PREVIEW_BYTES} byte preview)")
    } else {
        text.into_owned()
    };
    format!(
        "OpenTerm result context (read-only):\n\
         id: {}\n\
         command: {} (exit {})\n\
         cwd: {}\n\
         evidence: {}\n\
         preview:\n{preview}",
        record.id, record.command, record.exit_code, record.cwd, record.evidence_kind
    )
}

const RESULT_PREVIEW_BYTES: usize = 8_192;

pub fn save_and_announce(request: &SaveRequest) {
    match store::Store::open().and_then(|store| store.save(request)) {
        Ok(id) => eprintln!("openterm: result {id} (expand with: ai-expand {id})"),
        Err(error) => eprintln!("openterm: warning: evidence was not stored: {error}"),
    }
}

impl SaveRequest {
    pub fn rtk(
        spec: &crate::rtk_commands::RtkCommand,
        child_args: &[OsString],
        started: Instant,
        output: &Output,
    ) -> Self {
        let elapsed = started.elapsed();
        let mut command_parts: Vec<String> = spec
            .prefix
            .iter()
            .map(|value| (*value).to_owned())
            .collect();
        command_parts.extend(
            child_args
                .iter()
                .map(|value| value.to_string_lossy().into_owned()),
        );
        Self {
            command: spec.name.to_owned(),
            executable: "rtk".to_owned(),
            argv: command_parts,
            cwd: current_directory(),
            exit_code: crate::rtk_commands::output_exit_code(output),
            started_ms: unix_time_ms().saturating_sub(elapsed_ms(elapsed)),
            duration_ms: elapsed_ms(elapsed),
            stdout: output.stdout.clone(),
            stderr: output.stderr.clone(),
            evidence_kind: "rtk-filtered-output".to_owned(),
        }
    }

    pub fn raw(child_args: &[OsString], started: Instant, output: &Output) -> Self {
        let elapsed = started.elapsed();
        let command_parts = child_args
            .iter()
            .map(|value| value.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        Self {
            command: "ai-run".to_owned(),
            executable: command_parts.first().cloned().unwrap_or_default(),
            argv: command_parts,
            cwd: current_directory(),
            exit_code: crate::rtk_commands::output_exit_code(output),
            started_ms: unix_time_ms().saturating_sub(elapsed_ms(elapsed)),
            duration_ms: elapsed_ms(elapsed),
            stdout: output.stdout.clone(),
            stderr: output.stderr.clone(),
            evidence_kind: "raw-command-output".to_owned(),
        }
    }
}

fn current_directory() -> String {
    std::env::current_dir()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn unix_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

fn elapsed_ms(duration: Duration) -> u64 {
    duration.as_millis().try_into().unwrap_or(u64::MAX)
}
