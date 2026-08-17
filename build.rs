// Build script that exists solely to make the `cdylib` crate-type (added so
// `maturin` can produce a real, importable `pyroboreplay` Python extension
// module — see the comment on `[lib]` in Cargo.toml) link cleanly on macOS
// under a plain `cargo build`/`cargo test`, not just under `maturin`.
//
// PyO3's `extension-module` feature intentionally does not link the cdylib
// against `libpython` (the symbols are resolved at runtime by the Python
// interpreter that dlopens the extension instead). On macOS that requires
// the linker be told `-undefined dynamic_lookup` so it doesn't error out on
// the resulting undefined `Py_*`/`PyO3` symbols at link time. `maturin`
// supplies this itself via its own build environment, which is why
// `maturin build`/`maturin develop` already worked without this file. Plain
// `cargo build --lib`/`cargo test --lib` do not go through maturin, so
// without this, adding `cdylib` to `[lib] crate-type` breaks those commands
// on macOS with "symbol(s) not found" linker errors (verified locally).
//
// `cargo:rustc-cdylib-link-arg` is scoped to cdylib artifacts only (per the
// Cargo book), so this has no effect on the `pyroboreplay` CLI binary
// (`src/main.rs`, a normal `bin` target) or on `cargo test`'s own harness
// linking — only the cdylib PyO3 extension module picks up this flag.
fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "macos" {
        println!("cargo:rustc-cdylib-link-arg=-undefined");
        println!("cargo:rustc-cdylib-link-arg=dynamic_lookup");
    }
}
