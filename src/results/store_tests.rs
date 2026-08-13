use super::store::Store;
use super::types::{SaveRequest, Stream};

#[test]
fn saves_and_reads_encrypted_evidence() {
    let root = test_root();
    let store = Store::open_at(root.clone()).expect("store");
    let id = store.save(&request()).expect("save");
    let record = store.get(&id).expect("get").expect("record");
    assert_eq!(store.evidence(&record, Stream::Stdout).unwrap(), b"hello");
    let blob = std::fs::read(root.join("blobs").join(format!("{id}.stdout.enc"))).unwrap();
    assert!(!blob.windows(5).any(|window| window == b"hello"));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn tracks_search_access_and_usage() {
    let root = test_root();
    let store = Store::open_at(root.clone()).expect("store");
    let id = store.save(&request()).expect("save");
    assert_eq!(store.search("ai-test", 5).unwrap().len(), 1);
    let record = store.get(&id).unwrap().unwrap();
    store.evidence(&record, Stream::Stdout).unwrap();
    let stats = store.stats().unwrap();
    assert_eq!(stats.active_results, 1);
    assert_eq!(stats.total_accesses, 1);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn pinned_results_cannot_be_trashed() {
    let root = test_root();
    let store = Store::open_at(root.clone()).expect("store");
    let id = store.save(&request()).expect("save");
    assert!(store.set_pinned(&id, true).unwrap());
    assert!(!store.trash(&id, 42).unwrap());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn trashed_results_can_be_restored() {
    let root = test_root();
    let store = Store::open_at(root.clone()).expect("store");
    let id = store.save(&request()).expect("save");

    assert!(!store.restore(&id).unwrap());
    assert!(store.trash(&id, 42).unwrap());
    assert!(store.get(&id).unwrap().unwrap().trashed_ms.is_some());
    assert!(store.restore(&id).unwrap());

    let record = store.get(&id).unwrap().unwrap();
    assert!(record.trashed_ms.is_none());
    assert_eq!(store.evidence(&record, Stream::Stdout).unwrap(), b"hello");
    assert!(!store.restore("res_missing").unwrap());
    std::fs::remove_dir_all(root).unwrap();
}

fn request() -> SaveRequest {
    SaveRequest {
        command: "ai-test".into(),
        executable: "test".into(),
        argv: vec!["test".into()],
        cwd: "/tmp".into(),
        exit_code: 0,
        started_ms: 1,
        duration_ms: 2,
        stdout: b"hello".to_vec(),
        stderr: b"warning".to_vec(),
        evidence_kind: "test".into(),
    }
}

fn test_root() -> std::path::PathBuf {
    let mut bytes = [0_u8; 8];
    getrandom::fill(&mut bytes).unwrap();
    std::env::temp_dir().join(format!("openterm-test-{}", hex::encode(bytes)))
}
