//! End-to-end coverage for the public Phase 12-15 / PyTerrainMap entry points.

use pyroboreplay::core::pyterrainmap::GridMapDocument;
use pyroboreplay::forensics;
use pyroboreplay::fusion::thermal_model::{ThermalCameraConfig, ThermalFrame};
use pyroboreplay::perception::object_detection::{BoundingBox, DetectedObject, ObjectClass};
use pyroboreplay::perception::retrospective_detection::DINODetection;
use pyroboreplay::phase14::session_report::{build_navigation_session, SessionSources};
use std::collections::HashMap;
use std::path::Path;

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn pyterrainmap_grid_fixture_is_consumable() {
    let doc = GridMapDocument::from_path(fixture("pyterrainmap-grid.json")).unwrap();
    let summary = doc.summary();
    assert_eq!(summary.cells, 5);
    assert_eq!(summary.total_observations, 5);
    assert!(doc.nearest_cell(40.7128, -74.0060).is_some());
}

#[test]
fn phase12_gap_analysis_runs_on_external_detections() {
    let analysis = forensics::analyze_detection_gaps(
        Vec::new(),
        vec![DINODetection {
            class_name: "person".to_string(),
            bbox: BoundingBox {
                x: 100.0,
                y: 50.0,
                width: 40.0,
                height: 90.0,
            },
            confidence: 0.8,
            distance_m: Some(3.0),
        }],
    );
    assert_eq!(analysis.stats.detection_gaps, 1);
}

#[test]
fn phase13_fusion_runs_on_external_thermal_frame() {
    let mut thermal_data = vec![300.0f32; 32 * 32];
    for y in 5..15 {
        for x in 5..15 {
            thermal_data[y * 32 + x] = 312.0;
        }
    }
    let frame = ThermalFrame {
        timestamp_sec: 1.0,
        frame_index: 0,
        thermal_data,
        width: 32,
        height: 32,
        config: ThermalCameraConfig::default(),
    };
    let rgb = vec![DetectedObject {
        id: 1,
        class: ObjectClass::Person,
        confidence: 0.9,
        bbox: BoundingBox {
            x: 5.0,
            y: 5.0,
            width: 10.0,
            height: 10.0,
        },
        distance_m: Some(2.0),
        velocity_ms: None,
        position_3d: None,
        trajectory_id: None,
        attributes: HashMap::new(),
    }];
    let analysis = forensics::fuse_rgb_thermal(rgb, frame);
    assert_eq!(analysis.stats.rgb_detections, 1);
    assert_eq!(analysis.stats.fused_detections, 1);
}

#[test]
fn phase14_builds_session_from_real_bag() {
    let bag = fixture("ros2_cdr_fixture.db3");
    let session = build_navigation_session(
        "integration",
        &SessionSources {
            bag: Some(&bag),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(session.public_summary().stats.total_events > 0);
}

#[test]
fn phase15_nav2_report_serializes() {
    let input: pyroboreplay::phase15::Nav2Diagnostics = serde_json::from_str(
        r#"{
          "localization": {"particle_spread_before": 0.2, "particle_spread_after": 1.4},
          "classification": {
            "failure_type": "door_crossing",
            "environment_type": "office",
            "scale_meters": 20.0
          }
        }"#,
    )
    .unwrap();
    let report = pyroboreplay::phase15::analyze_nav2_diagnostics(&input);
    assert!(!report.localization_issues.is_empty());
    assert!(report.classification.is_some());
    assert!(serde_json::to_string(&report)
        .unwrap()
        .contains("ParticleDivergence"));
}
