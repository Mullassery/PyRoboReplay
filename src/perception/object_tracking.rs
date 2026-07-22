//! Object Tracking Layer
//!
//! Associates detections across frames to form object trajectories.
//! Enables per-object analysis and prediction.

use crate::perception::object_detection::{DetectedObject, DetectionFrame, ObjectClass};
use std::collections::HashMap;

/// Single tracked object across time
#[derive(Debug, Clone)]
pub struct TrackedObject {
    /// Unique trajectory ID (persistent across frames)
    pub trajectory_id: u32,

    /// Object class
    pub class: ObjectClass,

    /// All positions (chronological)
    pub positions: Vec<ObjectPosition>,

    /// First detection timestamp
    pub first_seen_sec: f32,

    /// Last detection timestamp
    pub last_seen_sec: f32,

    /// Number of frames this object was visible
    pub visibility_frames: usize,

    /// Average confidence across frames
    pub avg_confidence: f32,

    /// Estimated velocity (m/s)
    pub estimated_velocity: Option<f32>,

    /// Predicted future position
    pub predicted_position: Option<(f32, f32, f32)>,
}

/// Position of object at a single frame
#[derive(Debug, Clone)]
pub struct ObjectPosition {
    /// Timestamp (seconds)
    pub timestamp_sec: f32,

    /// Frame index
    pub frame_index: usize,

    /// 3D position (x, y, z)
    pub position_3d: (f32, f32, f32),

    /// Confidence at this frame
    pub confidence: f32,

    /// Velocity at this frame
    pub velocity: Option<f32>,
}

/// Tracks objects across frames
pub struct TrackingEngine {
    /// Next trajectory ID to assign
    next_trajectory_id: u32,

    /// Active tracks (trajectory_id → TrackedObject)
    active_tracks: HashMap<u32, TrackedObject>,

    /// Completed tracks (for analysis)
    completed_tracks: Vec<TrackedObject>,

    /// Max frames without detection before track dies
    max_tracking_gap_frames: usize,

    /// Max distance (meters) to associate detection to track
    max_association_distance: f32,
}

impl TrackingEngine {
    /// Create new tracking engine
    pub fn new() -> Self {
        TrackingEngine {
            next_trajectory_id: 1,
            active_tracks: HashMap::new(),
            completed_tracks: Vec::new(),
            max_tracking_gap_frames: 10,
            max_association_distance: 5.0,
        }
    }

    /// Process a new detection frame and update tracks
    pub fn process_frame(&mut self, frame: &DetectionFrame) {
        // Associate detections to existing tracks
        let mut associated: HashMap<u32, &DetectedObject> = HashMap::new();

        for detection in &frame.objects {
            if let Some(best_track_id) = self.find_best_track(detection, frame.timestamp_sec) {
                associated.insert(best_track_id, detection);
            }
        }

        // Update associated tracks
        for (track_id, detection) in &associated {
            if let Some(track) = self.active_tracks.get_mut(track_id) {
                track.positions.push(ObjectPosition {
                    timestamp_sec: frame.timestamp_sec,
                    frame_index: frame.frame_index,
                    position_3d: detection.position_3d.unwrap_or((0.0, 0.0, 0.0)),
                    confidence: detection.confidence,
                    velocity: detection.velocity_ms,
                });
                track.last_seen_sec = frame.timestamp_sec;
                track.visibility_frames += 1;
            }
        }

        // Create new tracks for unassociated detections
        for detection in &frame.objects {
            if !associated.values().any(|d| d.id == detection.id) {
                let trajectory_id = self.next_trajectory_id;
                self.next_trajectory_id += 1;

                let track = TrackedObject {
                    trajectory_id,
                    class: detection.class,
                    positions: vec![ObjectPosition {
                        timestamp_sec: frame.timestamp_sec,
                        frame_index: frame.frame_index,
                        position_3d: detection.position_3d.unwrap_or((0.0, 0.0, 0.0)),
                        confidence: detection.confidence,
                        velocity: detection.velocity_ms,
                    }],
                    first_seen_sec: frame.timestamp_sec,
                    last_seen_sec: frame.timestamp_sec,
                    visibility_frames: 1,
                    avg_confidence: detection.confidence,
                    estimated_velocity: detection.velocity_ms,
                    predicted_position: None,
                };

                self.active_tracks.insert(trajectory_id, track);
            }
        }

        // Clean up old tracks
        self.prune_tracks(frame.timestamp_sec);
    }

    /// Find best matching track for a detection
    fn find_best_track(&self, detection: &DetectedObject, current_time: f32) -> Option<u32> {
        let mut best_track_id = None;
        let mut best_distance = self.max_association_distance;

        for (track_id, track) in &self.active_tracks {
            // Only match same class
            if track.class != detection.class {
                continue;
            }

            // Only match if track is recent
            if current_time - track.last_seen_sec > 1.0 {
                continue;
            }

            // Calculate distance
            if let (Some(det_pos), Some(last_pos)) = (detection.position_3d, track.positions.last()) {
                let distance = ((det_pos.0 - last_pos.position_3d.0).powi(2)
                    + (det_pos.1 - last_pos.position_3d.1).powi(2)
                    + (det_pos.2 - last_pos.position_3d.2).powi(2))
                    .sqrt();

                if distance < best_distance {
                    best_distance = distance;
                    best_track_id = Some(*track_id);
                }
            }
        }

        best_track_id
    }

    /// Remove old tracks that haven't been updated
    fn prune_tracks(&mut self, current_time: f32) {
        let mut to_complete = Vec::new();

        for (track_id, track) in &self.active_tracks {
            if current_time - track.last_seen_sec > (self.max_tracking_gap_frames as f32 * 0.033) {
                // Assume 30fps, so ~330ms per frame
                to_complete.push(*track_id);
            }
        }

        for track_id in to_complete {
            if let Some(track) = self.active_tracks.remove(&track_id) {
                self.completed_tracks.push(track);
            }
        }
    }

    /// Get all active tracks
    pub fn get_active_tracks(&self) -> Vec<TrackedObject> {
        self.active_tracks.values().cloned().collect()
    }

    /// Get completed tracks
    pub fn get_completed_tracks(&self) -> Vec<TrackedObject> {
        self.completed_tracks.clone()
    }

    /// Get track by ID
    pub fn get_track(&self, trajectory_id: u32) -> Option<TrackedObject> {
        self.active_tracks.get(&trajectory_id).cloned()
    }
}

impl Default for TrackingEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Trajectory statistics
#[derive(Debug, Clone)]
pub struct TrajectoryStatistics {
    /// Total trajectories created
    pub total_trajectories: usize,

    /// Completed trajectories
    pub completed_trajectories: usize,

    /// Average trajectory length (frames)
    pub avg_trajectory_length: usize,

    /// Trajectories by class
    pub trajectories_by_class: HashMap<ObjectClass, usize>,

    /// Average object persistence (seconds)
    pub avg_persistence_sec: f32,
}

impl TrajectoryStatistics {
    /// Compute from tracking engine
    pub fn from_engine(engine: &TrackingEngine) -> Self {
        let mut trajectories_by_class: HashMap<ObjectClass, usize> = HashMap::new();
        let mut total_length = 0;
        let mut total_persistence = 0.0;

        let all_tracks = [
            engine.get_active_tracks(),
            engine.get_completed_tracks(),
        ]
        .concat();

        for track in &all_tracks {
            *trajectories_by_class.entry(track.class).or_insert(0) += 1;
            total_length += track.visibility_frames;
            total_persistence += track.last_seen_sec - track.first_seen_sec;
        }

        let completed = engine.get_completed_tracks().len();
        let total = all_tracks.len();
        let avg_length = if total > 0 {
            total_length / total
        } else {
            0
        };
        let avg_persistence = if total > 0 {
            total_persistence / total as f32
        } else {
            0.0
        };

        TrajectoryStatistics {
            total_trajectories: total,
            completed_trajectories: completed,
            avg_trajectory_length: avg_length,
            trajectories_by_class,
            avg_persistence_sec: avg_persistence,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_frame(timestamp: f32, index: usize, objects: Vec<DetectedObject>) -> DetectionFrame {
        use crate::perception::object_detection::{BoundingBox, FrameMetadata};

        DetectionFrame {
            timestamp_sec: timestamp,
            frame_index: index,
            camera_id: "front".to_string(),
            objects,
            metadata: FrameMetadata {
                width: 1920,
                height: 1080,
                detector_model: "yolov8".to_string(),
                inference_time_ms: 45.0,
                quality_score: 0.95,
                environmental_factors: HashMap::new(),
            },
        }
    }

    #[test]
    fn test_tracking_engine_creation() {
        let engine = TrackingEngine::new();
        assert_eq!(engine.next_trajectory_id, 1);
        assert!(engine.active_tracks.is_empty());
    }

    #[test]
    fn test_single_object_tracking() {
        let mut engine = TrackingEngine::new();

        use crate::perception::object_detection::BoundingBox;

        let obj = DetectedObject {
            id: 1,
            class: ObjectClass::Person,
            confidence: 0.95,
            bbox: BoundingBox {
                x: 100.0,
                y: 200.0,
                width: 50.0,
                height: 100.0,
            },
            distance_m: Some(2.5),
            velocity_ms: Some(1.0),
            position_3d: Some((2.0, 0.0, 0.0)),
            trajectory_id: None,
            attributes: HashMap::new(),
        };

        let frame1 = create_test_frame(0.0, 0, vec![obj.clone()]);
        engine.process_frame(&frame1);

        assert_eq!(engine.active_tracks.len(), 1);

        let obj2 = DetectedObject {
            position_3d: Some((2.5, 0.0, 0.0)),
            ..obj
        };

        let frame2 = create_test_frame(0.033, 1, vec![obj2]);
        engine.process_frame(&frame2);

        assert_eq!(engine.active_tracks.len(), 1); // Same object associated
    }

    #[test]
    fn test_multiple_object_tracking() {
        let mut engine = TrackingEngine::new();

        use crate::perception::object_detection::BoundingBox;

        let person = DetectedObject {
            id: 1,
            class: ObjectClass::Person,
            confidence: 0.95,
            bbox: BoundingBox {
                x: 100.0,
                y: 200.0,
                width: 50.0,
                height: 100.0,
            },
            distance_m: Some(2.5),
            velocity_ms: Some(1.0),
            position_3d: Some((2.0, 0.0, 0.0)),
            trajectory_id: None,
            attributes: HashMap::new(),
        };

        let vehicle = DetectedObject {
            id: 2,
            class: ObjectClass::Vehicle,
            confidence: 0.87,
            bbox: BoundingBox {
                x: 500.0,
                y: 300.0,
                width: 200.0,
                height: 150.0,
            },
            distance_m: Some(10.0),
            velocity_ms: Some(5.0),
            position_3d: Some((10.0, 2.0, 0.0)),
            trajectory_id: None,
            attributes: HashMap::new(),
        };

        let frame = create_test_frame(0.0, 0, vec![person, vehicle]);
        engine.process_frame(&frame);

        assert_eq!(engine.active_tracks.len(), 2);
    }
}
