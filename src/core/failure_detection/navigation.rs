/// Navigation Failure Detector
///
/// Detects:
/// - Planner timeout
/// - Controller oscillation
/// - Recovery loop (excessive recovery behavior triggers)
/// - Goal failure
/// - Path deviation

use super::{DetectedFailure, FailureDetector, FailureDomain, FailureSeverity};
use crate::core::event::MissionEvent;
use crate::core::timeline_correlation::NormalizedEvent;
use chrono::Duration;

pub struct NavigationFailureDetector;

impl NavigationFailureDetector {
    /// Detect planner timeout: explicit timeout event or no path found
    fn detect_planner_timeout(events: &[NormalizedEvent]) -> Vec<DetectedFailure> {
        let mut failures = Vec::new();
        let mut plan_requests = Vec::new();

        for event in events {
            match &event.event {
                MissionEvent::NavigationDecision {
                    timestamp,
                    decision_type,
                    ..
                } if decision_type == "plan_request" => {
                    plan_requests.push((event.id.clone(), *timestamp));
                }
                MissionEvent::NavigationDecision {
                    timestamp,
                    decision_type,
                    ..
                } if decision_type == "plan_timeout" => {
                    let duration_ms = plan_requests
                        .iter()
                        .rev()
                        .find_map(|(_, req_time)| {
                            let dur = (*timestamp - *req_time).num_milliseconds();
                            if dur > 0 { Some(dur) } else { None }
                        })
                        .unwrap_or(5000);

                    failures.push(
                        DetectedFailure::new(
                            "planner_timeout",
                            FailureDomain::Navigation,
                            *timestamp,
                            1.0, // Explicit timeout = high confidence
                            FailureSeverity::High,
                            format!("Planner timeout after {}ms", duration_ms),
                        )
                        .with_event_ids(vec![event.id.clone()]),
                    );
                }
                _ => {}
            }
        }

        failures
    }

    /// Detect oscillation: robot cycling through same positions
    fn detect_oscillation(events: &[NormalizedEvent]) -> Vec<DetectedFailure> {
        let mut failures = Vec::new();
        let mut poses = Vec::new();

        for event in events {
            if let MissionEvent::RobotPose { timestamp, pose, .. } = &event.event {
                poses.push((event.id.clone(), *timestamp, pose.x, pose.y));
            }
        }

        // Check for oscillation in windows of 5+ poses
        const OSCILLATION_WINDOW: usize = 5;
        const MAX_POSITION_VARIANCE: f64 = 0.5; // 0.5m radius

        if poses.len() > OSCILLATION_WINDOW {
            for i in 0..=poses.len() - OSCILLATION_WINDOW {
                let window = &poses[i..i + OSCILLATION_WINDOW];

                // Check if all poses are within a small area
                let avg_x = window.iter().map(|(_, _, x, _)| x).sum::<f64>() / window.len() as f64;
                let avg_y = window.iter().map(|(_, _, _, y)| y).sum::<f64>() / window.len() as f64;

                let max_distance = window
                    .iter()
                    .map(|(_, _, x, y)| {
                        let dx = x - avg_x;
                        let dy = y - avg_y;
                        (dx * dx + dy * dy).sqrt()
                    })
                    .fold(0.0, f64::max);

                if max_distance < MAX_POSITION_VARIANCE {
                    let (_, start_time, _, _) = window[0];
                    let (_, end_time, _, _) = window[window.len() - 1];
                    let duration = (end_time - start_time).num_seconds();

                    if duration > 2 {
                        // Must persist for >2 seconds
                        failures.push(
                            DetectedFailure::new(
                                "oscillation",
                                FailureDomain::Navigation,
                                start_time,
                                0.85,
                                FailureSeverity::High,
                                format!(
                                    "Robot oscillating in {:.2}m radius for {}s",
                                    max_distance, duration
                                ),
                            )
                            .with_event_ids(window.iter().map(|(id, _, _, _)| id.clone()).collect()),
                        );
                        break;
                    }
                }
            }
        }

        failures
    }

    /// Detect recovery loop: recovery behavior triggered repeatedly
    fn detect_recovery_loop(events: &[NormalizedEvent]) -> Vec<DetectedFailure> {
        let mut failures = Vec::new();
        let mut recovery_events = Vec::new();

        for event in events {
            if let MissionEvent::NavigationDecision {
                timestamp,
                decision_type,
                ..
            } = &event.event
            {
                if decision_type.starts_with("recovery_") {
                    recovery_events.push((event.id.clone(), *timestamp));
                }
            }
        }

        // Detect if recovery is triggered >3 times in 30 seconds
        const RECOVERY_THRESHOLD: usize = 3;
        const RECOVERY_WINDOW_SECS: i64 = 30;

        for i in 0..recovery_events.len() {
            let window_start = recovery_events[i].1;
            let window_end = window_start + Duration::seconds(RECOVERY_WINDOW_SECS);

            let recoveries_in_window: Vec<_> = recovery_events
                .iter()
                .filter(|(_, ts)| *ts >= window_start && *ts <= window_end)
                .collect();

            if recoveries_in_window.len() >= RECOVERY_THRESHOLD {
                failures.push(
                    DetectedFailure::new(
                        "recovery_loop",
                        FailureDomain::Navigation,
                        window_start,
                        0.90,
                        FailureSeverity::High,
                        format!(
                            "{} recovery behaviors triggered in {}s",
                            recoveries_in_window.len(),
                            RECOVERY_WINDOW_SECS
                        ),
                    )
                    .with_event_ids(
                        recoveries_in_window.iter().map(|(id, _)| id.clone()).collect(),
                    ),
                );
                break;
            }
        }

        failures
    }

    /// Detect goal failure: goal reached but failed
    fn detect_goal_failure(events: &[NormalizedEvent]) -> Vec<DetectedFailure> {
        let mut failures = Vec::new();

        for event in events {
            if let MissionEvent::NavigationDecision {
                timestamp,
                decision_type,
                ..
            } = &event.event
            {
                if decision_type == "goal_failed" || decision_type == "goal_unreachable" {
                    let severity = if decision_type == "goal_unreachable" {
                        FailureSeverity::High
                    } else {
                        FailureSeverity::Medium
                    };

                    failures.push(
                        DetectedFailure::new(
                            "goal_failure",
                            FailureDomain::Navigation,
                            *timestamp,
                            0.95,
                            severity,
                            "Navigation goal could not be reached".to_string(),
                        )
                        .with_event_ids(vec![event.id.clone()]),
                    );
                }
            }
        }

        failures
    }

    /// Detect path deviation: actual path deviates from planned
    fn detect_path_deviation(events: &[NormalizedEvent]) -> Vec<DetectedFailure> {
        let mut failures = Vec::new();

        // Would compare planned path vs actual trajectory
        // For now, check for explicit path_deviation events
        for event in events {
            if let MissionEvent::NavigationDecision {
                timestamp,
                decision_type,
                ..
            } = &event.event
            {
                if decision_type == "path_deviation" || decision_type == "off_path" {
                    failures.push(
                        DetectedFailure::new(
                            "path_deviation",
                            FailureDomain::Navigation,
                            *timestamp,
                            0.80,
                            FailureSeverity::Medium,
                            "Robot deviated from planned path".to_string(),
                        )
                        .with_event_ids(vec![event.id.clone()]),
                    );
                }
            }
        }

        failures
    }
}

impl FailureDetector for NavigationFailureDetector {
    fn detect(&self, events: &[NormalizedEvent]) -> Vec<DetectedFailure> {
        let mut all_failures = Vec::new();

        all_failures.extend(Self::detect_planner_timeout(events));
        all_failures.extend(Self::detect_oscillation(events));
        all_failures.extend(Self::detect_recovery_loop(events));
        all_failures.extend(Self::detect_goal_failure(events));
        all_failures.extend(Self::detect_path_deviation(events));

        all_failures
    }

    fn domain(&self) -> FailureDomain {
        FailureDomain::Navigation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_detector_creation() {
        let detector = NavigationFailureDetector;
        assert_eq!(detector.domain(), FailureDomain::Navigation);
    }

    #[test]
    fn test_empty_events() {
        let detector = NavigationFailureDetector;
        let events = vec![];
        let failures = detector.detect(&events);
        assert_eq!(failures.len(), 0);
    }

    #[test]
    fn test_plan_timeout_detection() {
        let failures = NavigationFailureDetector::detect_planner_timeout(&[]);
        assert_eq!(failures.len(), 0);
    }
}
