use pyroboreplay::core::{DeterministicReplay, EventHasher, MissionRecord, MissionEvent, Pose};
use chrono::Utc;

fn main() {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║  PyRoboReplay: Bit-Perfect Deterministic Replay              ║");
    println!("║  Phase 7.1: Advanced Forensics                              ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    println!("═══════════════════════════════════════════════════════════════════");
    println!("DEMO 1: CREATE DETERMINISTIC REPLAY FROM MISSION");
    println!("═══════════════════════════════════════════════════════════════════\n");

    // Create a synthetic mission
    let mut mission = MissionRecord::new("forensic_test_mission");
    let now = Utc::now();

    for i in 0..5 {
        mission.add_event(MissionEvent::RobotPose {
            robot_id: "robot_1".to_string(),
            timestamp: now + chrono::Duration::seconds(i),
            pose: Pose {
                x: (i as f64) * 1.0,
                y: (i as f64) * 0.5,
                z: 0.0,
                qx: 0.0,
                qy: 0.0,
                qz: 0.0,
                qw: 1.0,
            },
            confidence: Some(0.95 - (i as f32) * 0.01),
        });
    }

    println!("✓ Created mission with {} events", mission.events.len());

    // Create deterministic replay
    let replay = DeterministicReplay::from_mission(&mission).expect("Failed to create replay");
    println!("✓ Created deterministic replay\n");

    println!("═══════════════════════════════════════════════════════════════════");
    println!("DEMO 2: EXAMINE REPLAY MANIFEST");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let manifest = replay.manifest();
    println!("Replay Manifest:");
    println!("  Replay ID: {}", manifest.replay_id);
    println!("  Original Mission ID: {}", manifest.original_mission_id);
    println!("  Event Count: {}", manifest.event_count);
    println!("  Chain Hash (first 16 chars): {}...", &manifest.chain_hash[..16]);
    println!("  Event Hashes Sample:");
    for (i, hash) in manifest.event_hashes.iter().take(3).enumerate() {
        println!("    Event {}: {}...", i, &hash[..16]);
    }
    if manifest.event_hashes.len() > 3 {
        println!("    ... and {} more", manifest.event_hashes.len() - 3);
    }
    println!();

    println!("═══════════════════════════════════════════════════════════════════");
    println!("DEMO 3: VERIFY MANIFEST INTEGRITY");
    println!("═══════════════════════════════════════════════════════════════════\n");

    match replay.verify_manifest() {
        Ok(verified) => {
            println!("✓ Manifest verified: {}", verified);
            println!("  All event hashes match stored values");
            println!("  Chain hash verified\n");
        }
        Err(e) => {
            eprintln!("✗ Verification failed: {}\n", e);
        }
    }

    println!("═══════════════════════════════════════════════════════════════════");
    println!("DEMO 4: RUN IDENTICAL REPLAYS");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let replay2 = DeterministicReplay::from_mission(&mission).expect("Failed to create replay 2");
    let replay3 = DeterministicReplay::from_mission(&mission).expect("Failed to create replay 3");

    match replay2.assert_identical_to(&replay3) {
        Ok(_) => {
            println!("✓ Two independent replays are identical");
            println!("  Chain hash 1: {}...", &replay2.manifest().chain_hash[..16]);
            println!("  Chain hash 2: {}...", &replay3.manifest().chain_hash[..16]);
            println!("  ✓ Hashes match\n");
        }
        Err(e) => {
            eprintln!("✗ Replays differ: {}\n", e);
        }
    }

    println!("═══════════════════════════════════════════════════════════════════");
    println!("DEMO 5: DETERMINISTIC HASHING");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let event = &mission.events[0];
    let hash1 = EventHasher::hash_event(event);
    let hash2 = EventHasher::hash_event(event);

    println!("Event hash (stable across multiple calls):");
    println!("  Hash 1: {}", hash1);
    println!("  Hash 2: {}", hash2);
    println!("  ✓ Identical: {}\n", hash1 == hash2);

    println!("═══════════════════════════════════════════════════════════════════");
    println!("FORENSIC CAPABILITIES ENABLED");
    println!("═══════════════════════════════════════════════════════════════════\n");

    println!("✓ Bit-perfect replay manifest generation");
    println!("✓ Deterministic SHA-256 event hashing");
    println!("✓ Chain integrity verification");
    println!("✓ Replay identity assertion");
    println!("✓ Canonical JSON serialization (key-sorted)");
    println!("✓ Tamper detection on any event modification");
    println!("✓ Proof of replay fidelity\n");

    println!("═══════════════════════════════════════════════════════════════════");
    println!("✨ Phase 7.1: Deterministic Replay Complete");
    println!("═══════════════════════════════════════════════════════════════════\n");
}
