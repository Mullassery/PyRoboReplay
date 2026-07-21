use pyroboreplay::core::{
    ComplianceEvent, ComplianceReportGenerator, ComplianceConfig, ProximityZoneEvent,
    ProximityZoneType, EmergencyStopEvent, SpeedComplianceEvent, OperatorPresenceEvent,
};
use chrono::Utc;

fn main() {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║  PyRoboReplay: ISO 3691-4 Regulatory Compliance Reporting     ║");
    println!("║  Phase 7.3: Advanced Forensics                              ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    println!("═══════════════════════════════════════════════════════════════════");
    println!("DEMO 1: GENERATE COMPLIANCE REPORT FOR CLEAN MISSION");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let config = ComplianceConfig::default();
    let generator = ComplianceReportGenerator::new(config);
    let now = Utc::now();

    let clean_events = vec![
        ComplianceEvent::SpeedCompliance(SpeedComplianceEvent {
            robot_id: "warehouse_bot_1".to_string(),
            timestamp: now,
            actual_speed_mps: 1.5,
            limit_mps: 2.0,
            compliant: true,
        }),
        ComplianceEvent::OperatorPresence(OperatorPresenceEvent {
            robot_id: "warehouse_bot_1".to_string(),
            timestamp: now,
            operator_id: Some("operator_1".to_string()),
            present: true,
        }),
        ComplianceEvent::ProximityZone(ProximityZoneEvent {
            robot_id: "warehouse_bot_1".to_string(),
            timestamp: now + chrono::Duration::seconds(5),
            zone_id: "zone_a1".to_string(),
            zone_type: ProximityZoneType::WarningZone,
            distance_m: 1.5,
            action_taken: "Slowed down".to_string(),
        }),
    ];

    let report = generator.generate_report("mission_warehouse_001", &clean_events);

    println!("Mission: {}", report.mission_id);
    println!("Standard: {}", report.standard);
    println!("Report ID: {}", report.report_id);
    println!("Overall Compliant: {}", report.overall_compliant);
    println!("Violation Count: {}", report.violations.len());
    println!("Summary: {}\n", report.summary);

    println!("═══════════════════════════════════════════════════════════════════");
    println!("DEMO 2: GENERATE COMPLIANCE REPORT WITH VIOLATIONS");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let violation_events = vec![
        ComplianceEvent::ProximityZone(ProximityZoneEvent {
            robot_id: "warehouse_bot_2".to_string(),
            timestamp: now,
            zone_id: "zone_b1".to_string(),
            zone_type: ProximityZoneType::SafetyZone,
            distance_m: 0.3,
            action_taken: "Emergency stop".to_string(),
        }),
        ComplianceEvent::SpeedCompliance(SpeedComplianceEvent {
            robot_id: "warehouse_bot_2".to_string(),
            timestamp: now + chrono::Duration::seconds(2),
            actual_speed_mps: 2.8,
            limit_mps: 2.0,
            compliant: false,
        }),
        ComplianceEvent::EmergencyStop(EmergencyStopEvent {
            robot_id: "warehouse_bot_2".to_string(),
            timestamp: now + chrono::Duration::seconds(3),
            cause: "Obstacle detected in safety zone".to_string(),
            stop_distance_m: 0.5,
            recovery_time_ms: Some(45_000),
        }),
    ];

    let violation_report = generator.generate_report("mission_warehouse_002", &violation_events);

    println!("Mission: {}", violation_report.mission_id);
    println!("Overall Compliant: {}", violation_report.overall_compliant);
    println!("Violation Count: {}", violation_report.violations.len());
    println!("Summary: {}\n", violation_report.summary);

    println!("Violations by Type:");
    for (violation_type, count) in &violation_report.violation_count_by_type {
        println!("  {}: {}", violation_type, count);
    }
    println!();

    println!("Violation Details:");
    for (i, violation) in violation_report.violations.iter().enumerate() {
        println!("  {}. Type: {:?}", i + 1, violation.violation_type);
        println!("     Robot: {}", violation.robot_id);
        println!("     Severity: {:?}", violation.severity);
        println!("     Description: {}\n", violation.description);
    }

    println!("═══════════════════════════════════════════════════════════════════");
    println!("DEMO 3: OPERATOR ABSENCE VIOLATION");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let absence_start = Utc::now();
    let absence_events = vec![
        ComplianceEvent::OperatorPresence(OperatorPresenceEvent {
            robot_id: "warehouse_bot_3".to_string(),
            timestamp: absence_start,
            operator_id: Some("operator_2".to_string()),
            present: false,
        }),
        ComplianceEvent::OperatorPresence(OperatorPresenceEvent {
            robot_id: "warehouse_bot_3".to_string(),
            timestamp: absence_start + chrono::Duration::seconds(3),
            operator_id: Some("operator_2".to_string()),
            present: true,
        }),
    ];

    let absence_report = generator.generate_report("mission_warehouse_003", &absence_events);

    println!("Operator Absence Violations: {}", absence_report.violations.len());
    if !absence_report.violations.is_empty() {
        let v = &absence_report.violations[0];
        println!("  Violation: {:?}", v.violation_type);
        println!("  Duration: {}", v.description);
        println!("  Severity: {:?}\n", v.severity);
    }

    println!("═══════════════════════════════════════════════════════════════════");
    println!("DEMO 4: CUSTOM COMPLIANCE CONFIGURATION");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let strict_config = ComplianceConfig {
        min_proximity_distance_m: 1.0,
        emergency_stop_recovery_timeout_ms: 20_000,
        max_speed_mps: 1.5,
        operator_motion_tolerance_ms: 500,
    };

    let strict_generator = ComplianceReportGenerator::new(strict_config);

    let strict_test_events = vec![
        ComplianceEvent::ProximityZone(ProximityZoneEvent {
            robot_id: "warehouse_bot_4".to_string(),
            timestamp: now,
            zone_id: "zone_c1".to_string(),
            zone_type: ProximityZoneType::WarningZone,
            distance_m: 0.8,
            action_taken: "Alert".to_string(),
        }),
    ];

    let strict_report = strict_generator.generate_report("mission_warehouse_004", &strict_test_events);

    println!("Default Config: min_proximity_distance_m = 0.5m");
    println!("Strict Config: min_proximity_distance_m = 1.0m");
    println!("  Event distance: 0.8m");
    println!("  Strict Config Result:");
    println!("    Compliant: {}",strict_report.overall_compliant);
    println!("    Violations: {}\n", strict_report.violations.len());

    println!("═══════════════════════════════════════════════════════════════════");
    println!("ISO 3691-4 COMPLIANCE FEATURES ENABLED");
    println!("═══════════════════════════════════════════════════════════════════\n");

    println!("✓ Proximity zone violation detection");
    println!("✓ Emergency stop monitoring");
    println!("✓ Speed limit compliance checking");
    println!("✓ Operator presence verification");
    println!("✓ Configurable compliance thresholds");
    println!("✓ Violation severity classification");
    println!("✓ Audit-ready compliance reporting");
    println!("✓ Type-based violation aggregation");
    println!("✓ ISO 3691-4 standard compliance\n");

    println!("═══════════════════════════════════════════════════════════════════");
    println!("✨ Phase 7.3: Regulatory Compliance Complete");
    println!("═══════════════════════════════════════════════════════════════════\n");
}
