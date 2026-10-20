# Install

PyRoboReplay has two entry points: a **Python API** and a **CLI**, packaged
together. The 2.12.x wheel ships both console scripts and declares `numpy`.

## Install

```bash
pip install pyroboreplay
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

`pyroboreplay-export` is included in the same wheel and sdist.

## Platform matrix

### Published on PyPI (2.12.1)

Verified 2026-10-20. PyPI 2.12.1 ships a macOS arm64 wheel and an sdist.

| Platform | Availability |
|---|---|
| macOS arm64 (11.0+) | wheel |
| Linux x86_64 / aarch64 | sdist (build from source) |
| macOS x86_64 (Intel) | sdist (build from source) |
| Windows x86_64 | sdist (build from source) |

### Native wheel matrix (configured in the release workflow)

The release workflow builds the targets below for every Python 3.10–3.13 and
installs each wheel on a matching native runner (not editable), running
`import pyroboreplay`, `from pyroboreplay import Mission`, and `ops()`.

| Target | Build | Status |
|---|---|---|
| Linux x86_64 (manylinux) | `maturin-action` (manylinux container) | configured |
| Linux aarch64 (manylinux) | `maturin-action` cross build (manylinux) | configured |
| macOS arm64 (11.0+) | `maturin-action` (native) | configured |
| macOS x86_64 (Intel) | `maturin-action` cross build | configured |
| Windows x86_64 | `maturin-action` (native) | configured |

No wheel for your platform? Build from source.

## Troubleshooting

- **`ModuleNotFoundError: pyroboreplay._core`** — the compiled extension is
  missing or built for another Python/architecture. Reinstall the matching
  wheel or build from source.
