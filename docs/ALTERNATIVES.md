# Alternatives

PyRoboReplay is **offline mission forensics**: it explains what happened in a
recorded robot mission. It is not a live fleet dashboard and not a general log
viewer. The tools below are complementary.

Sources verified on **2026-10-19**:

- Foxglove — <https://docs.foxglove.dev/>
- Rerun — <https://rerun.io/>

This is not a benchmark; features change. Re-check before relying on a row.

## Pick by problem

| Your problem | Start with | Why |
|---|---|---|
| "Why did this recorded mission fail?" | **PyRoboReplay** | Failure detection, hypotheses, prioritized actions |
| "What did perception miss?" | **PyRoboReplay** | Detection-gap analysis (external DINO detections) |
| "Compare two mission revisions" | **PyRoboReplay** or Foxglove | CLI diff vs visual comparison |
| "Inspect 3D/sensor data visually" | Foxglove or Rerun | Mature viewers, layouts, panels |
| "Search/organize a library of recordings" | Foxglove Data Platform or Rerun Hub | Hosted indexing/search |
| "Log custom multimodal data from my stack" | Rerun or Foxglove SDK | Python/C++/Rust logging SDKs |
| "Monitor robots live" | Foxglove / fleet platforms | PyRoboReplay is offline-only |

## Comparison

| Capability | PyRoboReplay | Foxglove | Rerun |
|---|---|---|---|
| Replay ROS 2 bags | ✅ | ✅ | via converters |
| Visual 3D/sensor inspection | basic interactive replay | ✅ panels/layouts | ✅ viewer |
| Automated failure detection | ✅ | — | — |
| Root-cause hypotheses + actions | ✅ | — (human triage) | — |
| Detection-gap analysis | ✅ | — | — |
| RGB + thermal fusion | ✅ | visualization | visualization |
| Nav2 root-cause heuristics | ✅ | — | — |
| Mission comparison | ✅ CLI | ✅ visual comparison | — |
| MCAP export | ✅ | ✅ native | — |
| Rerun export | ✅ | — | native |
| Hosted search / data platform | — | ✅ | ✅ (Hub) |
| Offline, self-contained | ✅ | local app, optional platform | ✅ SDK/viewer |
| License | Apache-2.0 | commercial platform | open-source SDK + commercial Hub |

Legend: ✅ stated in the reviewed official docs; — not stated (not proof of
absence).

## How they fit together

- Use PyRoboReplay to **analyze** a mission and export MCAP
  ([README](../README.md#mcap-export)) or Rerun
  ([README](../README.md#rerun-mission-playback)).
- Open the export in Foxglove or Rerun for **visual** review with your team.
- Use Foxglove/Rerun Hub or a fleet platform when you need **live** monitoring
  or a searchable library.

## What PyRoboReplay does not do

- Live/fleet monitoring (offline recorded missions only).
- General-purpose visualization (use a viewer).
- Hosted storage/search.
- Phases 16–20 are reachable only through a JSON operation bridge (six
  allowlisted operations), not typed Python APIs. See
  [Advanced operations](ADVANCED_OPS.md),
  [Project status](STATUS.md), and
  [Limitations in the README](../README.md#what-it-is-not).
