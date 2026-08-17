//! Real integration tests for `PostgresBackend` against a live PostgreSQL
//! server.
//!
//! These are `#[ignore]`d by default so a plain `cargo test` never requires
//! Docker/Postgres to be running. To run them for real:
//!
//!   docker run -d --name pyroboreplay-test-pg -p 5433:5432 \
//!       -e POSTGRES_PASSWORD=test -e POSTGRES_DB=pyroboreplay_test postgres:16
//!   cargo test --test test_postgres_backend_integration -- --ignored --test-threads=1
//!
//! (or use the connection string below directly if you already have a
//! Postgres instance running elsewhere; override via
//! `PYROBOREPLAY_TEST_POSTGRES_URL`).

use pyroboreplay::storage::{PostgresBackend, StorageBackend, StorageError};

fn connection_string() -> String {
    std::env::var("PYROBOREPLAY_TEST_POSTGRES_URL")
        .unwrap_or_else(|_| "postgresql://postgres:test@localhost:5433/pyroboreplay_test".to_string())
}

fn connected_backend() -> PostgresBackend {
    let mut backend = PostgresBackend::new(&connection_string());
    backend
        .connect()
        .expect("failed to connect to local test PostgreSQL (is the container running? see module docs)");
    backend
}

#[test]
#[ignore]
fn test_connect_creates_schema_and_reports_stats() {
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

    // Overwrite (upsert) and confirm the update took effect.
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
fn test_store_and_retrieve_events_and_reports() {
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

    // Deleting the mission must cascade-delete its events and report (FK ON
    // DELETE CASCADE), proving the schema is really wired up correctly.
    backend.delete_mission(&mission_id).expect("delete_mission failed");
    match backend.retrieve_event(&mission_id, "evt-2") {
        Err(StorageError::NotFound(_)) => {}
        other => panic!("expected event to be cascade-deleted, got {:?}", other),
    }
    match backend.retrieve_report(&mission_id) {
        Err(StorageError::NotFound(_)) => {}
        other => panic!("expected report to be cascade-deleted, got {:?}", other),
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
