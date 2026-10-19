"""Console entry point for the PyRoboReplay CLI.

The command surface lives in the Rust core (``src/cli/mod.rs``). This thin
wrapper forwards Python argv to the compiled ``run_cli`` function so that
``pip install pyroboreplay`` provides the same ``pyroboreplay`` command as the
Rust binary.
"""

from __future__ import annotations

import sys


def main() -> None:
    from pyroboreplay._core import run_cli

    raise SystemExit(run_cli(sys.argv[1:]))


if __name__ == "__main__":
    main()
