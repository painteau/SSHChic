# Contributing to SSHChic

Thank you for your interest in contributing to SSHChic! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
- [Code Standards](#code-standards)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Release Process](#release-process)

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment. We welcome contributions from everyone, regardless of experience level.

### Our Standards

- Be respectful and constructive in discussions
- Focus on what is best for the project and community
- Show empathy towards other community members
- Accept constructive criticism gracefully

## Getting Started

### Prerequisites

- **Rust**: Version 1.70 or higher (we use Edition 2021)
- **Git**: For version control
- **GitHub Account**: For submitting pull requests

### Useful Resources

- [Rust Book](https://doc.rust-lang.org/book/) - Learn Rust
- [Cargo Book](https://doc.rust-lang.org/cargo/) - Rust's package manager
- [ED25519 Algorithm](https://ed25519.cr.yp.to/) - Understanding the cryptography

## Development Setup

### 1. Fork and Clone

```bash
# Fork the repository on GitHub, then clone your fork
git clone https://github.com/YOUR_USERNAME/SSHChic.git
cd SSHChic

# Add upstream remote
git remote add upstream https://github.com/painteau/SSHChic.git
```

### 2. Build the Project

```bash
# Debug build (faster compilation, slower execution)
cargo build

# Release build (slower compilation, optimized execution)
cargo build --release
```

### 3. Run the Project

```bash
# Run with cargo
cargo run -- --regex "TEST$"

# Run the binary directly
./target/debug/sshchic --regex "TEST$"
```

### 4. Check Code Quality

```bash
# Format code
cargo fmt

# Check formatting (CI requirement)
cargo fmt -- --check

# Run linter
cargo clippy

# Clippy with strict warnings (CI requirement)
cargo clippy -- -D warnings
```

### 5. Run Tests

```bash
# Run all tests
cargo test --verbose

# Run tests in release mode
cargo test --release --verbose
```

## How to Contribute

### Types of Contributions

We welcome various types of contributions:

- **Bug Reports**: Found a bug? Open an issue with details
- **Feature Requests**: Have an idea? Discuss it in an issue first
- **Documentation**: Improve docs, add examples, fix typos
- **Code**: Bug fixes, features, performance improvements
- **Testing**: Add test cases, improve coverage

### Reporting Bugs

When reporting bugs, please include:

1. **Description**: Clear description of the issue
2. **Steps to Reproduce**: Minimal steps to reproduce the bug
3. **Expected Behavior**: What you expected to happen
4. **Actual Behavior**: What actually happened
5. **Environment**:
   - OS and version (e.g., Ubuntu 22.04, macOS 13, Windows 11)
   - Rust version (`rustc --version`)
   - SSHChic version or commit hash
6. **Regex Pattern**: The pattern you were searching for
7. **Command Used**: Full command line with flags

**Example Bug Report:**

```markdown
**Description**: Program crashes when using complex regex with lookaheads

**Steps to Reproduce**:
1. Run: `sshchic --regex "(?=.*SSH)(?=.*KEY)"`
2. Wait 10 seconds
3. Program crashes

**Expected**: Should search for keys matching both patterns
**Actual**: Segmentation fault

**Environment**:
- OS: Ubuntu 22.04
- Rust: 1.75.0
- SSHChic: commit abc123

**Error Output**:
[paste error here]
```

### Suggesting Features

Before suggesting a feature:

1. **Search existing issues** to avoid duplicates
2. **Open a discussion issue** explaining:
   - What problem does it solve?
   - How would it work?
   - Are there alternatives?
   - Would it break existing functionality?

Wait for feedback before implementing large features.

## Code Standards

### Rust Style Guide

We follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/). Key points:

- **Formatting**: Use `rustfmt` (enforced in CI)
- **Linting**: Pass `clippy` with zero warnings (enforced in CI)
- **Naming**:
  - `snake_case` for functions, variables, modules
  - `PascalCase` for types, traits
  - `SCREAMING_SNAKE_CASE` for constants
- **Imports**: Group and sort imports logically

### Documentation

All public items must have documentation:

```rust
/// Brief one-line description
///
/// Detailed explanation of what this does.
///
/// # Arguments
///
/// * `param` - Description of parameter
///
/// # Returns
///
/// Description of return value
///
/// # Examples
///
/// ```
/// // Example code
/// ```
///
/// # Panics
///
/// When this panics (if applicable)
pub fn my_function(param: Type) -> ReturnType {
    // implementation
}
```

### Error Handling

- **Avoid `unwrap()`** in production code where possible
- Use `expect()` with descriptive messages for unrecoverable errors
- Return `Result` types for recoverable errors
- Provide context in error messages

### Performance

- **Profile before optimizing**: Use `cargo bench` or `perf`
- **Justify optimizations**: Comment why, not just what
- **Maintain readability**: Don't sacrifice clarity for minor gains

### Security

- **No hardcoded secrets**: Use environment variables or config files
- **Validate input**: Especially regex patterns
- **Safe dependencies**: Keep dependencies updated
- **Review crypto code carefully**: ED25519 implementation is critical

## Testing

### Test Coverage

We aim for comprehensive testing:

- **Unit tests**: Test individual functions
- **Integration tests**: Test component interaction
- **Functional tests**: Test end-to-end workflows (in CI)
- **Edge cases**: Test boundary conditions

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_name() {
        // Arrange
        let input = /* setup */;

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected);
    }

    #[test]
    #[should_panic(expected = "error message")]
    fn test_error_case() {
        // Test that should panic
    }
}
```

### Running Specific Tests

```bash
# Run a specific test
cargo test test_feature_name

# Run tests matching a pattern
cargo test pattern

# Show test output
cargo test -- --nocapture

# Run tests with threads
cargo test -- --test-threads=1
```

## Pull Request Process

### Before Submitting

1. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**:
   - Write clean, documented code
   - Follow code standards
   - Add tests for new functionality

3. **Run quality checks**:
   ```bash
   # Format code
   cargo fmt

   # Run linter
   cargo clippy -- -D warnings

   # Run tests
   cargo test --verbose

   # Build in release mode
   cargo build --release
   ```

4. **Commit changes**:
   ```bash
   git add .
   git commit -m "feat: add support for custom output filenames"
   ```

### Commit Message Convention

We use [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `style:` - Code style changes (formatting, no logic change)
- `refactor:` - Code refactoring
- `perf:` - Performance improvements
- `test:` - Adding or updating tests
- `chore:` - Maintenance tasks, dependency updates
- `ci:` - CI/CD changes

**Examples:**
```
feat: add support for RSA key generation
fix: prevent crash on invalid regex patterns
docs: add examples for fingerprint matching
perf: optimize key generation loop
```

### Submitting the PR

1. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Create Pull Request** on GitHub with:
   - **Title**: Clear, concise description (using conventional commit format)
   - **Description**:
     - What does this PR do?
     - Why is this change needed?
     - How has it been tested?
     - Any breaking changes?
     - Related issues (use `Fixes #123` to auto-close)
   - **Screenshots/Output**: If applicable

3. **PR Checklist**:
   - [ ] Code follows the style guide
   - [ ] All tests pass locally
   - [ ] New code has tests
   - [ ] Documentation is updated
   - [ ] Commit messages follow convention
   - [ ] No unnecessary dependencies added
   - [ ] Changes are backwards compatible (or clearly documented)

### Review Process

1. **Automated Checks**: CI will run tests on multiple platforms
2. **Code Review**: Maintainers will review your code
3. **Feedback**: Address any requested changes
4. **Approval**: Once approved, a maintainer will merge

**Be patient**: Reviews may take a few days. Feel free to ping after a week.

### After Merge

1. **Sync your fork**:
   ```bash
   git checkout main
   git pull upstream main
   git push origin main
   ```

2. **Delete feature branch**:
   ```bash
   git branch -d feature/your-feature-name
   git push origin --delete feature/your-feature-name
   ```

## Release Process

> **Note**: This section is for maintainers

### Versioning

We use [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backwards compatible)
- **PATCH**: Bug fixes

### Creating a Release

1. **Update version** in `Cargo.toml`
2. **Update CHANGELOG.md** with changes
3. **Commit**: `git commit -m "chore: bump version to X.Y.Z"`
4. **Tag**: `git tag -a vX.Y.Z -m "Release vX.Y.Z"`
5. **Push**: `git push && git push --tags`
6. **GitHub Release**: Create release from tag (triggers CI to build artifacts)

## Development Tips

### Debugging

```bash
# Run with debug output
RUST_BACKTRACE=1 cargo run -- --regex "TEST"

# Full backtrace
RUST_BACKTRACE=full cargo run -- --regex "TEST"

# Use a debugger
rust-gdb ./target/debug/sshchic
```

### Performance Profiling

```bash
# Install perf tools
sudo apt install linux-tools-generic

# Build with debug symbols
cargo build --release

# Profile
perf record --call-graph=dwarf ./target/release/sshchic --regex "TEST$"
perf report

# Or use cargo-flamegraph
cargo install flamegraph
cargo flamegraph -- --regex "TEST$"
```

### Benchmarking

```bash
# Add criterion to dev-dependencies for benchmarks
# Run benchmarks
cargo bench
```

### Cross-Compilation

```bash
# Install cross
cargo install cross

# Build for different targets
cross build --target x86_64-unknown-linux-musl
cross build --target aarch64-unknown-linux-gnu
```

## Project Structure

```
SSHChic/
├── src/
│   └── main.rs                      # Main application code
├── docs/                            # Additional documentation
│   ├── ARCHITECTURE.md              # Technical architecture
│   ├── EXAMPLES.md                  # Usage examples
│   └── WORKFLOWS.md                 # CI/CD documentation
├── .github/
│   └── workflows/                   # CI/CD workflows
│       ├── build-rust.yml           # Build and test
│       ├── create-release-tag.yaml  # Release automation
│       └── release-artefacts.yaml   # Multi-platform builds
├── Cargo.toml                       # Rust package manifest
├── README.md                        # User documentation
├── CONTRIBUTING.md                  # This file
└── LICENSE                          # MIT License
```

## CI/CD Workflows

SSHChic uses automated GitHub Actions workflows for continuous integration and deployment. Understanding these workflows helps you contribute effectively.

### Build & Test Workflow

**When it runs:**
- On every push to main
- On every pull request
- When you manually trigger it

**What it checks:**
- Code formatting (`cargo fmt`)
- Linting with Clippy (`cargo clippy`)
- All tests pass (`cargo test`)
- Builds successfully on Linux, macOS, and Windows

**Your responsibility:**
Before pushing, always run:
```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
```

This ensures CI will pass!

### Release Workflow

**Automatic release creation:**
- When you push to main with code changes
- Creates a GitHub release with binaries for all platforms
- Runs quality checks before releasing

**Skip release:**
Add `[skip-release]` to your commit message:
```bash
git commit -m "docs: update README [skip-release]"
```

**For maintainers:**
See [docs/WORKFLOWS.md](docs/WORKFLOWS.md) for detailed release process.

### Workflow Status

Check workflow status:
- Go to the **Actions** tab on GitHub
- Look for green checkmarks ✅ (success) or red X ❌ (failure)
- Click on failed runs to see error logs

**If your PR fails CI:**
1. Review the error logs
2. Fix the issues locally
3. Push updates to your branch
4. CI will automatically re-run

### More Information

For comprehensive workflow documentation, see:
- **[docs/WORKFLOWS.md](docs/WORKFLOWS.md)** - Complete CI/CD guide
- Covers all workflows in detail
- Troubleshooting common issues
- Maintenance and updates

## Getting Help

- **Documentation**: Check the README and code documentation
- **Issues**: Search existing issues or create a new one
- **Discussions**: Use GitHub Discussions for questions
- **Code Review**: Tag maintainers in your PR

## Recognition

Contributors will be recognized in:
- GitHub contributors page
- Release notes (for significant contributions)

Thank you for contributing to SSHChic! 🎉
