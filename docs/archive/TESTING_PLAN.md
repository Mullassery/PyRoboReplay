# PyRoboReplay Testing Plan (Phase 1-3)

**Scope**: Complete test coverage for all three phases  
**Target**: 85%+ code coverage  
**Time Estimate**: 2-3 weeks implementation  
**Status**: Planning (ready to implement)

---

## Testing Strategy

### Pyramid Approach
```
         ╱╲
        ╱  ╲          End-to-End Tests (10%)
       ╱────╲
      ╱      ╲        Integration Tests (25%)
     ╱────────╲
    ╱          ╲      Unit Tests (65%)
   ╱____________╲
```

### Test Types
- **Unit Tests**: Individual functions, isolated
- **Integration Tests**: Cross-module interactions
- **End-to-End Tests**: Full workflow (Mission → Diagnosis → Export)
- **Fixture Tests**: Real ROS bag scenarios
- **Performance Tests**: Latency requirements
- **Format Tests**: GIS export validation

---

## Phase 1: Failure Detection & Diagnosis Tests

### 1.1 Unit Tests: AnomalyDetector (src/core/anomaly_detector.rs)

#### 1.1.1 Near Collision Detection
```rust
#[test]
fn test_detect_near_collision_above_threshold() {
    // Arrange: LiDAR ranges with one value < 0.5m
    let events = vec![MissionEvent::LidarScan {
        ranges: vec![0.3, 2.0, 3.0],
        intensities: vec![0.5, 0.5, 0.5],
        data: LidarData { /* ... */ },
    }];
    
    // Act
    let detector = AnomalyDetector::new(events);
    let failures = detector.detect_near_collision();
    
    // Assert
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].failure_type, "near_collision");
    assert!(failures[0].confidence > 0.5);
    assert_eq!(failures[0].severity, "high");
}

#[test]
fn test_detect_near_collision_critical_range() {
    // Test < 0.25m = critical
}

#[test]
fn test_no_failure_when_safe_range() {
    // All ranges > 0.5m = no failure
}

#[test]
fn test_near_collision_evidence_collection() {
    // Verify min_range_m and threshold_m in evidence
}
```

#### 1.1.2 Perception Failure Detection
```rust
#[test]
fn test_detect_low_confidence_detections() {
    // Arrange: Camera frames with <50% confidence detections
    // Act: detector.detect_perception_failure()
    // Assert: failure_type == "perception_failure", ratio > 0.3
}

#[test]
fn test_no_failure_when_high_confidence() {
    // All detections > threshold = no failure
}

#[test]
fn test_perception_failure_confidence_ratio() {
    // Verify (low_conf_count / total) maps to confidence
}

#[test]
fn test_perception_failure_requires_minimum_sample() {
    // Need at least 10 frames to trigger
}
```

#### 1.1.3 Sensor Dropout Detection
```rust
#[test]
fn test_detect_sensor_gap_over_threshold() {
    // Arrange: 2-second gap between LiDAR messages
    // Act: detector.detect_sensor_dropout()
    // Assert: failure detected, gap_seconds > 1.0
}

#[test]
fn test_no_failure_continuous_stream() {
    // Regular 100ms intervals = no failure
}

#[test]
fn test_sensor_dropout_identifies_sensor_type() {
    // Verify which sensor stopped reporting
}

#[test]
fn test_multiple_sensors_can_dropout() {
    // If multiple sensors stop, multiple failures
}
```

#### 1.1.4 Communication Loss Detection
```rust
#[test]
fn test_detect_message_rate_drop() {
    // Arrange: 3x average gap
    // Act: detector.detect_communication_loss()
    // Assert: failure, gap ratio in evidence
}

#[test]
fn test_no_failure_consistent_timing() {
    // Regular intervals = no failure
}

#[test]
fn test_communication_loss_needs_minimum_messages() {
    // Require >10 messages to establish baseline
}
```

#### 1.1.5 Navigation Deadlock Detection
```rust
#[test]
fn test_detect_excessive_replanning() {
    // Arrange: >20 NavigationDecision events
    // Act: detector.detect_navigation_deadlock()
    // Assert: failure detected
}

#[test]
fn test_deadlock_with_zero_velocity() {
    // Replanning + 70% zero-velocity states = deadlock
}

#[test]
fn test_no_failure_convergent_replanning() {
    // Replanning that makes progress = OK
}
```

#### 1.1.6 Localization Loss Detection
```rust
#[test]
fn test_detect_low_pose_confidence() {
    // Arrange: RobotPose with confidence < 0.5
    // Act: detector.detect_localization_loss()
    // Assert: failure, confidence_level in evidence
}

#[test]
fn test_no_failure_high_confidence_pose() {
    // Confidence > 0.8 = safe
}

#[test]
fn test_localization_multiple_low_confidence_events() {
    // Each low confidence = separate failure
}
```

#### 1.1.7 Oscillation Detection
```rust
#[test]
fn test_detect_oscillating_movement() {
    // Arrange: Path with frequent direction changes
    // Act: detector.detect_oscillation()
    // Assert: direction_changes > len(positions) / 3
}

#[test]
fn test_oscillation_requires_duration() {
    // Need >5 seconds of data to detect
}

#[test]
fn test_no_failure_smooth_path() {
    // Consistent direction = OK
}

#[test]
fn test_oscillation_velocity_calculation() {
    // Verify distance_traveled / time_elapsed
}
```

#### 1.1.8 Costmap Anomaly Detection
```rust
#[test]
fn test_detect_sudden_obstacle_increase() {
    // Arrange: Costmap change from 50 to 90 obstacles
    // Act: detector.detect_costmap_anomaly()
    // Assert: change_ratio > 0.8, anomaly detected
}

#[test]
fn test_no_failure_gradual_changes() {
    // <10% change = OK
}

#[test]
fn test_costmap_anomaly_tracks_previous_state() {
    // Verify comparison to previous obstacle count
}
```

#### 1.1.9 Edge Cases
```rust
#[test]
fn test_empty_event_stream() {
    // No events = no failures
    let detector = AnomalyDetector::new(vec![]);
    assert_eq!(detector.detect_all().len(), 0);
}

#[test]
fn test_single_event_mission() {
    // Only 1 event = limited detection
}

#[test]
fn test_mixed_event_types() {
    // All 8 failure types in one mission
}

#[test]
fn test_detector_independence() {
    // Each detector works independently
    // Failure in detector X doesn't affect detector Y
}

#[test]
fn test_confidence_bounds() {
    // All confidences in [0.0, 1.0]
    let detector = AnomalyDetector::new(create_test_events());
    for failure in detector.detect_all() {
        assert!(failure.confidence >= 0.0);
        assert!(failure.confidence <= 1.0);
    }
}
```

---

### 1.2 Unit Tests: ExplanationGenerator (src/core/explanation.rs)

#### 1.2.1 Explanation Generation
```rust
#[test]
fn test_explain_near_collision() {
    let failure = Failure::new(
        "near_collision".to_string(),
        Utc::now(),
        0.95,
        "high".to_string(),
        "Test".to_string(),
    );
    let explanation = ExplanationGenerator::explain(&failure);
    
    assert!(explanation.contains("obstacle"));
    assert!(explanation.contains("collision"));
    assert!(explanation.contains("high")); // severity
}

#[test]
fn test_all_8_failure_types_have_explanations() {
    let types = vec![
        "near_collision", "perception_failure", "sensor_dropout",
        "communication_loss", "navigation_deadlock", "localization_loss",
        "oscillation", "costmap_anomaly"
    ];
    
    for ftype in types {
        let failure = Failure::new(
            ftype.to_string(),
            Utc::now(), 0.75, "medium".to_string(), "Test".to_string()
        );
        let explanation = ExplanationGenerator::explain(&failure);
        assert!(!explanation.contains("Unknown"), "No explanation for {}", ftype);
    }
}

#[test]
fn test_explanation_includes_severity_context() {
    let critical = create_failure_with_severity("critical");
    let explanation = ExplanationGenerator::explain(&critical);
    assert!(explanation.contains("critical"));
    assert!(explanation.contains("immediate"));
}

#[test]
fn test_explanation_uses_evidence() {
    let mut failure = Failure::new(...);
    failure.evidence.insert("min_range_m".to_string(), "0.30".to_string());
    let explanation = ExplanationGenerator::explain(&failure);
    assert!(explanation.contains("0.30"));
}

#[test]
fn test_explanation_sentence_count() {
    // Each explanation should be 2-3 sentences
    let explanation = ExplanationGenerator::explain(&failure);
    let sentences = explanation.split('.').filter(|s| !s.is_empty()).count();
    assert!(sentences >= 2 && sentences <= 4);
}
```

---

### 1.3 Unit Tests: ActionRecommender (src/core/failure_actions.rs)

#### 1.3.1 Recommendation Generation
```rust
#[test]
fn test_recommend_for_near_collision() {
    let failure = create_near_collision_failure();
    let actions = ActionRecommender::recommend(&failure);
    
    assert_eq!(actions.len(), 3); // 3 recommendations
    assert_eq!(actions[0].priority, "P0"); // Priority order
    assert_eq!(actions[1].priority, "P1");
    assert_eq!(actions[2].priority, "P2");
}

#[test]
fn test_all_8_failure_types_have_recommendations() {
    let types = vec![...]; // 8 types
    for ftype in types {
        let failure = create_failure_of_type(ftype);
        let actions = ActionRecommender::recommend(&failure);
        assert!(actions.len() >= 2, "Too few actions for {}", ftype);
    }
}

#[test]
fn test_recommendation_priorities() {
    let failure = create_failure();
    let actions = ActionRecommender::recommend(&failure);
    
    for action in &actions {
        assert!(
            action.priority == "P0" || action.priority == "P1" || action.priority == "P2"
        );
    }
}

#[test]
fn test_recommendation_impact_levels() {
    let failure = create_failure();
    let actions = ActionRecommender::recommend(&failure);
    
    for action in &actions {
        assert!(
            vec!["high", "medium", "low"].contains(&action.impact.as_str())
        );
    }
}

#[test]
fn test_recommendation_complexity_levels() {
    let failure = create_failure();
    let actions = ActionRecommender::recommend(&failure);
    
    for action in &actions {
        assert!(
            vec!["easy", "medium", "hard"].contains(&action.complexity.as_str())
        );
    }
}

#[test]
fn test_recommendation_has_implementation_guide() {
    let failure = create_failure();
    let actions = ActionRecommender::recommend(&failure);
    
    for action in &actions {
        assert!(!action.implementation.is_empty());
        assert!(action.implementation.len() > 50);
    }
}
```

---

### 1.4 Integration Tests: Full Diagnostic Pipeline

#### 1.4.1 End-to-End Failure Analysis
```rust
#[test]
fn test_end_to_end_collision_diagnosis() {
    // Arrange: Create mission with near_collision scenario
    let mission = create_test_mission_with_collision();
    
    // Act: Full pipeline
    let failures = mission.detect_failures();
    let failure = failures.iter().find(|f| f.failure_type == "near_collision");
    
    let analysis = mission.analyze_failure(failure.timestamp_seconds);
    let explanation = ExplanationGenerator::explain(failure);
    let actions = ActionRecommender::recommend(failure);
    
    // Assert: All steps succeed
    assert!(failure.is_some());
    assert!(!analysis.primary_hypothesis.is_empty());
    assert!(!explanation.is_empty());
    assert!(!actions.is_empty());
}

#[test]
fn test_multiple_failures_in_single_mission() {
    // Mission with 3+ different failure types
    let failures = mission.detect_failures();
    
    assert!(failures.len() >= 3);
    
    // Each failure independently analyzable
    for failure in &failures {
        let analysis = mission.analyze_failure(failure.timestamp_seconds);
        assert!(!analysis.primary_hypothesis.is_empty());
    }
}

#[test]
fn test_failure_evidence_flows_to_explanation() {
    // Evidence → Explanation → Recommendation chain
    let failure = detect_failure();
    let explanation = explain_failure(&failure);
    
    // Explanation should reference evidence
    for (key, value) in &failure.evidence {
        // Some evidence should appear in explanation or action
    }
}
```

---

### 1.5 Python API Tests

#### 1.5.1 Mission.detect_failures()
```python
def test_detect_failures_returns_list():
    failures = mission.detect_failures()
    assert isinstance(failures, list)
    assert all(isinstance(f, Failure) for f in failures)

def test_detect_failures_empty_mission():
    empty_mission = Mission.from_ros_bag("empty.bag")
    failures = empty_mission.detect_failures()
    assert failures == []

def test_failure_object_properties():
    failures = mission.detect_failures()
    for failure in failures:
        assert isinstance(failure.get_failure_type(), str)
        assert 0.0 <= failure.get_confidence() <= 1.0
        assert failure.get_severity() in ["critical", "high", "medium", "low"]
        assert isinstance(failure.get_timestamp(), float)
        assert isinstance(failure.get_description(), str)
        assert isinstance(failure.get_affected_systems(), list)
        assert isinstance(failure.get_evidence(), dict)
```

#### 1.5.2 Mission.analyze_failure()
```python
def test_analyze_failure_returns_analysis():
    failures = mission.detect_failures()
    if failures:
        analysis = mission.analyze_failure(failures[0].get_timestamp())
        assert isinstance(analysis, RootCauseAnalysis)

def test_analysis_properties():
    analysis = mission.analyze_failure(timestamp)
    assert isinstance(analysis.get_primary_hypothesis(), str)
    assert isinstance(analysis.get_hypotheses(), list)
    assert 0.0 <= analysis.get_diagnostic_confidence() <= 1.0

def test_analyze_failure_invalid_timestamp():
    with pytest.raises(Exception):  # ValueError expected
        mission.analyze_failure(99999999.0)
```

#### 1.5.3 Mission.explain_failure()
```python
def test_explain_failure_returns_string():
    explanation = mission.explain_failure(timestamp)
    assert isinstance(explanation, str)
    assert len(explanation) > 50

def test_explanation_contains_failure_type_info():
    failure = mission.detect_failures()[0]
    explanation = mission.explain_failure(failure.get_timestamp())
    assert failure.get_failure_type().lower() in explanation.lower()
```

#### 1.5.4 Mission.recommend_actions()
```python
def test_recommend_actions_returns_list():
    actions = mission.recommend_actions(timestamp)
    assert isinstance(actions, list)
    assert all(isinstance(a, Action) for a in actions)

def test_action_properties():
    actions = mission.recommend_actions(timestamp)
    for action in actions:
        assert action.get_priority() in ["P0", "P1", "P2"]
        assert action.get_impact() in ["high", "medium", "low"]
        assert action.get_complexity() in ["easy", "medium", "hard"]
        assert isinstance(action.get_description(), str)
        assert isinstance(action.get_implementation(), str)
```

---

## Phase 2: Cross-Mission Learning Tests

### 2.1 Unit Tests: CrossMissionAnalyzer

#### 2.1.1 Pattern Extraction
```rust
#[test]
fn test_learn_from_single_mission() {
    let mut analyzer = CrossMissionAnalyzer::new();
    let analysis = create_root_cause_analysis();
    
    let patterns = analyzer.learn_from_mission("mission_1", &analysis);
    assert!(!patterns.is_empty());
}

#[test]
fn test_pattern_library_accumulates() {
    let mut analyzer = CrossMissionAnalyzer::new();
    
    analyzer.learn_from_mission("mission_1", &analysis1);
    analyzer.learn_from_mission("mission_2", &analysis2);
    analyzer.learn_from_mission("mission_3", &analysis3);
    
    let library = analyzer.library;
    assert!(library.patterns.len() > 0);
}

#[test]
fn test_identical_patterns_merge() {
    // Same failure pattern in mission 1 and 2
    // Should update occurrence count, not create new pattern
}

#[test]
fn test_pattern_occurrence_tracking() {
    // Each occurrence tracked with: mission_id, timestamp, confidence
}
```

#### 2.1.2 Fleet Statistics
```rust
#[test]
fn test_fleet_statistics_empty() {
    let analyzer = CrossMissionAnalyzer::new();
    let stats = analyzer.fleet_statistics();
    
    assert_eq!(stats.mission_count, 0);
    assert_eq!(stats.total_failures, 0);
}

#[test]
fn test_fleet_statistics_aggregation() {
    // 3 missions: 2 failures, 1 failure, 3 failures = 6 total
    let stats = analyzer.fleet_statistics();
    
    assert_eq!(stats.mission_count, 3);
    assert_eq!(stats.total_failures, 6);
    assert_eq!(stats.failure_rate, 2.0); // 6 / 3
}

#[test]
fn test_most_common_failure_type() {
    // 4x near_collision, 2x perception_failure
    let stats = analyzer.fleet_statistics();
    assert_eq!(stats.most_common_failure, "near_collision");
}
```

#### 2.1.3 Hotspot Detection
```rust
#[test]
fn test_find_failure_zones() {
    let zones = analyzer.find_failure_zones();
    assert!(!zones.is_empty());
}

#[test]
fn test_hotspot_properties() {
    let zones = analyzer.find_failure_zones();
    for zone in zones {
        assert!(!zone.zone_id.is_empty());
        assert!(zone.failure_count > 0);
        assert!(!zone.dominant_failure_type.is_empty());
    }
}

#[test]
fn test_hotspot_radius_calculation() {
    // Zones should have meaningful radius (not 0, not infinite)
    let zones = analyzer.find_failure_zones();
    for zone in zones {
        assert!(zone.radius > 0.0);
        assert!(zone.radius < 1000.0); // Reasonable max
    }
}
```

---

## Phase 3: Geospatial Export Tests

### 3.1 Unit Tests: GeospatialExporter

#### 3.1.1 GeoJSON Export
```rust
#[test]
fn test_failures_to_geojson_structure() {
    let failures = create_test_failures();
    let geojson = GeospatialExporter::failures_to_geojson(&failures);
    
    assert_eq!(geojson.r#type, "FeatureCollection");
    assert_eq!(geojson.features.len(), failures.len());
}

#[test]
fn test_geojson_feature_properties() {
    let failure = create_near_collision_failure();
    let geojson = GeospatialExporter::failures_to_geojson(&[failure]);
    
    let props = &geojson.features[0].properties;
    assert!(props.contains_key("failure_type"));
    assert!(props.contains_key("severity"));
    assert!(props.contains_key("confidence"));
    assert!(props.contains_key("timestamp"));
}

#[test]
fn test_geojson_valid_geometry() {
    let geojson = GeospatialExporter::failures_to_geojson(&[...]);
    for feature in &geojson.features {
        assert_eq!(feature.geometry.r#type, "Point");
        assert_eq!(feature.geometry.coordinates.len(), 2);
    }
}

#[test]
fn test_geojson_serialization() {
    let geojson = GeospatialExporter::failures_to_geojson(&[...]);
    let json_str = serde_json::to_string(&geojson).unwrap();
    
    // Should be valid JSON
    let _: serde_json::Value = serde_json::from_str(&json_str).unwrap();
}

#[test]
fn test_geojson_empty_failures() {
    let geojson = GeospatialExporter::failures_to_geojson(&[]);
    assert_eq!(geojson.features.len(), 0);
}
```

#### 3.1.2 KML Export
```rust
#[test]
fn test_kml_valid_xml() {
    let kml = GeospatialExporter::to_kml(&create_test_failures());
    assert!(kml.contains("<?xml"));
    assert!(kml.contains("<kml"));
}

#[test]
fn test_kml_has_placemarks() {
    let failures = create_test_failures(); // 3 failures
    let kml = GeospatialExporter::to_kml(&failures);
    
    let placemark_count = kml.matches("<Placemark>").count();
    assert_eq!(placemark_count, failures.len());
}

#[test]
fn test_kml_contains_failure_info() {
    let kml = GeospatialExporter::to_kml(&[...]);
    assert!(kml.contains("near_collision"));
    assert!(kml.contains("Failure"));
}

#[test]
fn test_kml_google_earth_compatible() {
    // Should parse without errors in GE
}
```

#### 3.1.3 Raster Export
```rust
#[test]
fn test_create_coverage_raster() {
    let raster = GeospatialExporter::create_coverage_raster(100, 100, 0.1, 0.0, 0.0);
    
    assert_eq!(raster.width, 100);
    assert_eq!(raster.height, 100);
    assert_eq!(raster.resolution, 0.1);
}

#[test]
fn test_raster_data_initialized() {
    let raster = GeospatialExporter::create_coverage_raster(50, 50, 0.1, 0.0, 0.0);
    assert_eq!(raster.data.len(), 50);
    assert_eq!(raster.data[0].len(), 50);
}

#[test]
fn test_geotiff_metadata() {
    let raster = GeospatialExporter::create_coverage_raster(100, 100, 0.1, 0.0, 0.0);
    let metadata = GeospatialExporter::to_geotiff_metadata(&raster);
    
    assert!(metadata.contains("Width: 100"));
    assert!(metadata.contains("Height: 100"));
    assert!(metadata.contains("Resolution: 0.1"));
}
```

#### 3.1.4 Hotspot GeoJSON
```rust
#[test]
fn test_hotspots_to_geojson() {
    let hotspots = create_test_hotspots();
    let geojson = GeospatialExporter::hotspots_to_geojson(&hotspots);
    
    assert_eq!(geojson.features.len(), hotspots.len());
}

#[test]
fn test_hotspot_polygon_geometry() {
    let hotspot = create_hotspot_at(10.0, 20.0, 5.0);
    let geojson = GeospatialExporter::hotspots_to_geojson(&[hotspot]);
    
    let feature = &geojson.features[0];
    assert_eq!(feature.geometry.r#type, "Polygon");
    // Should be closed polygon (first = last point)
}

#[test]
fn test_hotspot_properties() {
    let geojson = GeospatialExporter::hotspots_to_geojson(&[...]);
    let props = &geojson.features[0].properties;
    
    assert!(props.contains_key("zone_id"));
    assert!(props.contains_key("failure_count"));
    assert!(props.contains_key("dominant_failure"));
}
```

---

## Integration Tests

### 4.1 Python-Rust Integration

#### 4.1.1 Mission → Failure → Analysis → Recommendation
```python
def test_full_diagnostic_pipeline():
    mission = Mission.from_ros_bag("test_mission.bag")
    
    # Step 1: Detect
    failures = mission.detect_failures()
    assert len(failures) > 0
    
    failure = failures[0]
    ts = failure.get_timestamp()
    
    # Step 2: Analyze
    analysis = mission.analyze_failure(ts)
    assert analysis is not None
    assert len(analysis.get_hypotheses()) > 0
    
    # Step 3: Explain
    explanation = mission.explain_failure(ts)
    assert len(explanation) > 0
    
    # Step 4: Recommend
    actions = mission.recommend_actions(ts)
    assert len(actions) > 0
    
    # Verify chain
    assert failure.get_failure_type() in explanation.lower()
```

#### 4.1.2 Geospatial Export Integration
```python
def test_geospatial_export_end_to_end():
    mission = Mission.from_ros_bag("test_mission.bag")
    
    # Export all formats
    geojson_str = mission.export_geojson()
    kml_str = mission.export_kml()
    tiff_meta = mission.export_geotiff_metadata()
    gpkg_meta = mission.export_geopackage_metadata()
    
    # Validate outputs
    assert len(geojson_str) > 0
    assert len(kml_str) > 0
    assert len(tiff_meta) > 0
    assert len(gpkg_meta) > 0
    
    # Parse JSON
    geojson_obj = json.loads(geojson_str)
    assert "features" in geojson_obj
```

---

## Fixture Tests

### 5.1 Test Missions (Synthetic ROS Bags)

#### 5.1.1 Scenario: Clean Mission (No Failures)
```python
def create_clean_mission_bag():
    # Robot navigates safely
    # All sensor readings nominal
    # Expected: 0 failures
```

#### 5.1.2 Scenario: Near-Collision Mission
```python
def create_collision_mission_bag():
    # Robot approaches obstacle at 0.3m
    # LiDAR triggers warning
    # Expected: 1 near_collision failure
```

#### 5.1.3 Scenario: Multi-Failure Mission
```python
def create_complex_mission_bag():
    # Multiple failures:
    # - Obstacle detection at t=100
    # - Sensor dropout at t=200
    # - Localization loss at t=300
    # Expected: 3+ failures of different types
```

#### 5.1.4 Scenario: GPS-Denied Zone
```python
def create_gps_denied_mission_bag():
    # Robot in indoor/shaded area
    # Localization confidence drops
    # Expected: localization_loss failure
```

---

## Performance Tests

### 6.1 Latency Requirements

```python
def test_detect_failures_latency():
    """detect_failures() should complete in <500ms for 100k events"""
    mission = load_large_mission(100000)
    
    start = time.time()
    failures = mission.detect_failures()
    elapsed = time.time() - start
    
    assert elapsed < 0.5  # 500ms

def test_analyze_failure_latency():
    """analyze_failure() should complete in <1s"""
    start = time.time()
    analysis = mission.analyze_failure(timestamp)
    elapsed = time.time() - start
    
    assert elapsed < 1.0  # 1 second

def test_export_geojson_latency():
    """export_geojson() should complete in <100ms"""
    start = time.time()
    geojson = mission.export_geojson()
    elapsed = time.time() - start
    
    assert elapsed < 0.1  # 100ms
```

### 6.2 Memory Requirements

```python
def test_mission_memory_usage():
    """Loading 1M event mission should use <1GB"""
    mission = load_large_mission(1000000)
    memory_mb = get_memory_usage() / 1024 / 1024
    
    assert memory_mb < 1024  # 1GB
```

---

## Edge Cases & Robustness

### 7.1 Malformed Data

```python
def test_mission_with_nulls():
    """Handle events with missing fields"""
    pass

def test_mission_with_nan_values():
    """Handle NaN in sensor readings"""
    pass

def test_mission_out_of_order_events():
    """Events not in chronological order"""
    pass

def test_mission_zero_duration():
    """Mission with single event"""
    pass

def test_mission_extreme_timestamps():
    """Timestamps far in past/future"""
    pass
```

### 7.2 Boundary Conditions

```python
def test_failure_confidence_exactly_zero():
    """Confidence = 0.0 edge case"""
    pass

def test_failure_confidence_exactly_one():
    """Confidence = 1.0 edge case"""
    pass

def test_severity_unknown_value():
    """Unrecognized severity string"""
    pass

def test_empty_evidence_dict():
    """Failure with no evidence"""
    pass
```

---

## GIS Format Validation Tests

### 8.1 QGIS Compatibility

```python
def test_geojson_imports_to_qgis():
    """Export GeoJSON can be opened in QGIS"""
    geojson = mission.export_geojson()
    
    # Write to file
    with open("/tmp/test.geojson", "w") as f:
        f.write(geojson)
    
    # Verify format
    data = json.loads(geojson)
    assert data["type"] == "FeatureCollection"
```

### 8.2 Google Earth Compatibility

```python
def test_kml_imports_to_google_earth():
    """Export KML can be opened in Google Earth"""
    kml = mission.export_kml()
    
    # Write to file
    with open("/tmp/test.kml", "w") as f:
        f.write(kml)
    
    # Validate XML structure
    root = xml.etree.ElementTree.fromstring(kml)
    assert root.tag.endswith("kml")
```

---

## Test Data Strategy

### Test Fixtures Location
```
tests/
├── fixtures/
│   ├── missions/
│   │   ├── clean_mission.bag
│   │   ├── collision_mission.bag
│   │   ├── multi_failure_mission.bag
│   │   └── gps_denied_mission.bag
│   ├── expected_outputs/
│   │   ├── collision_analysis.json
│   │   ├── multi_failure.geojson
│   │   └── coverage.tif
│   └── helpers.py (fixture creation)
├── unit/
│   ├── test_anomaly_detector.rs
│   ├── test_explanation.rs
│   ├── test_actions.rs
│   └── test_geospatial.rs
├── integration/
│   ├── test_diagnostic_pipeline.py
│   ├── test_export_formats.py
│   └── test_full_workflow.py
├── performance/
│   └── test_latency.py
└── robustness/
    └── test_edge_cases.py
```

---

## Test Execution Plan

### Week 1: Unit Tests
- Monday-Tuesday: Phase 1 anomaly detector tests (45 tests)
- Wednesday: Phase 1 explanation tests (15 tests)
- Thursday: Phase 1 action tests (15 tests)
- Friday: Phase 3 export tests (25 tests)

### Week 2: Integration Tests
- Monday-Tuesday: Python API integration (20 tests)
- Wednesday: Full diagnostic pipeline (10 tests)
- Thursday-Friday: Fixture-based scenarios (20 tests)

### Week 3: Validation
- Monday: Performance tests
- Tuesday: Edge case robustness tests
- Wednesday: GIS format validation
- Thursday-Friday: Bug fixes & optimization

---

## Coverage Goals

| Component | Target Coverage | Type |
|-----------|-----------------|------|
| anomaly_detector.rs | 90% | Unit |
| explanation.rs | 85% | Unit |
| failure_actions.rs | 85% | Unit |
| geospatial_export.rs | 80% | Unit |
| Python API | 95% | Integration |
| End-to-end workflow | 100% | Integration |
| **Overall** | **85%+** | Mixed |

---

## Quality Metrics

- ✅ All tests pass (0 failures)
- ✅ 85%+ code coverage
- ✅ <500ms latency for detect_failures()
- ✅ <1s latency for analyze_failure()
- ✅ <100ms latency for export functions
- ✅ All edge cases handled
- ✅ GIS formats valid
- ✅ Python API type-safe

---

**Status**: Ready for implementation  
**Estimated Timeline**: 2-3 weeks  
**Total Tests**: 170+ (40 unit + 50 integration + 25 performance + 20 edge case + 35 GIS)
