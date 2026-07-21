use pyroboreplay::core::{RecommendationEngine};

fn main() {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║   PyRoboReplay: Recommendation Engine - Phase 5 Task #3       ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    let mut engine = RecommendationEngine::new();

    println!("═══════════════════════════════════════════════════════════════════");
    println!("SCENARIO 1: NAVIGATION DEADLOCK FAILURE");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let deadlock_recs = engine.generate_for_failure(
        "navigation_deadlock",
        "obstacle_detected_but_unresolvable",
        "critical",
    );

    println!("Generated {} recommendations:\n", deadlock_recs.len());
    for (idx, rec) in deadlock_recs.iter().enumerate() {
        println!("Recommendation {}. {} [{}]", idx + 1, rec.title, rec.priority);
        println!("  Description: {}", rec.description);
        println!("  Expected Impact: {:.0}%", rec.expected_impact * 100.0);
        println!("  Effort Required: {:.0}%", rec.implementation_effort * 100.0);
        println!("  ROI Score: {:.2}", rec.roi_score);
        println!("  Risk of Regression: {:.0}%\n", rec.risk_of_regression * 100.0);
    }

    println!("═══════════════════════════════════════════════════════════════════");
    println!("SCENARIO 2: BATTERY DRAIN FAILURE");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let battery_recs = engine.generate_for_failure(
        "battery_drain",
        "suboptimal_path_planning",
        "high",
    );

    println!("Generated {} recommendations:\n", battery_recs.len());
    for (idx, rec) in battery_recs.iter().enumerate() {
        println!("Recommendation {}. {}", idx + 1, rec.title);
        println!("  Description: {}", rec.description);
        println!("  Subsystem: {}", rec.affected_subsystem);
        println!("  Fix Type: {}", rec.fix_type);
        println!("  Expected Impact: {:.0}%", rec.expected_impact * 100.0);
        println!("  Confidence: {:.0}%", rec.confidence * 100.0);
        println!("  ROI Score: {:.2}\n", rec.roi_score);
    }

    println!("═══════════════════════════════════════════════════════════════════");
    println!("COMPREHENSIVE RECOMMENDATION SET");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let set = engine.create_recommendation_set(
        "Navigation deadlock prevents mission completion",
        "Obstacle blocking all paths, navigation decisions fail repeatedly",
    );

    println!("Failure: {}", set.failure_description);
    println!("Root Cause: {}\n", set.root_cause);

    println!("Summary Statistics:");
    println!("  Total Recommendations: {}", set.stats.total_recommendations);
    println!("  Critical Priority: {}", set.stats.critical_count);
    println!("  High Priority: {}", set.stats.high_count);
    println!("  Medium Priority: {}", set.stats.medium_count);
    println!("  Low Priority: {}", set.stats.low_count);
    println!("  Average Expected Impact: {:.0}%", set.stats.avg_impact * 100.0);
    println!("  Average Implementation Effort: {:.0}%", set.stats.avg_effort * 100.0);
    println!("  Average Confidence: {:.0}%", set.stats.avg_confidence * 100.0);
    println!("  Best ROI Score: {:.2}\n", set.stats.best_roi);

    println!("═══════════════════════════════════════════════════════════════════");
    println!("QUICK WINS (High Impact, Low Effort)");
    println!("═══════════════════════════════════════════════════════════════════\n");

    if !set.quick_wins.is_empty() {
        for (idx, qw) in set.quick_wins.iter().enumerate() {
            println!("Quick Win {}. {}", idx + 1, qw.title);
            println!("  Impact: {:.0}% | Effort: {:.0}% | ROI: {:.2}",
                qw.expected_impact * 100.0,
                qw.implementation_effort * 100.0,
                qw.roi_score);
            println!("  Action: {}\n", qw.description);
        }
    } else {
        println!("No quick wins identified for this failure.\n");
    }

    println!("═══════════════════════════════════════════════════════════════════");
    println!("STRATEGIC IMPROVEMENTS (Medium-Term, Higher Impact)");
    println!("═══════════════════════════════════════════════════════════════════\n");

    if !set.strategic_improvements.is_empty() {
        for (idx, si) in set.strategic_improvements.iter().enumerate() {
            println!("Strategic Improvement {}. {}", idx + 1, si.title);
            println!("  Impact: {:.0}% | Effort: {:.0}% | ROI: {:.2}",
                si.expected_impact * 100.0,
                si.implementation_effort * 100.0,
                si.roi_score);
            println!("  Rationale: {}\n", si.rationale);
        }
    } else {
        println!("No strategic improvements in current recommendation set.\n");
    }

    println!("═══════════════════════════════════════════════════════════════════");
    println!("IMPLEMENTATION ROADMAP");
    println!("═══════════════════════════════════════════════════════════════════\n");

    println!("Phase 1 (Immediate - Quick Wins):");
    if !set.quick_wins.is_empty() {
        for (idx, qw) in set.quick_wins.iter().take(2).enumerate() {
            println!("  {}. {} (Est. {}% effort)", idx + 1, qw.title, (qw.implementation_effort * 100.0) as i32);
        }
    }

    println!("\nPhase 2 (Medium-Term):");
    if !set.strategic_improvements.is_empty() {
        for (idx, si) in set.strategic_improvements.iter().take(2).enumerate() {
            println!("  {}. {} (Est. {}% effort)", idx + 1, si.title, (si.implementation_effort * 100.0) as i32);
        }
    }

    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("SAMPLE FAILURE-SPECIFIC RECOMMENDATIONS");
    println!("═══════════════════════════════════════════════════════════════════\n");

    // Show different failure types
    let failures = vec![
        ("collision", "sensor_malfunction", "critical"),
        ("coverage_gap", "no_systematic_planning", "medium"),
        ("communication_failure", "network_issue", "high"),
    ];

    for (failure_type, root_cause, severity) in failures {
        println!("For {} failure ({}), {} severity:", failure_type, root_cause, severity);
        let mut temp_engine = RecommendationEngine::new();
        let recs = temp_engine.generate_for_failure(failure_type, root_cause, severity);
        if let Some(first) = recs.first() {
            println!("  → Top recommendation: {}", first.title);
            println!("    Impact: {:.0}% | Effort: {:.0}%\n",
                first.expected_impact * 100.0,
                first.implementation_effort * 100.0);
        }
    }

    println!("═══════════════════════════════════════════════════════════════════");
    println!("CAPABILITIES ENABLED");
    println!("═══════════════════════════════════════════════════════════════════\n");

    println!("💡 Recommendation Engine Capabilities:");
    println!("  ✓ Failure-type-specific recommendations");
    println!("  ✓ Impact/effort/confidence scoring");
    println!("  ✓ Return-on-investment (ROI) calculation");
    println!("  ✓ Quick-win identification (high impact, low effort)");
    println!("  ✓ Strategic improvement planning (medium-term)");
    println!("  ✓ Risk of regression assessment");
    println!("  ✓ Subsystem and fix-type categorization");
    println!("  ✓ Priority-based recommendation ranking");
    println!("  ✓ Actionable implementation guidance");

    println!("\n✨ Phase 5 Task #3 Complete: Recommendation Engine");
}
