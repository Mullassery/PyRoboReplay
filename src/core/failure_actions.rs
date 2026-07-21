use crate::core::anomaly_detector::Failure;

/// A recommended action to mitigate or prevent a failure
#[derive(Debug, Clone)]
pub struct Action {
    /// Priority level (P0, P1, P2)
    pub priority: String,
    /// Short description of the action
    pub description: String,
    /// Expected impact (high, medium, low)
    pub impact: String,
    /// Implementation complexity (easy, medium, hard)
    pub complexity: String,
    /// Detailed implementation steps
    pub implementation: String,
}

/// Generates recommended actions for failures
pub struct ActionRecommender;

impl ActionRecommender {
    /// Generate recommended actions to mitigate a failure
    pub fn recommend(failure: &Failure) -> Vec<Action> {
        match failure.failure_type.as_str() {
            "near_collision" => Self::actions_near_collision(),
            "perception_failure" => Self::actions_perception_failure(),
            "sensor_dropout" => Self::actions_sensor_dropout(),
            "communication_loss" => Self::actions_communication_loss(),
            "navigation_deadlock" => Self::actions_navigation_deadlock(),
            "localization_loss" => Self::actions_localization_loss(),
            "oscillation" => Self::actions_oscillation(),
            "costmap_anomaly" => Self::actions_costmap_anomaly(),
            _ => vec![],
        }
    }

    fn actions_near_collision() -> Vec<Action> {
        vec![
            Action {
                priority: "P0".to_string(),
                description: "Reduce obstacle detection threshold in LiDAR config".to_string(),
                impact: "high".to_string(),
                complexity: "easy".to_string(),
                implementation: "Decrease 'lidar_min_range' from 0.5m to 0.3m in config. \
                                 This will detect obstacles earlier and give planner more time to react. \
                                 Test in simulator first with known obstacles."
                    .to_string(),
            },
            Action {
                priority: "P1".to_string(),
                description: "Improve collision avoidance margins in planner".to_string(),
                impact: "high".to_string(),
                complexity: "medium".to_string(),
                implementation: "Increase 'safety_margin' parameter in DWA/TEB planner. \
                                 Also reduce max_velocity to give more time for obstacle avoidance. \
                                 Test with dynamic obstacle scenarios."
                    .to_string(),
            },
            Action {
                priority: "P2".to_string(),
                description: "Add sensor fusion with camera-based obstacle detection".to_string(),
                impact: "medium".to_string(),
                complexity: "hard".to_string(),
                implementation: "Integrate camera detections into costmap building. \
                                 Use camera to validate LiDAR detections before triggering collision avoidance. \
                                 Requires detector model inference and fusion logic."
                    .to_string(),
            },
        ]
    }

    fn actions_perception_failure() -> Vec<Action> {
        vec![
            Action {
                priority: "P0".to_string(),
                description: "Increase detection confidence threshold".to_string(),
                impact: "medium".to_string(),
                complexity: "easy".to_string(),
                implementation: "Raise 'min_detection_confidence' from 0.5 to 0.7 in perception config. \
                                 This will reduce false positives but may miss some objects. \
                                 Monitor detection rates in next few missions."
                    .to_string(),
            },
            Action {
                priority: "P1".to_string(),
                description: "Improve camera image quality or lighting".to_string(),
                impact: "high".to_string(),
                complexity: "medium".to_string(),
                implementation: "Check camera lens for dirt/fog. Ensure adequate lighting. \
                                 If outdoors, may need higher ISO or faster shutter. \
                                 May require hardware adjustment - check mount alignment."
                    .to_string(),
            },
            Action {
                priority: "P2".to_string(),
                description: "Upgrade to higher-quality object detection model".to_string(),
                impact: "high".to_string(),
                complexity: "hard".to_string(),
                implementation: "Evaluate YOLO v8, Faster R-CNN, or Vision Transformer models. \
                                 Requires retraining or fine-tuning on your specific objects. \
                                 Plan 2-3 weeks for model evaluation and deployment."
                    .to_string(),
            },
        ]
    }

    fn actions_sensor_dropout() -> Vec<Action> {
        vec![
            Action {
                priority: "P0".to_string(),
                description: "Investigate sensor driver health and restart if needed".to_string(),
                impact: "high".to_string(),
                complexity: "easy".to_string(),
                implementation: "Check sensor driver logs for errors. \
                                 Restart sensor driver: 'systemctl restart ros_sensors'. \
                                 Monitor for recurrence in next 10 missions. \
                                 If dropout persists, advance to P1."
                    .to_string(),
            },
            Action {
                priority: "P1".to_string(),
                description: "Check network connectivity and bandwidth".to_string(),
                impact: "high".to_string(),
                complexity: "medium".to_string(),
                implementation: "Run 'iperf' to measure network bandwidth. \
                                 Check for network congestion using 'nethogs'. \
                                 If WiFi: try different channel (2.4GHz vs 5GHz). \
                                 If Ethernet: check cable integrity and switch port."
                    .to_string(),
            },
            Action {
                priority: "P2".to_string(),
                description: "Replace or upgrade sensor hardware".to_string(),
                impact: "high".to_string(),
                complexity: "hard".to_string(),
                implementation: "If dropout is intermittent, may indicate failing hardware. \
                                 Test sensor in isolation with direct USB connection. \
                                 If persistent, order replacement and schedule swap. \
                                 Budget: 1-2 weeks lead time + 2 hours installation."
                    .to_string(),
            },
        ]
    }

    fn actions_communication_loss() -> Vec<Action> {
        vec![
            Action {
                priority: "P0".to_string(),
                description: "Increase topic publish rate to reduce message gaps".to_string(),
                impact: "low".to_string(),
                complexity: "easy".to_string(),
                implementation: "Check sensor publish rates: 'rostopic hz /lidar /camera /odom'. \
                                 Increase rates by 20-50% in launch files. \
                                 May increase CPU load - monitor with 'top'."
                    .to_string(),
            },
            Action {
                priority: "P1".to_string(),
                description: "Optimize ROS middleware and network settings".to_string(),
                impact: "medium".to_string(),
                complexity: "medium".to_string(),
                implementation: "Switch to ROS 2 DDS with optimized QoS settings. \
                                 Reduce message size: enable compression, skip non-critical fields. \
                                 Check ROS_DOMAIN_ID conflicts with other robots."
                    .to_string(),
            },
            Action {
                priority: "P2".to_string(),
                description: "Upgrade network infrastructure".to_string(),
                impact: "high".to_string(),
                complexity: "hard".to_string(),
                implementation: "Switch to dedicated 5GHz WiFi channel for robot fleet. \
                                 Or move to Ethernet via slip rings or onboard processing. \
                                 Budget: 3-4 weeks for infrastructure upgrade and testing."
                    .to_string(),
            },
        ]
    }

    fn actions_navigation_deadlock() -> Vec<Action> {
        vec![
            Action {
                priority: "P0".to_string(),
                description: "Increase local planner goal tolerance".to_string(),
                impact: "medium".to_string(),
                complexity: "easy".to_string(),
                implementation: "Increase 'xy_goal_tolerance' and 'yaw_goal_tolerance' parameters. \
                                 Allows robot to consider goal reached even if not perfectly aligned. \
                                 Test with missions that previously got stuck."
                    .to_string(),
            },
            Action {
                priority: "P1".to_string(),
                description: "Enable dynamic obstacle avoidance with recovery behaviors".to_string(),
                impact: "high".to_string(),
                complexity: "medium".to_string(),
                implementation: "Enable rotate_recovery, clear_costmap_recovery behaviors. \
                                 These give planner multiple strategies to escape deadlock. \
                                 Configure recovery attempt limits to avoid infinite loops."
                    .to_string(),
            },
            Action {
                priority: "P2".to_string(),
                description: "Upgrade to model-predictive control planner".to_string(),
                impact: "high".to_string(),
                complexity: "hard".to_string(),
                implementation: "Consider MPC-based planners (e.g., ROS Navigation2 MPPI Controller). \
                                 MPC can predict and avoid deadlock situations. \
                                 Requires solver library integration and tuning (2-3 weeks)."
                    .to_string(),
            },
        ]
    }

    fn actions_localization_loss() -> Vec<Action> {
        vec![
            Action {
                priority: "P0".to_string(),
                description: "Check for wheel slippage or sensor calibration drift".to_string(),
                impact: "high".to_string(),
                complexity: "easy".to_string(),
                implementation: "Run odometry calibration routine: \
                                 Move robot known distances and verify output. \
                                 Check wheel pressure (pneumatic wheels). \
                                 Ensure IMU is mounted securely and not vibrating."
                    .to_string(),
            },
            Action {
                priority: "P1".to_string(),
                description: "Re-initialize or improve SLAM system".to_string(),
                impact: "high".to_string(),
                complexity: "medium".to_string(),
                implementation: "Rebuild SLAM map in area of failure. \
                                 Increase feature detection parameters. \
                                 Enable loop closure if available. \
                                 Consider relocalization from existing map."
                    .to_string(),
            },
            Action {
                priority: "P2".to_string(),
                description: "Integrate external localization source (GPS/RTK)".to_string(),
                impact: "high".to_string(),
                complexity: "hard".to_string(),
                implementation: "Add GPS or RTK-GNSS as external reference. \
                                 Use Extended Kalman Filter to fuse with odometry. \
                                 Requires 1-2 weeks integration + outdoor testing."
                    .to_string(),
            },
        ]
    }

    fn actions_oscillation() -> Vec<Action> {
        vec![
            Action {
                priority: "P0".to_string(),
                description: "Reduce planner oscillation gains".to_string(),
                impact: "medium".to_string(),
                complexity: "easy".to_string(),
                implementation: "Decrease DWA/TEB 'oscillation_v' parameter. \
                                 Reduce gain on heading controller. \
                                 This dampens oscillations but may slow convergence. \
                                 Tune incrementally: 10% reduction at a time."
                    .to_string(),
            },
            Action {
                priority: "P1".to_string(),
                description: "Increase minimum planning timeout".to_string(),
                impact: "medium".to_string(),
                complexity: "easy".to_string(),
                implementation: "Increase 'controller_patience' parameter. \
                                 Gives planner more time before triggering replanning. \
                                 Prevents reactive replanning that causes oscillation."
                    .to_string(),
            },
            Action {
                priority: "P2".to_string(),
                description: "Switch to trajectory-following planner with look-ahead".to_string(),
                impact: "high".to_string(),
                complexity: "hard".to_string(),
                implementation: "Implement Pure Pursuit or Stanley controller. \
                                 Look-ahead removes local oscillations. \
                                 Requires trajectory generation system (2-3 weeks)."
                    .to_string(),
            },
        ]
    }

    fn actions_costmap_anomaly() -> Vec<Action> {
        vec![
            Action {
                priority: "P0".to_string(),
                description: "Check costmap inflation and padding parameters".to_string(),
                impact: "low".to_string(),
                complexity: "easy".to_string(),
                implementation: "Review costmap inflation_radius and padding. \
                                 Sudden changes may indicate parameter misalignment. \
                                 Verify inflation matches robot footprint."
                    .to_string(),
            },
            Action {
                priority: "P1".to_string(),
                description: "Enable costmap smoothing to reduce sudden changes".to_string(),
                impact: "medium".to_string(),
                complexity: "medium".to_string(),
                implementation: "Enable costmap smoothing filter. \
                                 Add temporal filtering to reduce noise. \
                                 Stabilizes planning around detected obstacle transitions."
                    .to_string(),
            },
            Action {
                priority: "P2".to_string(),
                description: "Improve sensor fusion for costmap generation".to_string(),
                impact: "medium".to_string(),
                complexity: "hard".to_string(),
                implementation: "Fuse multiple sensor sources (LiDAR + camera + radar). \
                                 Use Dempster-Shafer or Bayesian fusion. \
                                 Reduces single-sensor artifacts (2-4 weeks integration)."
                    .to_string(),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_near_collision_recommendations() {
        let failure = Failure::new(
            "near_collision".to_string(),
            Utc::now(),
            0.95,
            "high".to_string(),
            "LiDAR obstacle too close".to_string(),
        );

        let actions = ActionRecommender::recommend(&failure);
        assert!(!actions.is_empty());
        assert_eq!(actions[0].priority, "P0");
    }

    #[test]
    fn test_all_failure_types_have_actions() {
        let failure_types = vec![
            "near_collision",
            "perception_failure",
            "sensor_dropout",
            "communication_loss",
            "navigation_deadlock",
            "localization_loss",
            "oscillation",
            "costmap_anomaly",
        ];

        for failure_type in failure_types {
            let failure = Failure::new(
                failure_type.to_string(),
                Utc::now(),
                0.75,
                "medium".to_string(),
                "Test failure".to_string(),
            );

            let actions = ActionRecommender::recommend(&failure);
            assert!(!actions.is_empty(), "No actions for {}", failure_type);
            assert!(actions.len() >= 2, "Too few actions for {}", failure_type);
        }
    }

    #[test]
    fn test_action_priorities_valid() {
        let failure = Failure::new(
            "near_collision".to_string(),
            Utc::now(),
            0.75,
            "high".to_string(),
            "Test".to_string(),
        );

        let actions = ActionRecommender::recommend(&failure);
        for action in actions {
            assert!(
                action.priority == "P0" || action.priority == "P1" || action.priority == "P2",
                "Invalid priority: {}",
                action.priority
            );
        }
    }
}
