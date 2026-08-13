use super::AppResult;
use super::crypto::Cipher;
use super::schema;
use super::store_paths::{create_private_directory, data_root};
use super::types::{ResultRecord, SaveRequest, StoreStats, Stream};
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

pub struct Store {
    root: PathBuf,
    connection: Connection,
    cipher: Cipher,
}

impl Store {
    pub fn open() -> AppResult<Self> {
        Self::open_at(data_root()?)
    }

    pub fn open_at(root: PathBuf) -> AppResult<Self> {
        create_private_directory(&root)?;
        fs::create_dir_all(root.join("blobs"))?;
        fs::create_dir_all(root.join("trash"))?;
        let connection = Connection::open(root.join("results.sqlite3"))?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        schema::initialize(&connection)?;
        let cipher = Cipher::load(&root)?;
        Ok(Self {
            root,
            connection,
            cipher,
        })
    }

    pub fn save(&self, request: &SaveRequest) -> AppResult<String> {
        let id = result_id(request)?;
        self.write_blob(&id, "stdout", &request.stdout)?;
        self.write_blob(&id, "stderr", &request.stderr)?;
        if let Err(error) = self.insert(&id, request) {
            self.remove_active_blobs(&id);
            return Err(error);
        }
        Ok(id)
    }

    pub fn get(&self, id: &str) -> AppResult<Option<ResultRecord>> {
        let mut statement = self.connection.prepare(
            "SELECT id, command, executable, argv_json, cwd, exit_code,
             started_ms, duration_ms, stdout_bytes, stderr_bytes,
             evidence_kind, pinned, trashed_ms, access_count, last_accessed_ms
             FROM results WHERE id = ?1",
        )?;
        statement
            .query_row([id], map_record)
            .optional()
            .map_err(Into::into)
    }

    pub fn evidence(&self, record: &ResultRecord, stream: Stream) -> AppResult<Vec<u8>> {
        let bytes = match stream {
            Stream::Stdout => self.read_blob(record, "stdout"),
            Stream::Stderr => self.read_blob(record, "stderr"),
            Stream::All => {
                let mut bytes = self.read_blob(record, "stdout")?;
                bytes.extend(self.read_blob(record, "stderr")?);
                Ok(bytes)
            }
        }?;
        self.record_access(&record.id)?;
        Ok(bytes)
    }

    pub fn recent(&self, limit: usize) -> AppResult<Vec<ResultRecord>> {
        self.query_records(
            "SELECT id, command, executable, argv_json, cwd, exit_code,
             started_ms, duration_ms, stdout_bytes, stderr_bytes, evidence_kind,
             pinned, trashed_ms, access_count, last_accessed_ms FROM results
             WHERE trashed_ms IS NULL ORDER BY started_ms DESC LIMIT ?1",
            limit,
        )
    }

    pub fn search(&self, query: &str, limit: usize) -> AppResult<Vec<ResultRecord>> {
        let pattern = format!("%{query}%");
        let mut statement = self.connection.prepare(
            "SELECT id, command, executable, argv_json, cwd, exit_code,
             started_ms, duration_ms, stdout_bytes, stderr_bytes, evidence_kind,
             pinned, trashed_ms, access_count, last_accessed_ms FROM results
             WHERE trashed_ms IS NULL AND
             (command LIKE ?1 OR argv_json LIKE ?1 OR cwd LIKE ?1)
             ORDER BY started_ms DESC LIMIT ?2",
        )?;
        let records = statement
            .query_map(params![pattern, limit], map_record)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(records)
    }

    pub fn stats(&self) -> AppResult<StoreStats> {
        self.connection
            .query_row(
                "SELECT
                 coalesce(sum(trashed_ms IS NULL), 0),
                 coalesce(sum(pinned), 0),
                 coalesce(sum(trashed_ms IS NOT NULL), 0),
                 coalesce(sum(stdout_bytes + stderr_bytes), 0),
                 coalesce(sum(access_count), 0)
                 FROM results",
                [],
                |row| {
                    Ok(StoreStats {
                        active_results: row.get(0)?,
                        pinned_results: row.get(1)?,
                        trashed_results: row.get(2)?,
                        evidence_bytes: row.get(3)?,
                        total_accesses: row.get(4)?,
                    })
                },
            )
            .map_err(Into::into)
    }

    pub fn set_pinned(&self, id: &str, pinned: bool) -> AppResult<bool> {
        let changed = self.connection.execute(
            "UPDATE results SET pinned = ?1 WHERE id = ?2",
            params![pinned, id],
        )?;
        Ok(changed == 1)
    }

    pub fn trash(&self, id: &str, now_ms: u64) -> AppResult<bool> {
        let Some(record) = self.get(id)? else {
            return Ok(false);
        };
        if record.pinned || record.trashed_ms.is_some() {
            return Ok(false);
        }
        self.move_and_mark(id, false, Some(now_ms))?;
        Ok(true)
    }

    pub fn restore(&self, id: &str) -> AppResult<bool> {
        let Some(record) = self.get(id)? else {
            return Ok(false);
        };
        if record.trashed_ms.is_none() {
            return Ok(false);
        }
        self.move_and_mark(id, true, None)?;
        Ok(true)
    }

    fn insert(&self, id: &str, request: &SaveRequest) -> AppResult<()> {
        let argv = serde_json::to_string(&request.argv)?;
        self.connection.execute(
            "INSERT INTO results
             (id, command, executable, argv_json, cwd, exit_code, started_ms,
              duration_ms, stdout_bytes, stderr_bytes, evidence_kind)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                id,
                request.command,
                request.executable,
                argv,
                request.cwd,
                request.exit_code,
                request.started_ms,
                request.duration_ms,
                request.stdout.len() as u64,
                request.stderr.len() as u64,
                request.evidence_kind,
            ],
        )?;
        Ok(())
    }

    fn write_blob(&self, id: &str, stream: &str, bytes: &[u8]) -> AppResult<()> {
        let encrypted = self.cipher.encrypt(bytes)?;
        fs::write(self.blob_path(id, stream, false), encrypted)?;
        Ok(())
    }

    fn read_blob(&self, record: &ResultRecord, stream: &str) -> AppResult<Vec<u8>> {
        let trashed = record.trashed_ms.is_some();
        let encrypted = fs::read(self.blob_path(&record.id, stream, trashed))?;
        self.cipher.decrypt(&encrypted).map_err(Into::into)
    }

    fn move_and_mark(
        &self,
        id: &str,
        source_trashed: bool,
        trashed_ms: Option<u64>,
    ) -> AppResult<()> {
        self.move_blobs(id, source_trashed)?;
        let result = self.connection.execute(
            "UPDATE results SET trashed_ms = ?1 WHERE id = ?2",
            params![trashed_ms, id],
        );
        if result.is_err() {
            let _ = self.move_blobs(id, !source_trashed);
        }
        result?;
        Ok(())
    }

    fn move_blobs(&self, id: &str, source_trashed: bool) -> AppResult<()> {
        let target_trashed = !source_trashed;
        let source_out = self.blob_path(id, "stdout", source_trashed);
        let target_out = self.blob_path(id, "stdout", target_trashed);
        fs::rename(&source_out, &target_out)?;
        let source_err = self.blob_path(id, "stderr", source_trashed);
        let target_err = self.blob_path(id, "stderr", target_trashed);
        if let Err(error) = fs::rename(source_err, target_err) {
            let _ = fs::rename(target_out, source_out);
            return Err(error.into());
        }
        Ok(())
    }

    fn blob_path(&self, id: &str, stream: &str, trashed: bool) -> PathBuf {
        let directory = if trashed { "trash" } else { "blobs" };
        self.root.join(directory).join(format!("{id}.{stream}.enc"))
    }

    fn remove_active_blobs(&self, id: &str) {
        let _ = fs::remove_file(self.blob_path(id, "stdout", false));
        let _ = fs::remove_file(self.blob_path(id, "stderr", false));
    }

    fn query_records(&self, sql: &str, limit: usize) -> AppResult<Vec<ResultRecord>> {
        let mut statement = self.connection.prepare(sql)?;
        let records = statement
            .query_map([limit], map_record)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(records)
    }

    fn record_access(&self, id: &str) -> AppResult<()> {
        self.connection.execute(
            "UPDATE results SET access_count = access_count + 1,
             last_accessed_ms = ?1 WHERE id = ?2",
            params![unix_time_ms(), id],
        )?;
        Ok(())
    }
}

fn map_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<ResultRecord> {
    let argv_json: String = row.get(3)?;
    let argv = serde_json::from_str(&argv_json).unwrap_or_default();
    Ok(ResultRecord {
        id: row.get(0)?,
        command: row.get(1)?,
        executable: row.get(2)?,
        argv,
        cwd: row.get(4)?,
        exit_code: row.get(5)?,
        started_ms: row.get(6)?,
        duration_ms: row.get(7)?,
        stdout_bytes: row.get(8)?,
        stderr_bytes: row.get(9)?,
        evidence_kind: row.get(10)?,
        pinned: row.get(11)?,
        trashed_ms: row.get(12)?,
        access_count: row.get(13)?,
        last_accessed_ms: row.get(14)?,
    })
}

fn unix_time_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

fn result_id(request: &SaveRequest) -> AppResult<String> {
    let mut random = [0_u8; 16];
    getrandom::fill(&mut random)
        .map_err(|error| format!("secure random generation failed: {error}"))?;
    let mut digest = Sha256::new();
    digest.update(request.started_ms.to_le_bytes());
    digest.update(std::process::id().to_le_bytes());
    digest.update(random);
    Ok(format!("res_{}", &hex::encode(digest.finalize())[..20]))
}
