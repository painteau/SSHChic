# GitHub Workflows Documentation

This document describes the CI/CD workflows used in SSHChic for automated testing, building, and releasing.

## Table of Contents

- [Overview](#overview)
- [Workflows](#workflows)
- [Workflow Details](#workflow-details)
- [Usage Guide](#usage-guide)
- [Troubleshooting](#troubleshooting)
- [Maintenance](#maintenance)

## Overview

SSHChic uses three main GitHub Actions workflows:

1. **Build & Test** (`build-rust.yml`) - Continuous Integration
2. **Create Release** (`create-release-tag.yaml`) - Automated versioning and tagging
3. **Build Release Artifacts** (`release-artefacts.yaml`) - Multi-platform binary builds

All workflows use modern, maintained GitHub Actions and follow best practices for caching, security, and reliability.

## Workflows

### 1. Build & Test (CI)

**File:** `.github/workflows/build-rust.yml`

**Purpose:** Continuous integration testing on every push and pull request

**Triggers:**
- Push to main branch (when Rust files or Cargo.toml/Cargo.lock change)
- Pull requests to main
- Manual dispatch

**What it does:**
- Runs on 3 operating systems (Ubuntu, Windows, macOS)
- Tests both debug and release builds (6 matrix combinations)
- Checks code formatting with `rustfmt`
- Runs linting with `clippy` (warnings treated as errors)
- Executes unit tests
- Builds the project
- Runs functional tests (release mode only)
- Uploads artifacts for 5 days

**Duration:** ~5-10 minutes per OS/mode combination

**Status Badge:**
```markdown
[![Rust Build](https://github.com/painteau/SSHChic/actions/workflows/build-rust.yml/badge.svg)](https://github.com/painteau/SSHChic/actions/workflows/build-rust.yml)
```

### 2. Create Release Tag

**File:** `.github/workflows/create-release-tag.yaml`

**Purpose:** Automatically create GitHub releases when code changes

**Triggers:**
- Push to main (when src/, Cargo.toml, or Cargo.lock change)
- Manual dispatch with version bump choice

**What it does:**
1. **Pre-release checks:**
   - Analyzes commit message for skip indicators
   - Filters out dependency updates
   - Runs full quality checks (format, lint, tests)
   - Validates Cargo.toml version format
   - Verifies tag doesn't already exist

2. **Release creation:**
   - Reads version from Cargo.toml
   - Generates release notes from git commits
   - Creates GitHub release with installation instructions
   - Tags the commit with version number

**Skip release:** Add `[skip-release]` or `[no-release]` to commit message

**Duration:** ~3-5 minutes

### 3. Build Release Artifacts

**File:** `.github/workflows/release-artefacts.yaml`

**Purpose:** Build binaries for all supported platforms and upload to release

**Triggers:**
- When a GitHub release is published
- Manual dispatch with tag input

**What it does:**
1. **Quality checks** (runs first):
   - Format, lint, and test validation
   - Ensures code quality before building

2. **Multi-platform builds:**
   - Builds for 8 platforms in parallel
   - Uses cross-compilation for Linux ARM targets
   - Native builds for macOS and Windows
   - Creates compressed archives (.tar.gz for Unix, .zip for Windows)
   - Generates SHA256 checksums for verification

3. **Release upload:**
   - Uploads binaries, checksums, LICENSE, and README
   - Adds all artifacts to the GitHub release

**Supported platforms:**
- Linux: i686, x86_64, ARM, ARM64
- macOS: x86_64 (Intel), ARM64 (Apple Silicon)
- Windows: i686, x86_64

**Duration:** ~20-40 minutes (parallel builds)

## Workflow Details

### Modern Actions Used

All workflows use maintained, modern GitHub Actions:

| Action | Version | Purpose | Replacement For |
|--------|---------|---------|-----------------|
| `actions/checkout` | v4 | Check out repository | v3 |
| `actions/cache` | v4 | Cache Cargo dependencies | v3 |
| `actions/upload-artifact` | v4 | Upload build artifacts | v3 |
| `dtolnay/rust-toolchain` | stable | Install Rust toolchain | `actions-rs/toolchain` (deprecated) |
| `softprops/action-gh-release` | v2 | Create/update releases | v1 |

**Note:** `actions-rs/*` actions are archived and no longer maintained. We've migrated away from them.

### Caching Strategy

All workflows use Cargo caching to speed up builds:

```yaml
- uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry  # Crate registry
      ~/.cargo/git       # Git dependencies
      target             # Build artifacts
    key: ${{ runner.os }}-cargo-${{ matrix.build-mode }}-${{ hashFiles('**/Cargo.lock') }}
```

**Cache invalidation:** When Cargo.lock changes, a new cache is created.

**Benefits:**
- Faster CI runs (dependencies downloaded once)
- Reduced bandwidth usage
- Better GitHub Actions quota utilization

### Cross-Compilation

For ARM Linux targets, we use `cross`:

```bash
cargo install cross --git https://github.com/cross-rs/cross
cross build --release --target arm-unknown-linux-gnueabihf
```

**Why cross?**
- Simplifies cross-compilation setup
- Uses Docker containers with proper toolchains
- Handles complex dependencies (OpenSSL, etc.)
- More reliable than manual toolchain setup

### Quality Checks

Multiple levels of quality assurance:

1. **Format check:**
   ```bash
   cargo fmt -- --check
   ```
   Ensures code follows Rust style guidelines

2. **Clippy linting:**
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   ```
   Catches common mistakes and anti-patterns

3. **Unit tests:**
   ```bash
   cargo test --verbose
   ```
   Runs all test suites

4. **Functional tests:**
   ```bash
   cargo run -- --help
   cargo run -- --regex "TEST$"
   ```
   Tests actual binary functionality

## Usage Guide

### For Contributors

#### Before Committing

Always run local checks before pushing:

```bash
# Format code
cargo fmt

# Check linting
cargo clippy -- -D warnings

# Run tests
cargo test

# Build release
cargo build --release
```

#### Creating a Pull Request

1. Make your changes in a feature branch
2. Run local checks (above)
3. Push to your fork
4. Create PR to main
5. CI will automatically run
6. Address any failures

**CI must pass before merge.**

#### Skipping Release

To prevent automatic release creation, add to commit message:

```bash
git commit -m "fix: minor typo [skip-release]"
```

### For Maintainers

#### Creating a Release

**Option 1: Automatic (Recommended)**

1. Update version in `Cargo.toml`:
   ```toml
   [package]
   version = "0.2.0"
   ```

2. Commit and push to main:
   ```bash
   git add Cargo.toml
   git commit -m "chore: bump version to 0.2.0"
   git push origin main
   ```

3. Wait for workflows:
   - `Create Release Tag` runs first (~3 min)
   - `Build Release Artifacts` runs after (~30 min)

4. Release is published with all binaries!

**Option 2: Manual Trigger**

1. Go to Actions tab
2. Select "Create Release Tag"
3. Click "Run workflow"
4. Choose version bump type (patch/minor/major)
5. Click "Run workflow"

#### Release Checklist

- [ ] All tests pass on main
- [ ] Version updated in Cargo.toml
- [ ] CHANGELOG updated (if you maintain one)
- [ ] Breaking changes documented
- [ ] README reflects new features
- [ ] Commit message is descriptive

#### Monitoring Releases

Watch the Actions tab for:
- ✅ Green checkmarks = success
- ❌ Red X = failure (check logs)
- 🟡 Yellow circle = running

**If release build fails:**
1. Check the specific platform that failed
2. Review error logs
3. Common issues:
   - Cross-compilation toolchain problem
   - Test failures
   - Network issues (retry workflow)

### For Users

#### Downloading Releases

1. Go to [Releases page](https://github.com/painteau/SSHChic/releases)
2. Download the binary for your platform:
   - `sshchic-linux-x86_64.tar.gz` - Linux 64-bit
   - `sshchic-macos-x86_64.tar.gz` - macOS Intel
   - `sshchic-macos-aarch64.tar.gz` - macOS Apple Silicon
   - `sshchic-windows-x86_64.zip` - Windows 64-bit
   - etc.

3. Verify checksum (optional but recommended):
   ```bash
   # Linux/macOS
   sha256sum -c sshchic-linux-x86_64.tar.gz.sha256

   # Windows (PowerShell)
   Get-FileHash sshchic-windows-x86_64.zip -Algorithm SHA256
   ```

4. Extract and install:
   ```bash
   # Linux/macOS
   tar xzf sshchic-linux-x86_64.tar.gz
   chmod +x sshchic
   sudo mv sshchic /usr/local/bin/

   # Windows: Just extract the .zip file
   ```

## Troubleshooting

### CI Failures

#### Formatting Errors

```
error: some files need formatting
```

**Fix:**
```bash
cargo fmt
git add -u
git commit --amend --no-edit
git push --force-with-lease
```

#### Clippy Warnings

```
error: this could be written more idiomatically
```

**Fix:**
```bash
cargo clippy --fix --allow-dirty
git add -u
git commit -m "fix: address clippy warnings"
```

#### Test Failures

```
test result: FAILED. X passed; Y failed
```

**Fix:**
1. Run tests locally: `cargo test`
2. Debug the specific failing test
3. Fix the code or test
4. Commit and push

#### Build Failures

```
error: could not compile `sshchic`
```

**Fix:**
1. Ensure code compiles locally: `cargo build --release`
2. Check for platform-specific issues
3. Review error logs in GitHub Actions

### Release Failures

#### Tag Already Exists

```
❌ Tag v0.1.0 already exists!
```

**Fix:**
1. Bump version in Cargo.toml to next version
2. Commit and push

#### Cross-Compilation Errors

```
error: could not cross compile for target
```

**Fix:**
1. Check `cross` version compatibility
2. Review target-specific dependencies
3. May need to update cross Docker images

### Workflow Permissions

```
Error: Resource not accessible by integration
```

**Fix:**
1. Check repository Settings → Actions → Permissions
2. Ensure "Read and write permissions" is enabled
3. Grant "Allow GitHub Actions to create pull requests"

## Maintenance

### Updating Actions

Periodically update GitHub Actions versions:

```yaml
# Check for new versions
uses: actions/checkout@v4  # Update from v3
uses: actions/cache@v4     # Update from v3
```

**How to check:**
- Visit each action's repository
- Check latest release tag
- Update in workflow files
- Test thoroughly

### Updating Rust Version

To update minimum Rust version:

1. Update in workflows:
   ```yaml
   - uses: dtolnay/rust-toolchain@stable
     # or specific version:
     # with:
     #   toolchain: 1.75.0
   ```

2. Update in Cargo.toml (optional):
   ```toml
   [package]
   rust-version = "1.75.0"
   ```

3. Test on all platforms

### Adding New Platforms

To add a new target (e.g., FreeBSD):

1. Add to matrix in `release-artefacts.yaml`:
   ```yaml
   - { name: "freebsd-x86_64", os: "ubuntu-latest", target: "x86_64-unknown-freebsd", use_cross: true }
   ```

2. Test cross-compilation:
   ```bash
   cross build --release --target x86_64-unknown-freebsd
   ```

3. Update documentation (README, ARCHITECTURE)

### Caching Optimization

Monitor cache usage:
- Go to Actions → Caches
- Review cache hit rates
- Adjust cache keys if needed
- Clean old caches periodically

### Security

**Dependabot:**
- Automatically opens PRs for dependency updates
- Review and merge regularly
- CI will test compatibility

**Secrets:**
- Never commit secrets to repository
- Use GitHub Secrets for sensitive data
- `GITHUB_TOKEN` is automatically provided

**Permissions:**
- Workflows use minimal permissions
- `contents: write` only for release workflows
- Review permission requirements regularly

## Workflow Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                      Code Push to Main                       │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
          ┌──────────────────────────────┐
          │   Build & Test Workflow      │
          │   (build-rust.yml)           │
          │                              │
          │  • Format check              │
          │  • Clippy lint               │
          │  • Run tests                 │
          │  • Build (debug + release)   │
          └──────────────┬───────────────┘
                         │
                         ▼
          ┌──────────────────────────────┐
          │  Create Release Tag          │
          │  (create-release-tag.yaml)   │
          │                              │
          │  • Pre-release checks        │
          │  • Verify version            │
          │  • Generate notes            │
          │  • Create GitHub release     │
          └──────────────┬───────────────┘
                         │
                         ▼
          ┌──────────────────────────────┐
          │  Build Release Artifacts     │
          │  (release-artefacts.yaml)    │
          │                              │
          │  • Quality checks            │
          │  • Build 8 platforms         │
          │  • Generate checksums        │
          │  • Upload to release         │
          └──────────────────────────────┘
```

## Best Practices

1. **Always test locally before pushing**
   - Saves CI time
   - Faster feedback loop
   - Reduces workflow runs

2. **Use descriptive commit messages**
   - Helps generate meaningful release notes
   - Follows conventional commits

3. **Version bumps in separate commits**
   - Makes version changes clear
   - Easier to track releases

4. **Monitor workflow runs**
   - Check Actions tab regularly
   - Address failures promptly
   - Review logs for warnings

5. **Keep dependencies updated**
   - Review Dependabot PRs
   - Test before merging
   - Update lock file

6. **Document breaking changes**
   - In commit message
   - In release notes
   - Update migration guide

## Additional Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust GitHub Actions](https://github.com/actions-rs)
- [Cross Compilation with Cross](https://github.com/cross-rs/cross)
- [Semantic Versioning](https://semver.org/)
- [Conventional Commits](https://www.conventionalcommits.org/)

---

**Last Updated:** 2025
**Maintainer:** painteau
**Questions?** Open an issue on GitHub
