# Install

PyRoboReplay has two entry points: a **Python API** and a **CLI**. They are
packaged together from source; the published 2.11.0 wheel has a known
packaging gap.

> **PyPI 2.11.0 packaging (verified 2026-10-19):** the published 2.11.0 wheel
> ships the Python package but **no console scripts**, and it does not declare
> `numpy`, so `import pyroboreplay` fails without a manual `pip install numpy`.
> Both issues are fixed in the repository (a `pyroboreplay` entry point backed
> by the Rust CLI, and a `numpy` dependency) and will ship in the next release.
> Until then, use the workaround below or build from source.

## Python API

**2.11.0 workaround** (manual dependency):

```bash
pip install pyroboreplay numpy
```

Verify:

```bash
python -c "import pyroboreplay; print(pyroboreplay.__version__)"
```

```python
from pyroboreplay import Mission

mission = Mission.from_ros_bag("mission.db3")
print(mission.event_count())
```

Requires Python 3.10+.

### Optional extras

```bash
pip install "pyroboreplay[mcap]"    # Python MCAP export helper
pip install "pyroboreplay[rerun]"   # Python Rerun export helper
```

## CLI

**From the next release** (source is fixed and verified):

```bash
pip install pyroboreplay
pyroboreplay --version
```

**From source today** (works with this repository):

```bash
git clone https://github.com/Mullassery/PyRoboReplay.git
cd pyroboreplay
python -m venv .venv && source .venv/bin/activate
pip install maturin
maturin develop --release
pyroboreplay --help
```

Or run the Rust binary directly:

```bash
cargo run --release -- analyze mission.db3 --detect-gaps
```

Requirements: Python 3.10+, a stable Rust toolchain, and the usual Rust build
prerequisites.

`pyroboreplay-export` uses the same packaging and is available from a source
install / the next release.

## Platform matrix (2.11.0)

| Platform | Python 3.10 | 3.11 | 3.12 | 3.13 |
|---|---|---|---|---|
| Linux x86_64 (manylinux_2_38) | ✅ | ✅ | ✅ | ✅ |
| macOS arm64 (11.0+) | ✅ | ✅ | ✅ | ✅ |
| Windows | — | — | — | — |
| macOS Intel | — | — | — | — |

No wheel for your platform? Build from source.

## Troubleshooting

- **`ModuleNotFoundError: numpy` on 2.11.0** — `pip install numpy` (fixed in
  the next release).
- **`pyroboreplay: command not found` on 2.11.0** — expected; the wheel ships
  no CLI. Build from source or upgrade once the next release is published.
- **`ModuleNotFoundError: pyroboreplay._core`** — the compiled extension is
  missing or built for another Python/architecture. Reinstall the matching
  wheel or build from source.
