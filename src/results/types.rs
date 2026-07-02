use serde::Serialize;

pub struct SaveRequest {
    pub command: String,
    pub executable: String,
    pub argv: Vec<String>,
    pub cwd: String,
    pub exit_code: i32,
    pub started_ms: u64,
    pub duration_ms: u64,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub evidence_kind: String,
}

#[derive(Debug, Serialize)]
pub struct ResultRecord {
    pub id: String,
    pub command: String,
    pub executable: String,
    pub argv: Vec<String>,
    pub cwd: String,
    pub exit_code: i32,
    pub started_ms: u64,
    pub duration_ms: u64,
    pub stdout_bytes: u64,
    pub stderr_bytes: u64,
    pub evidence_kind: String,
    pub pinned: bool,
    pub trashed_ms: Option<u64>,
    pub access_count: u64,
    pub last_accessed_ms: Option<u64>,
}

#[derive(Clone, Copy)]
pub enum Stream {
    Stdout,
    Stderr,
    All,
}

#[derive(Default, Serialize)]
pub struct StoreStats {
    pub active_results: u64,
    pub pinned_results: u64,
    pub trashed_results: u64,
    pub evidence_bytes: u64,
    pub total_accesses: u64,
}

impl Stream {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "stdout" => Some(Self::Stdout),
            "stderr" => Some(Self::Stderr),
            "all" => Some(Self::All),
            _ => None,
        }
    }
}
