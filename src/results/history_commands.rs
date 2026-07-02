use super::AppResult;
use super::retention;
use super::store::Store;
use super::types::ResultRecord;
use std::ffi::OsString;
use std::time::{SystemTime, UNIX_EPOCH};

const COMMANDS: [&str; 4] = [
    "ai-history",
    "ai-search-history",
    "ai-clean-history",
    "ai-usage",
];
const DEFAULT_LIMIT: usize = 12;
const MAX_LIMIT: usize = 100;

pub fn is_command(name: &str) -> bool {
    COMMANDS.contains(&name)
}

pub fn run(name: &str, args: &[OsString]) -> i32 {
    match dispatch(name, args) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("error: {error}");
            2
        }
    }
}

fn dispatch(name: &str, args: &[OsString]) -> AppResult<()> {
    let store = Store::open()?;
    match name {
        "ai-history" => history(&store, args),
        "ai-search-history" => search(&store, args),
        "ai-clean-history" => cleanup(&store, args),
        "ai-usage" => usage(&store, args),
        _ => Err("unknown history command".into()),
    }
}

fn history(store: &Store, args: &[OsString]) -> AppResult<()> {
    let limit = parse_limit(args)?;
    let records = store.recent(limit)?;
    print_records(&records, unix_time_ms());
    Ok(())
}

fn search(store: &Store, args: &[OsString]) -> AppResult<()> {
    let query = args
        .first()
        .and_then(|value| value.to_str())
        .ok_or("usage: ai-search-history QUERY [--limit N]")?;
    let limit = parse_limit(&args[1..])?;
    let records = store.search(query, limit)?;
    print_records(&records, unix_time_ms());
    Ok(())
}

fn cleanup(store: &Store, args: &[OsString]) -> AppResult<()> {
    let limit = parse_limit(args)?;
    let now = unix_time_ms();
    let mut records = store.recent(MAX_LIMIT)?;
    records.retain(|record| !record.pinned);
    records.sort_by(|left, right| {
        retention::score(left, now)
            .value
            .total_cmp(&retention::score(right, now).value)
    });
    records.truncate(limit);
    println!("Cleanup suggestions only — nothing was deleted.\n");
    print_cleanup_records(&records, now);
    println!("\nReview exact evidence, then use ai-forget RESULT if appropriate.");
    Ok(())
}

fn print_cleanup_records(records: &[ResultRecord], now_ms: u64) {
    if records.is_empty() {
        println!("No cleanup candidates.");
        return;
    }
    println!("KEEP   FRESH  USE    DIAG   COMMAND                     RESULT");
    for record in records {
        let score = retention::score(record, now_ms);
        println!(
            "{:>3.0}%   {:>3.0}%   {:>3.0}%   {:>3.0}%   {:<27} {}",
            score.value * 100.0,
            score.freshness * 100.0,
            score.access * 100.0,
            score.diagnostic * 100.0,
            compact_command(record, 27),
            record.id
        );
    }
}

fn usage(store: &Store, args: &[OsString]) -> AppResult<()> {
    let stats = store.stats()?;
    if args.first().is_some_and(|value| value == "--json") {
        println!("{}", serde_json::to_string_pretty(&stats)?);
        return Ok(());
    }
    println!("OpenTerm local evidence");
    println!("  active     {}", stats.active_results);
    println!("  pinned     {}", stats.pinned_results);
    println!("  trash      {}", stats.trashed_results);
    println!("  evidence   {}", human_bytes(stats.evidence_bytes));
    println!("  expansions {}", stats.total_accesses);
    Ok(())
}

fn print_records(records: &[ResultRecord], now_ms: u64) {
    if records.is_empty() {
        println!("No matching OpenTerm history yet.");
        return;
    }
    println!("SCORE  AGE      EXIT  COMMAND                         RESULT");
    for record in records {
        let score = retention::score(record, now_ms);
        let command = compact_command(record, 30);
        println!(
            "{:>3.0}%   {:<8} {:>4}  {:<30}  {}",
            score.value * 100.0,
            relative_age(record.started_ms, now_ms),
            record.exit_code,
            command,
            record.id
        );
    }
}

fn compact_command(record: &ResultRecord, limit: usize) -> String {
    let joined = record.argv.join(" ");
    if joined.chars().count() <= limit {
        return joined;
    }
    let prefix = joined
        .chars()
        .take(limit.saturating_sub(1))
        .collect::<String>();
    format!("{prefix}…")
}

fn parse_limit(args: &[OsString]) -> AppResult<usize> {
    if args.is_empty() {
        return Ok(DEFAULT_LIMIT);
    }
    if args.len() != 2 || args[0] != "--limit" {
        return Err("expected --limit N".into());
    }
    let value = args[1].to_str().ok_or("limit must be UTF-8")?;
    let limit = value.parse::<usize>()?;
    if !(1..=MAX_LIMIT).contains(&limit) {
        return Err(format!("limit must be between 1 and {MAX_LIMIT}").into());
    }
    Ok(limit)
}

fn relative_age(started_ms: u64, now_ms: u64) -> String {
    let seconds = now_ms.saturating_sub(started_ms) / 1_000;
    match seconds {
        0..=59 => format!("{seconds}s"),
        60..=3_599 => format!("{}m", seconds / 60),
        3_600..=86_399 => format!("{}h", seconds / 3_600),
        _ => format!("{}d", seconds / 86_400),
    }
}

fn human_bytes(bytes: u64) -> String {
    const UNITS: [(&str, u64); 5] = [
        ("TiB", 1_u64 << 40),
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

fn unix_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::{compact_command, human_bytes, parse_limit, relative_age};
    use crate::results::types::ResultRecord;
    use std::ffi::OsString;

    #[test]
    fn validates_limits() {
        assert_eq!(parse_limit(&[]).unwrap(), 12);
        assert!(parse_limit(&[OsString::from("--limit"), OsString::from("0")]).is_err());
    }

    #[test]
    fn formats_compact_history() {
        let mut record = sample_record();
        record.argv = vec!["a-very-long-program-name".into(), "argument".into()];
        assert!(compact_command(&record, 12).ends_with('…'));
        assert_eq!(relative_age(0, 3_600_000), "1h");
        assert_eq!(human_bytes(1_024), "1.0 KiB");
    }

    fn sample_record() -> ResultRecord {
        ResultRecord {
            id: "res_test".into(),
            command: "ai-run".into(),
            executable: "test".into(),
            argv: vec!["test".into()],
            cwd: "/tmp".into(),
            exit_code: 0,
            started_ms: 0,
            duration_ms: 1,
            stdout_bytes: 0,
            stderr_bytes: 0,
            evidence_kind: "raw-command-output".into(),
            pinned: false,
            trashed_ms: None,
            access_count: 0,
            last_accessed_ms: None,
        }
    }
}
