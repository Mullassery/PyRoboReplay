# Contributing to PyRoboReplay

Thanks for your interest in contributing! We welcome contributions of all kinds.

## How to Contribute

### Report Bugs 🐛

Found a bug? [Open an issue](https://github.com/Mullassery/pyroboreplay/issues) with:
- Clear title: "Bug: [component] description"
- Steps to reproduce
- Expected vs actual behavior
- Environment (OS, Rust version, Python version)
- Minimal reproducible example if possible

### Suggest Features 💡

Have an idea? Start a [discussion](https://github.com/Mullassery/pyroboreplay/discussions) or [issue](https://github.com/Mullassery/pyroboreplay/issues) with:
- Use case: why would this be useful?
- Proposed solution (if you have one)
- Alternative approaches you've considered
- Links to relevant research or prior art

### Submit Code 💻

#### Setup

```bash
# Clone repo
git clone https://github.com/Mullassery/pyroboreplay.git
cd pyroboreplay

# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Python dev dependencies
pip install maturin pytest

# Build locally
cargo build --release
maturin develop
```

#### Workflow

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Run tests: `cargo test --lib`
5. Run lints: `cargo clippy --all-targets -- -D warnings && cargo fmt --check`
6. Commit with clear messages (see [Commit Style](#commit-style))
7. Push to your fork
8. Open a PR with a clear description

#### Commit Style

Follow conventional commits:

```
type(scope): description

Detailed explanation if needed.

Fixes #123
```

**Types:** feat, fix, docs, style, refactor, test, chore, perf

**Examples:**
- `feat(fleet-monitor): add health trend detection`
- `fix(sla): correct compliance score calculation`
- `docs(readme): update v0.8.0 features`

#### Code Style

**Rust:**
```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
```

**Python:**
```bash
pip install black isort
black .
isort .
```

#### Tests

All code must have tests:
- Unit tests: co-located in same file, `#[cfg(test)]` module
- Integration tests: in `tests/` directory (if added)
- Run: `cargo test --lib`
- Aim for >90% coverage on new code

**Example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_feature() {
        let result = my_feature(42);
        assert_eq!(result, 42);
    }
}
```

#### Documentation

Add docs for public APIs:

```rust
/// Brief one-line description.
///
/// Longer explanation of what this does, when to use it,
/// and any important caveats.
///
/// # Examples
///
/// ```
/// let result = my_function(42);
/// assert_eq!(result, 42);
/// ```
pub fn my_function(x: i32) -> i32 {
    x
}
```

#### PR Guidelines

- **Title:** Clear, descriptive, start with type (feat/fix/docs)
- **Description:** Explain the why, not just the what
- **Tests:** Include tests (new code should have >90% coverage)
- **Docs:** Update README/docs if user-facing
- **Breaking changes:** Clearly mark and explain
- **Related issues:** Link with "Fixes #123" if applicable

**Good PR description template:**

```markdown
## Description
Brief summary of changes.

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Added tests
- [ ] All tests passing (`cargo test --lib`)
- [ ] Linting passing (`cargo clippy && cargo fmt`)

## Related Issues
Fixes #123

## Performance Impact
None / Describe any perf implications
```

---

## Areas We're Looking For Help

See [ROADMAP_HONEST.md](ROADMAP_HONEST.md) for the current, code-verified list of
what's built, what's partial, and what's missing. As of this writing, the
highest-leverage gap is CLI/Python exposure for Phases 12-20 (the analysis
logic already exists in the Rust core with unit tests; it just isn't wired up
to `src/cli/` or `src/pyroboreplay/__init__.py` yet). Note: Postgres, S3, and
BigQuery storage backends already exist with dedicated integration test
suites — that work is done, not needed.

### Good First Issues

### Good First Issues
- Look for [good-first-issue](https://github.com/Mullassery/pyroboreplay/labels/good-first-issue) label
- Start with documentation or small bug fixes
- Ask questions in [discussions](https://github.com/Mullassery/pyroboreplay/discussions) if stuck

---

## Development Tips

### Running Examples

```bash
cargo run --example fleet_monitor_demo
cargo run --example compliance_report_demo
cargo run --example deterministic_replay_demo
```

### Debugging

```bash
# Print debug info
RUST_LOG=debug cargo run --example my_example

# Run single test with output
cargo test --lib test_my_feature -- --nocapture

# Use rust-gdb for debugging
rust-gdb --args target/debug/my_binary
```

### Git Workflow

```bash
# Create feature branch
git checkout -b feat/my-feature

# Keep updated with master (the default branch)
git fetch origin
git rebase origin/master

# Before pushing, squash if needed
git rebase -i origin/master

# Push and open PR
git push origin feat/my-feature
```

### CI/CD

Workflows run automatically on push/PR:
- **ci.yml**: Tests, linting, building
- **security.yml**: Vulnerability scanning, coverage
- **release.yml**: Auto-publish on version tags

Check status in [Actions](https://github.com/Mullassery/pyroboreplay/actions).

---

## Architecture & Design

- **[docs/CLAUDE.md](docs/CLAUDE.md)** — Product vision, principles
- **[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)** — System architecture
- **[.github/CI_SETUP.md](.github/CI_SETUP.md)** — CI/CD architecture
- **[ROADMAP_HONEST.md](ROADMAP_HONEST.md)** — What's actually built, partial, or missing (no fabricated timelines)

---

## Code of Conduct

- **Be respectful:** Treat everyone with kindness
- **Be inclusive:** Welcome people of all backgrounds
- **Be constructive:** Aim to help, not harm
- **Be honest:** Acknowledge mistakes, give credit

[Full Code of Conduct](CODE_OF_CONDUCT.md)

---

## Getting Help

- **Questions?** Ask in [GitHub Discussions](https://github.com/Mullassery/pyroboreplay/discussions)
- **Stuck?** Open an issue and label it `help-wanted`
- **Chat?** Connect via GitHub issues/discussions. This is currently maintained by one person in their spare time — there is no guaranteed response time.

---

## Recognition

Contributors are recognized in:
- README.md (major contributions)
- Release notes (all contributions)
- GitHub contributors page

Thank you for making PyRoboReplay better! 🚀

---

**Happy contributing! ⭐**
