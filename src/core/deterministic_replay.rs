use crate::core::event::MissionEvent;
use crate::core::root_cause::RootCauseAnalyzer;
use crate::core::causality::CausalGraphBuilder;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

/// SHA-256 hasher for mission events
pub struct EventHasher;

impl EventHasher {
    /// Hash a single event deterministically
    /// Uses canonical JSON representation (sorted keys) for byte stability
    pub fn hash_event(event: &MissionEvent) -> String {
        let canonical_bytes = Self::canonical_json(event);
        let digest = Sha256::digest(&canonical_bytes);
        format!("{:02x?}", digest).replace(", ", "").replace("[", "").replace("]", "")
    }

    /// Produce canonical (byte-stable) JSON for an event
    /// Serializes via serde_json, then recursively sorts all object keys
    pub fn canonical_json(event: &MissionEvent) -> Vec<u8> {
        let value = serde_json::to_value(event).unwrap_or(serde_json::Value::Null);
        let sorted = Self::sort_json_keys(&value);
        serde_json::to_vec(&sorted).unwrap_or_default()
    }

    /// Recursively sort all object keys in a JSON value
    fn sort_json_keys(value: &serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Object(map) => {
                let mut sorted: Vec<_> = map.iter().collect();
                sorted.sort_by_key(|(k, _)| k.as_str());
                let mut sorted_map = serde_json::Map::new();
                for (k, v) in sorted {
                    sorted_map.insert(k.clone(), Self::sort_json_keys(v));
                }
                serde_json::Value::Object(sorted_map)
            }
            serde_json::Value::Array(arr) => {
                serde_json::Value::Array(arr.iter().map(|v| Self::sort_json_keys(v)).collect())
            }
            other => other.clone(),
        }
    }

    /// Hash a chain of event hashes to produce a single chain hash
    /// Concatenates all hex-encoded hashes and produces a SHA-256 of the concatenation
    pub fn chain_hash(event_hashes: &[String]) -> String {
        let concatenated = event_hashes.join("");
        let digest = Sha256::digest(concatenated.as_bytes());
        format!("{:02x?}", digest).replace(", ", "").replace("[", "").replace("]", "")
    }
}

/// Manifest of a deterministic replay
/// Proves that a replay is identical to the original by storing hashes of all events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayManifest {
    pub replay_id: Uuid,
    pub original_mission_id: String,
    pub created_at: DateTime<Utc>,
    pub event_hashes: Vec<String>,
    pub chain_hash: String,
    pub event_count: usize,
}

/// Errors that can occur during deterministic replay
#[derive(Debug, Error)]
pub enum DeterministicReplayError {
    #[error("Hash mismatch at event index {index}: expected {expected}, got {actual}")]
    HashMismatch {
        index: usize,
        expected: String,
        actual: String,
    },
    #[error("Chain hash mismatch: expected {expected}, got {actual}")]
    ChainHashMismatch { expected: String, actual: String },
    #[error("Event count mismatch: expected {expected}, got {actual}")]
    EventCountMismatch { expected: usize, actual: usize },
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
}

/// A deterministically reproducible replay of a mission
/// Ensures that running the same analysis twice on the same mission produces identical results
pub struct DeterministicReplay {
    manifest: ReplayManifest,
    events: Vec<MissionEvent>,
    mission_id: String,
}

impl DeterministicReplay {
    /// Create a deterministic replay from a mission record
    pub fn from_mission(mission: &crate::core::event::MissionRecord) -> Result<Self, DeterministicReplayError> {
        let events = mission.events.clone();
        let event_hashes: Vec<String> = events.iter().map(EventHasher::hash_event).collect();
        let chain_hash = EventHasher::chain_hash(&event_hashes);

        let manifest = ReplayManifest {
            replay_id: Uuid::new_v4(),
            original_mission_id: mission.id.to_string(),
            created_at: Utc::now(),
            event_hashes,
            chain_hash,
            event_count: events.len(),
        };

        Ok(DeterministicReplay {
            mission_id: mission.id.to_string(),
            manifest,
            events,
        })
    }

    /// Verify that the manifest matches the replayed events
    /// Re-hashes all events and compares to stored hashes
    pub fn verify_manifest(&self) -> Result<bool, DeterministicReplayError> {
        if self.events.len() != self.manifest.event_count {
            return Err(DeterministicReplayError::EventCountMismatch {
                expected: self.manifest.event_count,
                actual: self.events.len(),
            });
        }

        for (i, event) in self.events.iter().enumerate() {
            let computed_hash = EventHasher::hash_event(event);
            let stored_hash = &self.manifest.event_hashes[i];

            if computed_hash != *stored_hash {
                return Err(DeterministicReplayError::HashMismatch {
                    index: i,
                    expected: stored_hash.clone(),
                    actual: computed_hash,
                });
            }
        }

        let computed_chain = EventHasher::chain_hash(&self.manifest.event_hashes);
        if computed_chain != self.manifest.chain_hash {
            return Err(DeterministicReplayError::ChainHashMismatch {
                expected: self.manifest.chain_hash.clone(),
                actual: computed_chain,
            });
        }

        Ok(true)
    }

    /// Assert that two replays are identical
    /// Compares chain hashes (fast proof of identity)
    pub fn assert_identical_to(&self, other: &DeterministicReplay) -> Result<(), DeterministicReplayError> {
        if self.manifest.chain_hash != other.manifest.chain_hash {
            return Err(DeterministicReplayError::ChainHashMismatch {
                expected: self.manifest.chain_hash.clone(),
                actual: other.manifest.chain_hash.clone(),
            });
        }
        Ok(())
    }

    /// Run causal analysis on this deterministic replay
    /// Results are guaranteed to be identical if run multiple times
    pub fn run_causal_analysis(&self) -> Option<crate::core::root_cause::RootCauseAnalysis> {
        let mut graph_builder = CausalGraphBuilder::new(self.events.clone());
        let graph = graph_builder.build();

        let mut analyzer = RootCauseAnalyzer::new(self.events.clone());
        analyzer = analyzer.with_causal_graph(graph);

        if let Some(first_failure_event) = self
            .events
            .iter()
            .position(|e| matches!(e, MissionEvent::NavigationDecision { .. }))
        {
            analyzer.analyze_failure(first_failure_event)
        } else {
            None
        }
    }

    /// Get the replay manifest
    pub fn manifest(&self) -> &ReplayManifest {
        &self.manifest
    }

    /// Get the replayed events
    pub fn events(&self) -> &[MissionEvent] {
        &self.events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event::{MissionRecord, Pose};

    fn create_test_mission() -> MissionRecord {
        let mut mission = MissionRecord::new("test_deterministic");
        let now = Utc::now();

        mission.add_event(MissionEvent::RobotPose {
            robot_id: "robot_1".to_string(),
            timestamp: now,
            pose: Pose {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                qx: 0.0,
                qy: 0.0,
                qz: 0.0,
                qw: 1.0,
            },
            confidence: Some(0.95),
        });

        mission.add_event(MissionEvent::RobotPose {
            robot_id: "robot_1".to_string(),
            timestamp: now + chrono::Duration::seconds(1),
            pose: Pose {
                x: 1.0,
                y: 1.0,
                z: 0.0,
                qx: 0.0,
                qy: 0.0,
                qz: 0.0,
                qw: 1.0,
            },
            confidence: Some(0.90),
        });

        mission
    }

    #[test]
    fn test_hash_event_deterministic() {
        let event = MissionEvent::RobotPose {
            robot_id: "robot_1".to_string(),
            timestamp: Utc::now(),
            pose: Pose {
                x: 1.0,
                y: 2.0,
                z: 3.0,
                qx: 0.0,
                qy: 0.0,
                qz: 0.0,
                qw: 1.0,
            },
            confidence: Some(0.95),
        };

        let hash1 = EventHasher::hash_event(&event);
        let hash2 = EventHasher::hash_event(&event);

        assert_eq!(hash1, hash2);
        assert!(!hash1.is_empty());
        assert!(hash1.len() == 64);
    }

    #[test]
    fn test_hash_event_differs_for_different_events() {
        let event1 = MissionEvent::RobotPose {
            robot_id: "robot_1".to_string(),
            timestamp: Utc::now(),
            pose: Pose {
                x: 1.0,
                y: 2.0,
                z: 3.0,
                qx: 0.0,
                qy: 0.0,
                qz: 0.0,
                qw: 1.0,
            },
            confidence: Some(0.95),
        };

        let event2 = MissionEvent::RobotPose {
            robot_id: "robot_2".to_string(),
            timestamp: Utc::now(),
            pose: Pose {
                x: 2.0,
                y: 3.0,
                z: 4.0,
                qx: 0.0,
                qy: 0.0,
                qz: 0.0,
                qw: 1.0,
            },
            confidence: Some(0.90),
        };

        let hash1 = EventHasher::hash_event(&event1);
        let hash2 = EventHasher::hash_event(&event2);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_canonical_json_key_ordering() {
        let event = MissionEvent::RobotPose {
            robot_id: "robot_1".to_string(),
            timestamp: Utc::now(),
            pose: Pose {
                x: 1.0,
                y: 2.0,
                z: 3.0,
                qx: 0.0,
                qy: 0.0,
                qz: 0.0,
                qw: 1.0,
            },
            confidence: Some(0.95),
        };

        let canonical = EventHasher::canonical_json(&event);
        let canonical_str = String::from_utf8_lossy(&canonical);

        let value: serde_json::Value = serde_json::from_str(&canonical_str).unwrap();
        if let serde_json::Value::Object(obj) = value {
            let keys: Vec<&String> = obj.keys().collect();
            let sorted_keys = {
                let mut k = keys.clone();
                k.sort();
                k
            };
            assert_eq!(keys, sorted_keys);
        }
    }

    #[test]
    fn test_chain_hash_changes_if_one_event_changes() {
        let hashes1 = vec![
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210".to_string(),
        ];
        let hashes2 = vec![
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543211".to_string(),
        ];

        let chain1 = EventHasher::chain_hash(&hashes1);
        let chain2 = EventHasher::chain_hash(&hashes2);

        assert_ne!(chain1, chain2);
    }

    #[test]
    fn test_replay_manifest_created_from_mission() {
        let mission = create_test_mission();
        let replay = DeterministicReplay::from_mission(&mission).unwrap();

        assert_eq!(replay.manifest.original_mission_id, mission.id.to_string());
        assert_eq!(replay.manifest.event_count, 2);
        assert_eq!(replay.manifest.event_hashes.len(), 2);
        assert!(!replay.manifest.chain_hash.is_empty());
    }

    #[test]
    fn test_replay_manifest_event_count_matches() {
        let mission = create_test_mission();
        let replay = DeterministicReplay::from_mission(&mission).unwrap();

        assert_eq!(replay.manifest.event_hashes.len(), replay.manifest.event_count);
        assert_eq!(replay.events.len(), replay.manifest.event_count);
    }

    #[test]
    fn test_verify_manifest_passes_unmodified() {
        let mission = create_test_mission();
        let replay = DeterministicReplay::from_mission(&mission).unwrap();

        let verified = replay.verify_manifest().unwrap();
        assert!(verified);
    }

    #[test]
    fn test_verify_manifest_fails_if_event_mutated() {
        let mission = create_test_mission();
        let mut replay = DeterministicReplay::from_mission(&mission).unwrap();

        replay.events[0] = MissionEvent::RobotPose {
            robot_id: "robot_2".to_string(),
            timestamp: Utc::now(),
            pose: Pose {
                x: 5.0,
                y: 6.0,
                z: 7.0,
                qx: 0.0,
                qy: 0.0,
                qz: 0.0,
                qw: 1.0,
            },
            confidence: Some(0.50),
        };

        let result = replay.verify_manifest();
        assert!(matches!(result, Err(DeterministicReplayError::HashMismatch { .. })));
    }

    #[test]
    fn test_two_replays_of_same_mission_are_identical() {
        let mission = create_test_mission();
        let replay1 = DeterministicReplay::from_mission(&mission).unwrap();
        let replay2 = DeterministicReplay::from_mission(&mission).unwrap();

        assert!(replay1.assert_identical_to(&replay2).is_ok());
    }

    #[test]
    fn test_replay_chain_hash_deterministic() {
        let mission = create_test_mission();
        let replay1 = DeterministicReplay::from_mission(&mission).unwrap();
        let replay2 = DeterministicReplay::from_mission(&mission).unwrap();

        assert_eq!(replay1.manifest.chain_hash, replay2.manifest.chain_hash);
    }

    #[test]
    fn test_replay_event_count_correct() {
        let mission = create_test_mission();
        let replay = DeterministicReplay::from_mission(&mission).unwrap();

        assert_eq!(replay.events().len(), 2);
        assert_eq!(replay.manifest.event_count, 2);
    }

    #[test]
    fn test_canonical_json_stable_across_calls() {
        let event = MissionEvent::RobotPose {
            robot_id: "robot_1".to_string(),
            timestamp: Utc::now(),
            pose: Pose {
                x: 1.0,
                y: 2.0,
                z: 3.0,
                qx: 0.0,
                qy: 0.0,
                qz: 0.0,
                qw: 1.0,
            },
            confidence: Some(0.95),
        };

        let bytes1 = EventHasher::canonical_json(&event);
        let bytes2 = EventHasher::canonical_json(&event);

        assert_eq!(bytes1, bytes2);
    }
}
