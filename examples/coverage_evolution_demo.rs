use chrono::Utc;
use pyroboreplay::core::{
    CoverageEvolutionAnalyzer, SpatialContext,
};

fn main() {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║   PyRoboReplay: Coverage Evolution Analysis - Phase 4 Task #3   ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // Create coverage evolution analyzer
    let mut analyzer = CoverageEvolutionAnalyzer::new(0.1, (100.0, 100.0));

    let base_time = Utc::now();

    println!("═══════════════════════════════════════════════════════════════════");
    println!("MISSION COVERAGE TIMELINE");
    println!("═══════════════════════════════════════════════════════════════════\n");

    println!("Simulating exploration mission with coverage evolution:");

    // Phase 1: Initial exploration (0-30 seconds)
    println!("\n[Phase 1] Initial Exploration (0-30s):");
    for i in 0..6 {
        let time = base_time + chrono::Duration::seconds(i as i64 * 5);
        let position = (i as f64 * 2.0, 0.0, 0.0);
        let coverage = (i as f32 * 5.0).min(30.0);

        analyzer.add_snapshot(time, i as usize, position, coverage);

        // Add spatial context
        let context = SpatialContext::new(position, Some((i as f64 * 2.0, 5.0, 0.0)))
            .with_traversability(0.85)
            .with_terrain("open".to_string());
        analyzer.add_spatial_context(i as usize, context);

        println!("  Event {}: Position ({:.1}, 0.0) | Coverage: {:.0}%", i, position.0, coverage);
    }

    // Phase 2: Expansion (30-60 seconds)
    println!("\n[Phase 2] Lateral Expansion (30-60s):");
    for i in 6..12 {
        let time = base_time + chrono::Duration::seconds(i as i64 * 5);
        let position = (10.0, (i as f64 - 6.0) * 2.0, 0.0);
        let coverage = (30.0 + (i as f32 - 6.0) * 5.0).min(60.0);

        analyzer.add_snapshot(time, i as usize, position, coverage);

        let context = SpatialContext::new(position, Some((10.0, (i as f64 - 6.0) * 2.0, 3.0)))
            .with_traversability(0.75)
            .with_terrain("confined".to_string());
        analyzer.add_spatial_context(i as usize, context);

        println!(
            "  Event {}: Position (10.0, {:.1}) | Coverage: {:.0}%",
            i,
            position.1,
            coverage
        );
    }

    // Phase 3: Refinement (60-90 seconds)
    println!("\n[Phase 3] Coverage Refinement (60-90s):");
    for i in 12..18 {
        let time = base_time + chrono::Duration::seconds(i as i64 * 5);
        let position = (5.0 + (i as f64 - 12.0) * 0.8, 5.0, 0.0);
        let coverage = (60.0 + (i as f32 - 12.0) * 3.0).min(85.0);

        analyzer.add_snapshot(time, i as usize, position, coverage);

        let context = SpatialContext::new(position, None)
            .with_traversability(0.95)
            .with_terrain("open".to_string());
        analyzer.add_spatial_context(i as usize, context);

        println!(
            "  Event {}: Position ({:.1}, 5.0) | Coverage: {:.0}%",
            i,
            position.0,
            coverage
        );
    }

    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("COVERAGE STATISTICS");
    println!("═══════════════════════════════════════════════════════════════════\n");

    // Get coverage timeline
    let timeline = analyzer.coverage_timeline();
    println!("Coverage Timeline:");
    println!("  Start: {:.0}%", timeline.first().map(|t| t.1).unwrap_or(0.0));
    println!("  End: {:.0}%", timeline.last().map(|t| t.1).unwrap_or(0.0));
    println!("  Growth rate: {:.2}%/s", analyzer.growth_rate());

    // Analyze full coverage evolution
    let query = analyzer.analyze();

    println!("\nCoverage Evolution Statistics:");
    println!("  Initial coverage: {:.1}%", query.stats.initial_coverage);
    println!("  Final coverage: {:.1}%", query.stats.final_coverage);
    println!("  Coverage gained: {:.1}%", query.stats.coverage_gained);
    if let Some(time) = query.stats.time_to_half_coverage {
        println!("  Time to 50%: {}ms", time);
    }
    if let Some(time) = query.stats.time_to_full_coverage {
        println!("  Time to 95%: {}ms", time);
    }
    println!("  Avg growth rate: {:.3}%/s", query.stats.avg_growth_rate);
    println!(
        "  Expansion efficiency: {:.3} coverage/%distance",
        query.stats.expansion_efficiency
    );

    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("HOTSPOT ANALYSIS");
    println!("═══════════════════════════════════════════════════════════════════\n");

    if !query.hotspots.is_empty() {
        println!("Coverage Activity Hotspots:");
        for (idx, hotspot) in query.hotspots.iter().enumerate() {
            println!(
                "\n  Hotspot {}: {}",
                idx,
                hotspot.id
            );
            println!(
                "    Location: ({:.1}, {:.1}, {:.1})",
                hotspot.center.0, hotspot.center.1, hotspot.center.2
            );
            println!("    Radius: {:.1}m", hotspot.radius_m);
            println!("    Events: {}", hotspot.event_count);
            println!("    Causal links: {}", hotspot.causal_links);
            println!("    Avg coverage: {:.0}%", hotspot.avg_coverage * 100.0);
            println!(
                "    Traversability impact: {:.0}%",
                hotspot.traversability_impact * 100.0
            );
        }
    } else {
        println!("No significant hotspots detected.");
    }

    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("COVERAGE GAP ANALYSIS");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let gaps = analyzer.identify_gaps(1.0);
    if !gaps.is_empty() {
        println!("Total gaps identified: {}", gaps.len());
        for gap in gaps.iter().take(5) {
            println!("\n  Gap: {}", gap.id);
            println!("    Center: ({:.1}, {:.1})", gap.center.0, gap.center.1);
            println!("    Radius: {:.1}m", gap.radius_m);
            println!("    Area: {:.1}m²", gap.area_m2);
            println!("    Reason: {}", gap.reason);
            println!("    Importance: {}", gap.importance);
            println!(
                "    Status: {}",
                if gap.was_filled { "FILLED" } else { "OPEN" }
            );
        }
    } else {
        println!("No significant gaps detected (>1.0m²)");
    }

    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("COVERAGE INSIGHTS");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let phases = vec![
        ("Exploration", 0usize, 6usize),
        ("Expansion", 6usize, 12usize),
        ("Refinement", 12usize, 18usize),
    ];

    for (name, start_idx, end_idx) in phases {
        if let (Some(start_snap), Some(end_snap)) =
            (timeline.get(start_idx), timeline.get(end_idx.saturating_sub(1)))
        {
            let phase_gain = end_snap.1 - start_snap.1;
            println!("{} Phase:", name);
            println!("  Coverage gain: {:.0}%", phase_gain);
            println!("  Events: {}", end_idx - start_idx);
            if end_idx > start_idx {
                println!("  Avg gain per event: {:.2}%", phase_gain / (end_idx - start_idx) as f32);
            }
        }
    }

    println!("\n💡 Coverage Analysis Capabilities Enabled:");
    println!("  ✓ Timeline-based coverage tracking");
    println!("  ✓ Growth rate and efficiency metrics");
    println!("  ✓ Hotspot detection (high-activity areas)");
    println!("  ✓ Gap identification and classification");
    println!("  ✓ Phase-based analysis");
    println!("  ✓ Spatial-causal correlation");

    println!("\n✨ Phase 4 Task #3 Complete: Spatial Coverage Evolution");
}
