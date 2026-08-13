use super::AppResult;
use super::store::Store;
use super::types::{ResultRecord, Stream};
use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const USAGE: &str = "usage:
  ai-result RESULT [--json]
  ai-expand RESULT [--stream stdout|stderr|all]
  ai-page RESULT [--stream stdout|stderr|all] [--offset N] [--limit N]
  ai-export RESULT PATH [--stream stdout|stderr|all]
  ai-pin RESULT
  ai-forget RESULT
  ai-restore RESULT
";
const DEFAULT_PAGE_BYTES: usize = 16_384;

pub fn run(name: &str, args: &[OsString]) -> i32 {
    match dispatch(name, args) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("error: {error}\n\n{USAGE}");
            2
        }
    }
}

fn dispatch(name: &str, args: &[OsString]) -> AppResult<()> {
    let store = Store::open()?;
    match name {
        "ai-result" => show_result(&store, args),
        "ai-expand" => expand(&store, args),
        "ai-page" => page(&store, args),
        "ai-export" => export(&store, args),
        "ai-pin" => pin(&store, args),
        "ai-forget" => forget(&store, args),
        "ai-restore" => restore(&store, args),
        _ => Err("unknown result command".into()),
    }
}

fn show_result(store: &Store, args: &[OsString]) -> AppResult<()> {
    let id = required_arg(args, 0, "result reference")?;
    let record = require_record(store, id)?;
    if args.get(1).is_some_and(|arg| arg == "--json") {
        println!("{}", serde_json::to_string_pretty(&record)?);
        return Ok(());
    }
    print_record(&record);
    Ok(())
}

fn print_record(record: &ResultRecord) {
    println!(
        "{} {} (exit {})",
        record.id, record.command, record.exit_code
    );
    println!("cwd: {}", record.cwd);
    println!("evidence: {}", record.evidence_kind);
    println!(
        "bytes: stdout={} stderr={}",
        record.stdout_bytes, record.stderr_bytes
    );
    println!("duration: {}ms", record.duration_ms);
    println!("pinned: {}", record.pinned);
    println!("trashed: {}", record.trashed_ms.is_some());
}

fn expand(store: &Store, args: &[OsString]) -> AppResult<()> {
    let id = required_arg(args, 0, "result reference")?;
    let stream = parse_stream(args, 1)?;
    let record = require_record(store, id)?;
    let evidence = store.evidence(&record, stream)?;
    std::io::stdout().write_all(&evidence)?;
    Ok(())
}

fn page(store: &Store, args: &[OsString]) -> AppResult<()> {
    let id = required_arg(args, 0, "result reference")?;
    let options = PageOptions::parse(&args[1..])?;
    let record = require_record(store, id)?;
    let evidence = store.evidence(&record, options.stream)?;
    let start = options.offset.min(evidence.len());
    let end = start.saturating_add(options.limit).min(evidence.len());
    std::io::stdout().write_all(&evidence[start..end])?;
    Ok(())
}

fn export(store: &Store, args: &[OsString]) -> AppResult<()> {
    let id = required_arg(args, 0, "result reference")?;
    let path = PathBuf::from(required_arg(args, 1, "export path")?);
    let stream = parse_stream(args, 2)?;
    let record = require_record(store, id)?;
    let evidence = store.evidence(&record, stream)?;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    file.write_all(&evidence)?;
    println!("exported {id} to {}", path.display());
    Ok(())
}

fn pin(store: &Store, args: &[OsString]) -> AppResult<()> {
    let id = required_arg(args, 0, "result reference")?;
    if !store.set_pinned(id, true)? {
        return Err(format!("result '{id}' was not found").into());
    }
    println!("pinned {id}");
    Ok(())
}

fn forget(store: &Store, args: &[OsString]) -> AppResult<()> {
    let id = required_arg(args, 0, "result reference")?;
    if !store.trash(id, unix_time_ms())? {
        return Err(
            format!("result '{id}' was not found, is pinned, or is already trashed").into(),
        );
    }
    println!("moved {id} to recoverable trash");
    Ok(())
}

fn restore(store: &Store, args: &[OsString]) -> AppResult<()> {
    let id = required_arg(args, 0, "result reference")?;
    if !store.restore(id)? {
        return Err(format!("result '{id}' was not found or is not trashed").into());
    }
    println!("restored {id}");
    Ok(())
}

fn require_record(store: &Store, id: &str) -> AppResult<ResultRecord> {
    store
        .get(id)?
        .ok_or_else(|| format!("result '{id}' was not found").into())
}

fn required_arg<'a>(args: &'a [OsString], index: usize, name: &str) -> AppResult<&'a str> {
    args.get(index)
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("{name} is required and must be UTF-8").into())
}

fn parse_stream(args: &[OsString], start: usize) -> AppResult<Stream> {
    if args.len() == start {
        return Ok(Stream::All);
    }
    if args.get(start).is_none_or(|arg| arg != "--stream") {
        return Err("expected --stream stdout|stderr|all".into());
    }
    let value = required_arg(args, start + 1, "stream")?;
    Stream::parse(value).ok_or_else(|| format!("unknown stream '{value}'").into())
}

struct PageOptions {
    stream: Stream,
    offset: usize,
    limit: usize,
}

impl PageOptions {
    fn parse(args: &[OsString]) -> AppResult<Self> {
        let mut options = Self {
            stream: Stream::All,
            offset: 0,
            limit: DEFAULT_PAGE_BYTES,
        };
        let mut index = 0;
        while index < args.len() {
            index += options.apply(args, index)?;
        }
        Ok(options)
    }

    fn apply(&mut self, args: &[OsString], index: usize) -> AppResult<usize> {
        let flag = required_arg(args, index, "page option")?;
        let value = required_arg(args, index + 1, "page option value")?;
        match flag {
            "--stream" => self.stream = Stream::parse(value).ok_or("invalid stream")?,
            "--offset" => self.offset = value.parse()?,
            "--limit" => self.limit = value.parse()?,
            _ => return Err(format!("unknown page option '{flag}'").into()),
        }
        Ok(2)
    }
}

fn unix_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}
