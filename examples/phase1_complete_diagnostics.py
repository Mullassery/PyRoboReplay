#!/usr/bin/env python3
"""
PyRoboReplay Phase 1: Complete Failure Diagnosis Pipeline

This example demonstrates the full diagnostic workflow:
1. Detect failures across 8 categories
2. Analyze root causes with ranked hypotheses
3. Generate human-readable explanations
4. Get prioritized recommended actions

Run with:
    python examples/phase1_complete_diagnostics.py <mission.bag>
"""

import sys
from pathlib import Path

try:
    from pyroboreplay import Mission
except ImportError:
    print("Error: PyRoboReplay not installed. Install with: pip install pyroboreplay")
    sys.exit(1)


def print_section(title):
    """Print a formatted section header."""
    print(f"\n{'='*70}")
    print(f"  {title}")
    print(f"{'='*70}\n")


def print_failure_summary(failure):
    """Print a brief summary of a failure."""
    severity_icon = {
        "critical": "🔴",
        "high": "🟠",
        "medium": "🟡",
        "low": "🟢",
    }.get(failure.get_severity(), "⚪")

    print(f"{severity_icon} {failure.get_failure_type().upper()}")
    print(f"   Timestamp: {failure.get_timestamp():.2f}s")
    print(f"   Severity: {failure.get_severity()}")
    print(f"   Confidence: {failure.get_confidence():.0%}")


def diagnose_mission(bag_path: str):
    """Run complete diagnosis on a ROS 2 bag file."""
    print_section("PyRoboReplay Phase 1: Complete Diagnostics")

    # Load mission
    print(f"Loading mission from: {bag_path}")
    try:
        mission = Mission.from_ros_bag(bag_path)
    except Exception as e:
        print(f"Error loading mission: {e}")
        return

    print(f"✓ Mission loaded")
    print(f"  Name: {mission.name()}")
    print(f"  Duration: {mission.duration_seconds()} seconds")
    print(f"  Total events: {mission.event_count()}")
    print(f"  Sensors: {', '.join(mission.get_available_sensors())}")

    # Detect failures
    print_section("Step 1: Detect Failures")
    print("Scanning for anomalies across 8 categories...")
    print("  • Near collision (LiDAR)")
    print("  • Perception failure (low detection confidence)")
    print("  • Sensor dropout (message gap)")
    print("  • Communication loss (synchronization)")
    print("  • Navigation deadlock (replanning without progress)")
    print("  • Localization loss (low pose confidence)")
    print("  • Oscillation (back-and-forth movement)")
    print("  • Costmap anomaly (sudden map changes)")

    failures = mission.detect_failures()

    if not failures:
        print("\n✓ No failures detected! Mission completed successfully.")
        return

    print(f"\n✓ Found {len(failures)} issue(s):\n")
    for i, failure in enumerate(failures, 1):
        print(f"{i}. ", end="")
        print_failure_summary(failure)
        print()

    # Analyze each failure
    print_section("Step 2: Analyze Root Causes")
    print("Building causal graphs and generating hypotheses...\n")

    for i, failure in enumerate(failures, 1):
        print(f"\n{'─'*70}")
        print(f"Issue {i}: {failure.get_failure_type().upper()}")
        print(f"{'─'*70}\n")

        try:
            analysis = mission.analyze_failure(failure.get_timestamp())

            print("ROOT CAUSE ANALYSIS")
            print(f"  Primary hypothesis: {analysis.get_primary_hypothesis()}")
            print(f"  Diagnostic confidence: {analysis.get_diagnostic_confidence():.0%}\n")

            hypotheses = analysis.get_hypotheses()
            if hypotheses:
                print("Ranked hypotheses:")
                for j, hyp in enumerate(hypotheses, 1):
                    print(f"  {j}. {hyp.get_description()}")
                    print(f"     Confidence: {hyp.get_confidence():.0%}")
                    if hyp.get_causal_chain():
                        print(f"     Causal chain: {' → '.join(hyp.get_causal_chain())}")
                    print()
        except Exception as e:
            print(f"  [Could not analyze root cause: {e}]\n")

        # Generate explanation
        print("WHAT HAPPENED")
        try:
            explanation = mission.explain_failure(failure.get_timestamp())
            # Print explanation with word wrapping
            words = explanation.split()
            line = ""
            for word in words:
                if len(line) + len(word) + 1 > 70:
                    print(f"  {line}")
                    line = word
                else:
                    line = f"{line} {word}" if line else word
            if line:
                print(f"  {line}")
            print()
        except Exception as e:
            print(f"  [Could not generate explanation: {e}]\n")

        # Get recommended actions
        print("RECOMMENDED ACTIONS")
        try:
            actions = mission.recommend_actions(failure.get_timestamp())

            if not actions:
                print("  [No recommendations available]\n")
            else:
                for action in actions:
                    priority_icon = {
                        "P0": "🔴",
                        "P1": "🟠",
                        "P2": "🟡",
                    }.get(action.get_priority(), "⚪")

                    print(f"\n  {priority_icon} [{action.get_priority()}] {action.get_description()}")
                    print(f"     Impact: {action.get_impact()} | Complexity: {action.get_complexity()}")
                    print(f"\n     Implementation:")
                    for line in action.get_implementation().split(". "):
                        if line.strip():
                            print(f"       • {line.strip()}.")
                print()
        except Exception as e:
            print(f"  [Could not generate recommendations: {e}]\n")

    # Summary
    print_section("Diagnostic Summary")
    print(f"Total issues found: {len(failures)}")

    severity_counts = {}
    for failure in failures:
        severity = failure.get_severity()
        severity_counts[severity] = severity_counts.get(severity, 0) + 1

    if severity_counts:
        print("\nIssues by severity:")
        for severity in ["critical", "high", "medium", "low"]:
            if severity in severity_counts:
                icon = {
                    "critical": "🔴",
                    "high": "🟠",
                    "medium": "🟡",
                    "low": "🟢",
                }.get(severity, "⚪")
                print(f"  {icon} {severity}: {severity_counts[severity]}")

    failure_types = {}
    for failure in failures:
        ftype = failure.get_failure_type()
        failure_types[ftype] = failure_types.get(ftype, 0) + 1

    if failure_types:
        print("\nIssues by type:")
        for ftype, count in sorted(failure_types.items()):
            print(f"  • {ftype}: {count}")

    print("\n✓ Diagnostic complete!")
    print(f"\nNext steps:")
    print("  1. Review the recommended actions above")
    print("  2. Prioritize P0 actions for immediate deployment")
    print("  3. Test fixes in simulator before deploying to fleet")
    print("  4. Re-run diagnostics after fixes to verify improvement")


def main():
    """Main entry point."""
    if len(sys.argv) != 2:
        print("Usage: python phase1_complete_diagnostics.py <mission.bag>")
        print("\nExample:")
        print("  python examples/phase1_complete_diagnostics.py warehouse_run.bag")
        sys.exit(1)

    bag_path = sys.argv[1]

    if not Path(bag_path).exists():
        print(f"Error: File not found: {bag_path}")
        sys.exit(1)

    diagnose_mission(bag_path)


if __name__ == "__main__":
    main()
