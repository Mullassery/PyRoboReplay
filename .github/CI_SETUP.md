# GitHub Actions CI/CD Setup

## Overview

Comprehensive CI/CD pipeline for PyRoboReplay with automated testing, linting, building, security audits, and PyPI publishing.

## Workflows

### 1. **ci.yml** - Main Continuous Integration

Runs on every push to `main`, `master`, `develop` and on all pull requests.

**Matrix Testing:**
- OS: Ubuntu, macOS
- Rust: stable, beta
- Python: 3.10, 3.11, 3.12, 3.13

**Jobs:**
- **Test Suite**: `cargo test --lib --verbose`
  - Runs on Ubuntu + macOS with latest Rust
  - Caches dependencies for faster runs
  - ~2-3 min per matrix combination

- **Lint & Format**: 
  - `cargo fmt -- --check` (Rustfmt validation)
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - Runs on Ubuntu with Rust stable
  - Treats warnings as errors (strict quality gate)

- **Build Wheels**:
  - Builds Python wheels using maturin
  - Matrix: Ubuntu + macOS × Python 3.10-3.13
  - Uploads wheels as GitHub artifacts for inspection
  - Validates packaging works across platforms

- **Publish** (conditional):
  - Only runs on tags matching `refs/tags/v*`
  - Publishes all wheels to PyPI
  - Requires `PYPI_TOKEN` secret configured in GitHub

### 2. **security.yml** - Security & Vulnerability Auditing

Runs on every push, pull request, and weekly on schedule (Sunday 00:00 UTC).

**Jobs:**
- **Cargo Audit**: 
  - Checks for known security vulnerabilities in dependencies
  - Uses official RustSec advisory database
  - Fails if vulnerabilities found (unless whitelisted)

- **Dependency Check**:
  - Identifies outdated/stale dependencies
  - Runs `cargo outdated` for visibility
  - Informational only (doesn't fail)

- **Code Coverage**:
  - Generates code coverage reports using `cargo-tarpaulin`
  - Uploads to Codecov for tracking
  - Timeout: 5 minutes per test
  - Excludes benchmark code

### 3. **release.yml** - Automated Release Pipeline

Triggered automatically when pushing version tags matching pattern `v*` (e.g., `v0.7.0`).

**Jobs:**
- **Create GitHub Release**:
  - Creates a draft release on GitHub with tag info
  - Outputs upload URL for assets

- **Build & Publish**:
  - Builds wheels for Python 3.10-3.13 on Ubuntu & macOS
  - Uploads each wheel as release asset
  - Automatically publishes to PyPI
  - Skips duplicates if re-run

**Usage:**
```bash
git tag v0.8.0
git push origin v0.8.0
# Workflow automatically runs, builds, and publishes
```

### 4. **docs.yml** - Documentation Validation

Runs on every push to main branches and all PRs.

**Jobs:**
- **Documentation Build**:
  - Builds Rust documentation: `cargo doc --no-deps`
  - Includes private items for complete API docs
  - Validates all doc comments compile

- **README Validation**:
  - Checks README.md and CLAUDE.md exist
  - Runs markdownlint for markdown compliance
  - Non-blocking (informational only)

## GitHub Secrets Required

### `PYPI_TOKEN` (Required for PyPI Publishing)

1. Generate token at https://pypi.org/manage/account/tokens/
2. Create token with scope: "Entire account"
3. Copy token value
4. In GitHub repo:
   - Settings → Secrets and variables → Actions
   - New repository secret
   - Name: `PYPI_TOKEN`
   - Value: `pypi-AgEI...` (your token)

**Note:** Token is automatically masked in logs.

## Performance & Caching

**Cargo Caching Strategy:**
- Registry cache: `~/.cargo/registry`
- Index cache: `~/.cargo/git`
- Build cache: `target/` directory

Cache keys use `Cargo.lock` hash to invalidate when dependencies change.

**Build Times (approximate):**
- Clean build: 2-3 min (Ubuntu), 3-4 min (macOS)
- Cached build: 30-60 sec
- Full test suite: 1-2 min
- Linting: 1-2 min

## Status Badges

Add to README.md:

```markdown
![CI Status](https://github.com/mullassery/pyroboreplay/actions/workflows/ci.yml/badge.svg)
![Security Audit](https://github.com/mullassery/pyroboreplay/actions/workflows/security.yml/badge.svg)
```

## Workflow Status & Logs

View workflow runs:
- GitHub repo → Actions tab
- Click workflow name to see runs
- Click run to see detailed logs
- Failed jobs show error messages

## Common Issues & Troubleshooting

### Issue: "Clippy warnings treated as errors"
**Solution:** Fix warnings before pushing. Run locally:
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### Issue: "Format check failed"
**Solution:** Auto-format code:
```bash
cargo fmt
git add .
git commit -m "Format code"
```

### Issue: "Security vulnerability detected"
**Solution:** Update affected dependency:
```bash
cargo update package-name
cargo test
git add Cargo.lock
git commit -m "Update vulnerable dependency"
```

### Issue: "PyPI publishing failed"
**Causes:**
- `PYPI_TOKEN` not configured in secrets
- Token expired or revoked
- Version already published (use `--skip-existing` flag)
- Package metadata invalid

**Debug:** Check workflow logs in Actions tab.

## Extending Workflows

### Add new test matrix:
Edit `ci.yml`, add OS/Rust version to matrix:
```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
    rust: [stable, beta, nightly]
```

### Add custom lint rule:
```yaml
- name: Custom Linter
  run: |
    cargo clippy --all-targets -- -W clippy::pedantic
```

### Deploy documentation:
Add step to release workflow to publish docs to GitHub Pages.

## Best Practices

✅ **DO:**
- Run `cargo test` and `cargo clippy` locally before pushing
- Test on macOS if targeting Apple Silicon
- Keep dependencies updated
- Review security audit results weekly
- Use meaningful commit messages

❌ **DON'T:**
- Force-push to main (breaks workflow artifacts)
- Commit `Cargo.lock` changes carelessly (affects build reproducibility)
- Ignore security warnings
- Use `--skip-existing` to hide publishing errors
- Configure credentials in workflow files (use Secrets)

## References

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust Toolchain GitHub Action](https://github.com/dtolnay/rust-toolchain)
- [Maturin Documentation](https://maturin.rs/)
- [PyPI Token Setup](https://pypi.org/help/#apitoken)
- [RustSec Advisory Database](https://rustsec.org/)
