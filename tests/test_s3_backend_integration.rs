//! Real integration tests for `S3Backend` against a live S3-compatible
//! service (MinIO).
//!
//! These are `#[ignore]`d by default so a plain `cargo test` never requires
//! Docker/MinIO to be running. To run them for real:
//!
//!   docker run -d --name pyroboreplay-test-minio -p 9000:9000 -p 9001:9001 \
//!       -e MINIO_ROOT_USER=minioadmin -e MINIO_ROOT_PASSWORD=minioadmin \
//!       minio/minio server /data --console-address ":9001"
//!   AWS_ACCESS_KEY_ID=minioadmin AWS_SECRET_ACCESS_KEY=minioadmin \
//!   cargo test --test test_s3_backend_integration -- --ignored --test-threads=1

use pyroboreplay::storage::{S3Backend, StorageBackend, StorageError};

fn connection_string() -> String {
    let bucket = format!("pyroboreplay-test-{}", std::process::id());
    std::env::var("PYROBOREPLAY_TEST_S3_URL").unwrap_or_else(|_| {
        format!("s3://{}?endpoint=http://localhost:9000&path_style=true", bucket)
    })
}

fn connected_backend() -> S3Backend {
    // Ensure credentials are set for the local MinIO instance if the caller
    // didn't already export them (matches the docker run command above).
    if std::env::var("AWS_ACCESS_KEY_ID").is_err() {
        std::env::set_var("AWS_ACCESS_KEY_ID", "minioadmin");
    }
    if std::env::var("AWS_SECRET_ACCESS_KEY").is_err() {
        std::env::set_var("AWS_SECRET_ACCESS_KEY", "minioadmin");
    }
    if std::env::var("AWS_REGION").is_err() {
        std::env::set_var("AWS_REGION", "us-east-1");
    }

    let mut backend = S3Backend::new(&connection_string());
    backend
        .connect()
        .expect("failed to connect to local test MinIO (is the container running? see module docs)");
    backend
}

#[test]
#[ignore]
fn test_connect_creates_bucket_and_reports_stats() {
    let backend = connected_backend();
    let stats = backend.get_stats().expect("get_stats should succeed once connected");
    assert!(stats.connected);
}

#[test]
#[ignore]
fn test_store_and_retrieve_mission_round_trip() {
    let backend = connected_backend();
    let mission_id = format!("test-mission-{}", uuid::Uuid::new_v4());
    let payload = r#"{"name":"integration test mission","events":3}"#;

    backend.store_mission(&mission_id, payload).expect("store_mission failed");
    let retrieved = backend.retrieve_mission(&mission_id).expect("retrieve_mission failed");
    assert_eq!(retrieved, payload);

    assert!(backend.mission_exists(&mission_id).expect("mission_exists failed"));

    let updated_payload = r#"{"name":"integration test mission","events":5}"#;
    backend.store_mission(&mission_id, updated_payload).expect("store_mission (update) failed");
    let retrieved_again = backend.retrieve_mission(&mission_id).expect("retrieve_mission after update failed");
    assert_eq!(retrieved_again, updated_payload);

    backend.delete_mission(&mission_id).expect("delete_mission failed");
    assert!(!backend.mission_exists(&mission_id).expect("mission_exists after delete failed"));
}

#[test]
#[ignore]
fn test_retrieve_missing_mission_returns_not_found() {
    let backend = connected_backend();
    let mission_id = format!("does-not-exist-{}", uuid::Uuid::new_v4());
    match backend.retrieve_mission(&mission_id) {
        Err(StorageError::NotFound(_)) => {}
        other => panic!("expected NotFound, got {:?}", other),
    }
}

#[test]
#[ignore]
fn test_store_and_retrieve_events_and_report_then_delete_mission() {
    let backend = connected_backend();
    let mission_id = format!("test-mission-events-{}", uuid::Uuid::new_v4());
    backend.store_mission(&mission_id, r#"{"name":"events test"}"#).expect("store_mission failed");

    for i in 0..5 {
        let event_id = format!("evt-{}", i);
        let data = format!(r#"{{"index":{}}}"#, i);
        backend.store_event(&mission_id, &event_id, &data).expect("store_event failed");
    }

    let evt2 = backend.retrieve_event(&mission_id, "evt-2").expect("retrieve_event failed");
    assert_eq!(evt2, r#"{"index":2}"#);

    let report = r#"{"summary":"all good"}"#;
    backend.store_report(&mission_id, report).expect("store_report failed");
    let retrieved_report = backend.retrieve_report(&mission_id).expect("retrieve_report failed");
    assert_eq!(retrieved_report, report);

    // delete_mission must remove the mission object, every event object, and
    // the report object (there's no FK cascade in S3 - this proves the
    // prefix list+bulk-delete logic actually walks all of them).
    backend.delete_mission(&mission_id).expect("delete_mission failed");
    match backend.retrieve_event(&mission_id, "evt-2") {
        Err(StorageError::NotFound(_)) => {}
        other => panic!("expected event to be deleted along with mission, got {:?}", other),
    }
    match backend.retrieve_report(&mission_id) {
        Err(StorageError::NotFound(_)) => {}
        other => panic!("expected report to be deleted along with mission, got {:?}", other),
    }
    match backend.retrieve_mission(&mission_id) {
        Err(StorageError::NotFound(_)) => {}
        other => panic!("expected mission to be deleted, got {:?}", other),
    }
}

#[test]
#[ignore]
fn test_list_missions_respects_limit() {
    let backend = connected_backend();
    let prefix = format!("list-test-{}", uuid::Uuid::new_v4());
    for i in 0..3 {
        let mission_id = format!("{}-{}", prefix, i);
        backend.store_mission(&mission_id, "{}").expect("store_mission failed");
    }

    let all = backend.list_missions(None).expect("list_missions(None) failed");
    let matching: Vec<_> = all.iter().filter(|m| m.starts_with(&prefix)).collect();
    assert_eq!(matching.len(), 3);

    let limited = backend.list_missions(Some(1)).expect("list_missions(Some(1)) failed");
    assert_eq!(limited.len(), 1);

    for i in 0..3 {
        let mission_id = format!("{}-{}", prefix, i);
        let _ = backend.delete_mission(&mission_id);
    }
}

#[test]
#[ignore]
fn test_close_disconnects_backend() {
    let backend = connected_backend();
    backend.close().expect("close should succeed while connected");
    match backend.get_stats() {
        Err(StorageError::ConnectionFailed(_)) => {}
        other => panic!("expected ConnectionFailed after close, got {:?}", other),
    }
}
