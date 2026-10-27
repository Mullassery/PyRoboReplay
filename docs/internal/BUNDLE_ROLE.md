# Bundle role — Robotics data & fleet loop (deferred)

Portfolio-level decision: **deferred**. See
[`Mullassery/BUNDLES.md`](../Mullassery/blob/main/BUNDLES.md).

- **Role:** Mission replay and forensic RCA
- **Upstream/downstream:** PyRoboFrames, PyTerrainMap, SyncFleet
- **Unlock condition:** Expose phases 12–15 through the public API and export to rerun/MCAP; make PyTerrainMap interop real.
- **Current status:** standalone; not a dependency of the other bundle members.

Do not add cross-repo runtime dependencies until the unlock condition is met.
When it is, add a versioned interface (not a git URL), a shared
observation/mission schema, and an end-to-end demo.
