"""Command-line exporter for MCAP and Rerun recordings.

Usage::

    pyroboreplay-export mission.db3 mission.mcap --format mcap
    pyroboreplay-export mission.db3 mission.rrd --format rerun

The native ``pyroboreplay export`` command handles MCAP without a Python
runtime; this script adds the optional Rerun path.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path
from typing import Optional, Sequence


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Export a recorded mission")
    parser.add_argument("bag_file", help="ROS 2 bag (.db3 or .mcap)")
    parser.add_argument("output", help="Output path")
    parser.add_argument(
        "--format",
        choices=("mcap", "rerun"),
        default="mcap",
        help="Export format (default: mcap)",
    )
    return parser


def main(argv: Optional[Sequence[str]] = None) -> int:
    args = build_parser().parse_args(argv)

    from . import Mission, export_mcap, export_mission

    mission = Mission.from_ros_bag(args.bag_file)
    output = Path(args.output)

    if args.format == "mcap":
        export_mcap(mission, output)
    else:
        export_mission(mission, output)
    print(f"Exported {args.bag_file} -> {output} ({args.format})")
    return 0


if __name__ == "__main__":  # pragma: no cover
    sys.exit(main())
