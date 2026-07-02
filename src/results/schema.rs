use super::AppResult;
use rusqlite::Connection;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS results (
  id TEXT PRIMARY KEY,
  command TEXT NOT NULL,
  executable TEXT NOT NULL,
  argv_json TEXT NOT NULL,
  cwd TEXT NOT NULL,
  exit_code INTEGER NOT NULL,
  started_ms INTEGER NOT NULL,
  duration_ms INTEGER NOT NULL,
  stdout_bytes INTEGER NOT NULL,
  stderr_bytes INTEGER NOT NULL,
  evidence_kind TEXT NOT NULL,
  pinned INTEGER NOT NULL DEFAULT 0,
  trashed_ms INTEGER,
  access_count INTEGER NOT NULL DEFAULT 0,
  last_accessed_ms INTEGER
);
CREATE INDEX IF NOT EXISTS results_started ON results(started_ms DESC);
";

pub fn initialize(connection: &Connection) -> AppResult<()> {
    connection.execute_batch(SCHEMA)?;
    ensure_column(connection, "access_count", "INTEGER NOT NULL DEFAULT 0")?;
    ensure_column(connection, "last_accessed_ms", "INTEGER")?;
    connection.pragma_update(None, "user_version", 1)?;
    Ok(())
}

fn ensure_column(connection: &Connection, name: &str, declaration: &str) -> AppResult<()> {
    let mut statement = connection.prepare("PRAGMA table_info(results)")?;
    let names = statement
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;
    if !names.iter().any(|existing| existing == name) {
        connection.execute(
            &format!("ALTER TABLE results ADD COLUMN {name} {declaration}"),
            [],
        )?;
    }
    Ok(())
}
