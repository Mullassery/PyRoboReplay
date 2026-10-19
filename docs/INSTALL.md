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

## Platform matrix

### Published on PyPI (2.11.0)

Verified 2026-10-20. These are the only wheels that currently exist.

| Platform | Python 3.10 | 3.11 | 3.12 | 3.13 |
|---|---|---|---|---|
| Linux x86_64 (manylinux_2_38) | yes | yes | yes | yes |
| macOS arm64 (11.0+) | yes | yes | yes | yes |
| Linux aarch64 | — | — | — | — |
| macOS x86_64 (Intel) | — | — | — | — |
| Windows x86_64 | — | — | — | — |

### Next release (configured, not yet published)

The release workflow builds the targets below for every Python 3.10–3.13.
They do **not** exist on PyPI until a new `v*` tag is pushed and the release
completes. Before publishing, the workflow installs each wheel on a matching
native runner (not editable) and runs `import pyroboreplay`,
`from pyroboreplay import Mission`, and `ops()`.

| Target | Build | Status |
|---|---|---|
| Linux x86_64 (manylinux) | `maturin-action` (manylinux container) | configured |
| Linux aarch64 (manylinux) | `maturin-action` cross build (manylinux) | configured |
| macOS arm64 (11.0+) | `maturin-action` (native) | configured |
| macOS x86_64 (Intel) | `maturin-action` cross build | configured |
| Windows x86_64 | `maturin-action` (native) | configured |

No wheel for your platform? Build from source.

## Troubleshooting

- **`ModuleNotFoundError: numpy` on 2.11.0** — `pip install numpy` (fixed in
  the next release).
- **`pyroboreplay: command not found` on 2.11.0** — expected; the wheel ships
  no CLI. Build from source or upgrade once the next release is published.
- **`ModuleNotFoundError: pyroboreplay._core`** — the compiled extension is
  missing or built for another Python/architecture. Reinstall the matching
  wheel or build from source.
