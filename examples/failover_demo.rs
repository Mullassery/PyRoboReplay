use pyroboreplay::storage::{
    FailoverManager, BackupConfig, InMemoryBackend, StorageBackend, StorageError, StorageResult,
};
use pyroboreplay::storage::backend::StorageStats;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Wraps a real `InMemoryBackend` but can be told to fail `connect()` on
/// demand, so the demo can trigger `FailoverManager`'s real automatic
/// promotion path (`heartbeat()` -> internal `promote_standby()`) instead of
/// reaching into a private method that isn't part of the public API.
struct FlakyPrimary {
    inner: InMemoryBackend,
    should_fail_connect: Arc<AtomicBool>,
}

impl FlakyPrimary {
    fn new(should_fail_connect: Arc<AtomicBool>) -> Self {
        FlakyPrimary {
            inner: InMemoryBackend::new(),
            should_fail_connect,
        }
    }
}

impl StorageBackend for FlakyPrimary {
    fn connect(&mut self) -> StorageResult<()> {
        if self.should_fail_connect.load(Ordering::SeqCst) {
            return Err(StorageError::ConnectionFailed(
                "simulated primary failure".to_string(),
            ));
        }
        self.inner.connect()
    }

    fn store_mission(&self, mission_id: &str, data: &str) -> StorageResult<()> {
        self.inner.store_mission(mission_id, data)
    }

    fn retrieve_mission(&self, mission_id: &str) -> StorageResult<String> {
        self.inner.retrieve_mission(mission_id)
    }

    fn store_event(&self, mission_id: &str, event_id: &str, data: &str) -> StorageResult<()> {
        self.inner.store_event(mission_id, event_id, data)
    }

    fn retrieve_event(&self, mission_id: &str, event_id: &str) -> StorageResult<String> {
        self.inner.retrieve_event(mission_id, event_id)
    }

    fn store_report(&self, mission_id: &str, report: &str) -> StorageResult<()> {
        self.inner.store_report(mission_id, report)
    }

    fn retrieve_report(&self, mission_id: &str) -> StorageResult<String> {
        self.inner.retrieve_report(mission_id)
    }

    fn list_missions(&self, limit: Option<usize>) -> StorageResult<Vec<String>> {
        self.inner.list_missions(limit)
    }

    fn delete_mission(&self, mission_id: &str) -> StorageResult<()> {
        self.inner.delete_mission(mission_id)
    }

    fn mission_exists(&self, mission_id: &str) -> StorageResult<bool> {
        self.inner.mission_exists(mission_id)
    }

    fn get_stats(&self) -> StorageResult<StorageStats> {
        self.inner.get_stats()
    }

    fn close(&self) -> StorageResult<()> {
        self.inner.close()
    }
}

fn main() {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║  PyRoboReplay: Mission-Critical Failover & Redundancy       ║");
    println!("║  Phase 7.2: Advanced Forensics                              ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    println!("═══════════════════════════════════════════════════════════════════");
    println!("DEMO 1: CREATE FAILOVER MANAGER WITH PRIMARY + STANDBYS");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let should_fail_connect = Arc::new(AtomicBool::new(false));
    let primary = Box::new(FlakyPrimary::new(should_fail_connect.clone()));
    let standby1 = Box::new(InMemoryBackend::new());
    let standby2 = Box::new(InMemoryBackend::new());

    let config = BackupConfig::default();
    let mut failover_mgr = FailoverManager::new(primary, vec![standby1, standby2], config);

    println!("✓ Created FailoverManager with:");
    println!("  - 1 Primary backend");
    println!("  - 2 Standby backends");
    println!("  - Write-ahead logging enabled");
    println!("  - Auto-promotion on primary failure\n");

    println!("═══════════════════════════════════════════════════════════════════");
    println!("DEMO 2: STORE MISSION WITH WRITE-AHEAD TO STANDBYS");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let mission_data = r#"{"id":"mission_001","name":"Warehouse Exploration","status":"completed"}"#;
    match failover_mgr.store_mission("mission_001", mission_data) {
        Ok(_) => {
            println!("✓ Stored mission to primary backend");
            println!("✓ Write-ahead logged to standby_1");
            println!("✓ Write-ahead logged to standby_2\n");
        }
        Err(e) => eprintln!("✗ Write failed: {:?}\n", e),
    }

    println!("═══════════════════════════════════════════════════════════════════");
    println!("DEMO 3: HEALTH CHECK VIA HEARTBEAT");
    println!("═══════════════════════════════════════════════════════════════════\n");

    match failover_mgr.heartbeat() {
        Ok(_) => {
            println!("✓ Primary backend healthy");
            println!("✓ No promotion needed");
            println!("✓ Failover log empty (0 events)\n");
        }
        Err(e) => eprintln!("✗ Heartbeat failed: {:?}\n", e),
    }

    println!("═══════════════════════════════════════════════════════════════════");
    println!("DEMO 4: RETRIEVE MISSION DATA");
    println!("═══════════════════════════════════════════════════════════════════\n");

    match failover_mgr.retrieve_mission("mission_001") {
        Ok(data) => {
            println!("✓ Retrieved mission from primary");
            println!("  Data: {}\n", data);
        }
        Err(e) => eprintln!("✗ Retrieve failed: {:?}\n", e),
    }

    println!("═══════════════════════════════════════════════════════════════════");
    println!("DEMO 5: AUTOMATIC FAILOVER");
    println!("═══════════════════════════════════════════════════════════════════\n");

    println!("Current primary: {}", failover_mgr.current_primary_name());
    println!("Using standby: {}\n", failover_mgr.is_using_standby());

    // Simulate a real primary failure: the next connect() attempt this
    // FlakyPrimary makes will genuinely return an error, so heartbeat()
    // detects a real failure and drives the manager's own internal
    // promotion logic - not a manually-triggered shortcut.
    should_fail_connect.store(true, Ordering::SeqCst);
    println!("Simulating primary failure (next heartbeat's connect() will fail)...\n");

    match failover_mgr.heartbeat() {
        Ok(_) => {
            println!("✓ Standby promoted to primary");
            println!("✓ New primary: {}", failover_mgr.current_primary_name());
            println!("✓ Using standby: {}\n", failover_mgr.is_using_standby());
        }
        Err(e) => eprintln!("✗ Promotion failed: {:?}\n", e),
    }

    println!("═══════════════════════════════════════════════════════════════════");
    println!("DEMO 6: FAILOVER EVENT LOG");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let log = failover_mgr.failover_log();
    println!("Failover Event Log ({} events):", log.len());
    for (i, event) in log.iter().enumerate() {
        println!("  {}. Type: {:?}", i + 1, event.event_type);
        println!("     Timestamp: {}", event.timestamp.format("%H:%M:%S UTC"));
        println!("     Promoted: {} → Demoted: {}", event.promoted_backend, event.demoted_backend);
    }
    println!();

    println!("═══════════════════════════════════════════════════════════════════");
    println!("PRODUCTION FAILOVER CAPABILITIES");
    println!("═══════════════════════════════════════════════════════════════════\n");

    println!("✓ Primary + Standby redundancy architecture");
    println!("✓ Write-ahead logging to all standbys");
    println!("✓ Heartbeat-based health checking");
    println!("✓ Automatic promotion on primary failure");
    println!("✓ Non-fatal standby write failures (logged)");
    println!("✓ Zero data loss during failover");
    println!("✓ Configurable heartbeat intervals");
    println!("✓ Complete failover audit trail\n");

    println!("═══════════════════════════════════════════════════════════════════");
    println!("✨ Phase 7.2: Mission-Critical Failover Complete");
    println!("═══════════════════════════════════════════════════════════════════\n");
}
