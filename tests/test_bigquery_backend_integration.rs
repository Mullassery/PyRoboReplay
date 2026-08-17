//! Real integration tests for `BigQueryBackend` against a live BigQuery
//! REST-compatible emulator (`ghcr.io/goccy/bigquery-emulator`).
//!
//! These are `#[ignore]`d by default so a plain `cargo test` never requires
//! Docker/the emulator to be running. To run them for real:
//!
//!   docker run -d --name pyroboreplay-test-bq -p 9050:9050 -p 9060:9060 \
//!       ghcr.io/goccy/bigquery-emulator:latest --project=test-project
//!   cargo test --test test_bigquery_backend_integration -- --ignored --test-threads=1
//!
//! (or point at a different emulator/project via
//! `PYROBOREPLAY_TEST_BIGQUERY_URL`; each test run uses a fresh dataset name
//! so tests don't collide with each other or with prior runs).

use pyroboreplay::storage::{BigQueryBackend, StorageBackend, StorageError};

fn connection_string() -> String {
    std::env::var("PYROBOREPLAY_TEST_BIGQUERY_URL").unwrap_or_else(|_| {
        let dataset = format!("pyroboreplay_test_{}", std::process::id());
        format!("bigquery://test-project/{}?endpoint=http://localhost:9050", dataset)
    })
}

fn connected_backend() -> BigQueryBackend {
    let mut backend = BigQueryBackend::new(&connection_string());
    backend
        .connect()
        .expect("failed to connect to local test BigQuery emulator (is the container running? see module docs)");
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

    // Overwrite (delete-then-insert upsert) and confirm the update took effect.
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

    // delete_mission only issues DELETEs against events/reports/missions
    // directly (BigQuery has no FK cascade), so this proves that path is
    // genuinely wired up rather than relying on a database-level cascade.
    backend.delete_mission(&mission_id).expect("delete_mission failed");
    match backend.retrieve_event(&mission_id, "evt-2") {
        Err(StorageError::NotFound(_)) => {}
        other => panic!("expected event to be deleted, got {:?}", other),
    }
    match backend.retrieve_report(&mission_id) {
        Err(StorageError::NotFound(_)) => {}
        other => panic!("expected report to be deleted, got {:?}", other),
    }
}

#[test]
#[ignore]
fn test_list_missions_respects_limit() {
    let backend = connected_backend();
    let prefix = format!("list-test-{}", uuid::Uuid::new_v4());
    for i in 0..3 {
        backend
            .store_mission(&format!("{}-{}", prefix, i), &format!(r#"{{"i":{}}}"#, i))
            .expect("store_mission failed");
    }

    let all = backend.list_missions(None).expect("list_missions failed");
    let matching: Vec<_> = all.iter().filter(|id| id.starts_with(&prefix)).collect();
    assert_eq!(matching.len(), 3);

    let limited = backend.list_missions(Some(1)).expect("list_missions with limit failed");
    assert_eq!(limited.len(), 1);
}

#[test]
#[ignore]
fn test_delete_missing_mission_returns_not_found() {
    let backend = connected_backend();
    let mission_id = format!("does-not-exist-{}", uuid::Uuid::new_v4());
    match backend.delete_mission(&mission_id) {
        Err(StorageError::NotFound(_)) => {}
        other => panic!("expected NotFound, got {:?}", other),
    }
}

#[test]
#[ignore]
fn test_get_stats_counts_reflect_stores() {
    let backend = connected_backend();
    let before = backend.get_stats().expect("get_stats failed");

    let mission_id = format!("stats-test-{}", uuid::Uuid::new_v4());
    backend.store_mission(&mission_id, r#"{"a":1}"#).expect("store_mission failed");
    backend.store_event(&mission_id, "evt-0", r#"{"b":2}"#).expect("store_event failed");
    backend.store_report(&mission_id, r#"{"c":3}"#).expect("store_report failed");

    let after = backend.get_stats().expect("get_stats failed");
    assert_eq!(after.total_missions, before.total_missions + 1);
    assert_eq!(after.total_events, before.total_events + 1);
    assert_eq!(after.total_reports, before.total_reports + 1);
}
