# MeCab-Ko CI/CD Workflow Guide

This document describes the comprehensive CI/CD pipeline for the MeCab-Ko project, designed for efficiency, reliability, and fast feedback loops.

## Overview

The CI/CD system is split into three focused workflows:

1. **ci.yml** - Core CI pipeline for library crates
2. **code-quality.yml** - Extended quality checks and metrics
3. **ffi-tests.yml** - Separate testing for FFI/binding crates with special environments

## Workflow 1: Core CI Pipeline (`ci.yml`)

The main CI workflow runs on every push and pull request to `main`, `master`, or `develop` branches.

### Triggers

- **Push events** to main/master/develop branches
- **Pull requests** to main/master/develop branches
- **Manual trigger** via `workflow_dispatch`
- **Path-based filtering** to avoid unnecessary runs (only when rust/ or config changes)

### Environment Variables

```yaml
CARGO_TERM_COLOR: always         # Colored output
RUST_BACKTRACE: 1               # Enable backtraces
RUSTFLAGS: -D warnings          # Treat warnings as errors
CARGO_INCREMENTAL: 0            # Disable incremental compilation for cleaner builds
```

### Jobs (Run in Parallel)

#### 1. `fmt` - Rustfmt Check
- **Purpose**: Ensure consistent code formatting
- **Command**: `cargo fmt --check`
- **Duration**: ~5-10 seconds
- **Fails PR**: Yes (critical check)

#### 2. `clippy` - Linter (Library Crates Only)
- **Purpose**: Catch common Rust mistakes and code quality issues
- **Command**: `cargo clippy` with all targets and features
- **Excluded Crates**:
  - `mecab-ko-python` (requires Python)
  - `mecab-ko-node` (requires Node.js)
  - `mecab-ko-wasm` (requires wasm-pack/clang)
  - `mecab-ko-elasticsearch` (requires JDK)
- **Duration**: ~2-3 minutes
- **Caching**: Uses `Swatinem/rust-cache@v2` for fast incremental builds
- **Fails PR**: Yes (critical check)

#### 3. `test` - Test Suite
- **Purpose**: Run unit and integration tests
- **Runs On**: Ubuntu latest (fast feedback on primary platform)
- **Rust Toolchains**: Stable, Beta, Nightly (3x matrix)
- **Test Variants**:
  - Debug build tests
  - Release build tests (optimizations enabled)
- **Excluded Crates**: FFI crates (see above)
- **Duration**: ~5-10 minutes per toolchain
- **Environment**: Full backtrace on debug tests
- **Caching**: Separate cache per toolchain
- **Fails PR**: Yes (critical check)

#### 4. `build` - Multi-Platform Build
- **Purpose**: Verify builds work across all supported platforms
- **Platforms**:
  - Linux (x86_64-unknown-linux-gnu)
  - macOS (x86_64-apple-darwin)
  - Windows (x86_64-pc-windows-msvc)
- **Build Type**: Release (with optimizations)
- **Excluded Crates**: FFI crates
- **Duration**: ~5-10 minutes per platform
- **Caching**: Per-platform cache
- **Fails PR**: Yes (critical check)

#### 5. `docs` - Documentation Build
- **Purpose**: Ensure documentation compiles without warnings
- **Command**: `cargo doc` on release profile
- **Flags**: `-D warnings` (treat warnings as errors)
- **Excluded Crates**: FFI crates
- **Duration**: ~3-5 minutes
- **Caching**: Dedicated cargo cache
- **Fails PR**: Yes (critical check)

#### 6. `security-audit` - Security Scanning
- **Purpose**: Check for known vulnerabilities in dependencies
- **Tools**:
  - RustSec audit-check-action (GitHub integration)
  - cargo-audit (detailed vulnerability reporting)
- **Duration**: ~1-2 minutes
- **Fails PR**: Yes (security-critical)

#### 7. `coverage` - Code Coverage
- **Purpose**: Track code coverage metrics for regression detection
- **Tool**: cargo-tarpaulin
- **Coverage Format**: Cobertura XML
- **Exclusions**: test files, benchmark files
- **Upload**: Codecov.io integration
- **Duration**: ~10-15 minutes
- **Fails PR**: No (informational only)
- **Artifacts**: Coverage reports retained 90 days

#### 8. `ci-status` - Status Summary
- **Purpose**: Provide overall pass/fail status for the PR
- **Depends On**: All previous jobs
- **Fails PR**: Yes if any critical job fails
- **Behavior**: Runs even if some jobs fail (always condition)

### Caching Strategy

Uses `Swatinem/rust-cache@v2` for efficient cargo caching:
- Automatic cargo registry cache
- Automatic cargo git index cache
- Automatic build artifacts cache
- Per-key granularity for different check types
- Separate caches per toolchain/OS for parallel jobs

### Expected Duration

- **Quick run** (no changes to rust): ~2 minutes (skipped)
- **Typical run**: ~20-30 minutes (all jobs in parallel)
- **With coverage**: ~40-50 minutes total

## Workflow 2: Code Quality Analysis (`code-quality.yml`)

Runs extended quality checks and detailed analysis.

### Triggers

- **Push/PR** to main/master/develop
- **Schedule**: Daily at 3 AM UTC
- **Manual trigger** via `workflow_dispatch`

### Jobs

#### 1. `unused-dependencies` - Unused Dependency Detection
- **Purpose**: Identify and remove unused dependencies
- **Tool**: cargo-udeps (nightly only)
- **Command**: Checks all targets in workspace
- **Excluded Crates**: FFI crates
- **Continue on Error**: Yes (warning, not blocker)
- **Duration**: ~5-10 minutes

#### 2. `dependency-outdated` - Freshness Check
- **Purpose**: Report outdated dependencies
- **Tool**: cargo-outdated
- **Output**: List of outdated packages
- **Continue on Error**: Yes (informational)
- **Duration**: ~3-5 minutes

#### 3. `code-metrics` - Code Statistics
- **Purpose**: Generate code metrics and line counts
- **Tool**: tokei (line counting utility)
- **Output**:
  - Lines of code per language
  - File/module statistics
  - Artifact upload for tracking
- **Duration**: ~2-3 minutes

#### 4. `docs-check` - Documentation Coverage
- **Purpose**: Ensure all public APIs have documentation
- **Command**: `cargo doc` with private items
- **Flags**: Warnings treated as errors
- **Excluded Crates**: FFI crates
- **Duration**: ~3-5 minutes

#### 5. `summary` - Quality Summary
- **Purpose**: Aggregate results and report status
- **Output**: GitHub Step Summary (markdown table)
- **Critical Checks**: Unused dependencies, documentation
- **Fails Workflow**: If critical checks fail

### Expected Duration

- ~30-40 minutes total (jobs run in parallel)
- Scheduled runs provide overnight feedback

## Workflow 3: FFI Crates Testing (`ffi-tests.yml`)

Specialized testing for language bindings and plugins requiring external environments.

### Triggers

- **Path-based**: Only when FFI crate code changes
- **Separate from main CI**: Isolated environment setup

### FFI Crates Covered

1. **mecab-ko-python**: Python bindings (PyO3)
2. **mecab-ko-node**: Node.js bindings (NAPI)
3. **mecab-ko-wasm**: WebAssembly bindings
4. **mecab-ko-elasticsearch**: Elasticsearch plugin (JNI)

### Jobs

#### 1. `python-bindings` - Python Binding Tests
- **Python Versions**: 3.8, 3.9, 3.10, 3.11, 3.12 (5x matrix)
- **Setup**: `actions/setup-python@v4`
- **Build Tool**: maturin (PyO3 build tool)
- **Tests**: pytest if tests directory exists
- **Clippy**: Runs on binding code with strict checks
- **Duration**: ~15-20 minutes total

#### 2. `node-bindings` - Node.js Binding Tests
- **Node.js Versions**: 18.x, 20.x, 21.x (3x matrix)
- **Setup**: `actions/setup-node@v4`
- **Build Tool**: npm (with native build)
- **Tests**: npm test if implemented
- **Clippy**: Runs with strict checks
- **Duration**: ~15-20 minutes total

#### 3. `wasm-bindings` - WebAssembly Binding Tests
- **Platform**: wasm32-unknown-unknown
- **Build Tool**: wasm-pack
- **Build Flags**: `--target bundler --release`
- **Tests**: Headless Firefox tests
- **Clippy**: WASM-specific checks
- **Duration**: ~10-15 minutes

#### 4. `elasticsearch-plugin` - Elasticsearch Plugin
- **Build**: Rust library only (JDK setup expensive in CI)
- **Clippy**: Strict checks on plugin code
- **Docs**: Generate with warnings-as-errors
- **Duration**: ~5-10 minutes

#### 5. `ffi-status` - FFI Status Summary
- **Purpose**: Aggregate FFI test results
- **Output**: GitHub Step Summary
- **Fails Workflow**: If any critical job fails

### Expected Duration

- ~40-50 minutes total (parallel jobs)
- Only triggers on FFI code changes

## Excluded Crates Rationale

### Why FFI Crates Are Excluded from Main CI

**mecab-ko-python**
- Requires Python runtime
- Requires maturin for build
- Heavy setup time (~2-3 minutes)
- Only builds with `cargo build` would fail without Python

**mecab-ko-node**
- Requires Node.js runtime
- Requires npm/node-gyp
- Heavy setup time (~2-3 minutes)
- NAPI-specific compilation requirements

**mecab-ko-wasm**
- Requires wasm-pack tool
- Requires clang/LLVM for WASM target
- Heavy setup time (~3-5 minutes)
- Different compilation profile than native

**mecab-ko-elasticsearch**
- Requires JDK for full testing
- JNI integration is complex
- Testing requires running Elasticsearch
- Kept to basic compilation + clippy

### Benefits of Separation

1. **Faster Main CI**: Main workflow runs in ~20-30 minutes without FFI overhead
2. **Parallel Execution**: FFI tests run independently without blocking main CI
3. **Environment Isolation**: Each FFI crate gets optimized environment setup
4. **Selective Testing**: Changes to FFI crates only trigger FFI tests
5. **Cost Efficiency**: Expensive environments (JDK, Python, Node) only when needed

## Code Quality Standards

### Linting Rules

All library crates enforce:
```rust
[workspace.lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
nursery = { level = "warn", priority = -1 }
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
```

### Documentation Requirements

- All public APIs must have rustdoc
- Documentation comments must compile without warnings
- Documentation checks run with `-D warnings`

### Testing Requirements

- Unit tests for all public modules
- Integration tests for feature combinations
- Coverage tracked (target: >80%)

## Performance Optimization Tips

### For Local Development

```bash
# Use mold for faster linking on Linux
export RUSTFLAGS="-C link-arg=-fuse-ld=mold"

# Enable incremental compilation
export CARGO_INCREMENTAL=1

# Parallel compilation
export CARGO_BUILD_JOBS=$(nproc)

# Run only specific checks
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

### For Faster CI Feedback

1. **Run fmt before push**: Prevents formatting-only CI failures
2. **Run clippy locally**: `cargo clippy -- -D warnings`
3. **Run tests locally**: `cargo test --release`
4. **Use `--lib` for library-only testing**: Skip binary/example tests during development

## Troubleshooting Common CI Failures

### Rustfmt Failures

```bash
# Auto-format code
cargo fmt --manifest-path rust/Cargo.toml
```

### Clippy Failures

```bash
# Check clippy issues
cargo clippy --manifest-path rust/Cargo.toml -- -D warnings

# For specific crate
cargo clippy -p crate-name -- -D warnings
```

### Test Failures

```bash
# Run tests with backtraces
RUST_BACKTRACE=1 cargo test --manifest-path rust/Cargo.toml

# Run specific test
cargo test --manifest-path rust/Cargo.toml test_name -- --nocapture
```

### Documentation Build Failures

```bash
# Build docs with same flags as CI
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rust/Cargo.toml --no-deps
```

### Cache Issues

If you suspect cache corruption:
1. Go to repo Settings → Actions → General
2. Click "Clear all caches"
3. Re-run the workflow

## Monitoring and Notifications

### GitHub Integration

- PR status checks show pass/fail for each job
- Required status checks prevent merging until all pass
- Detailed logs available for each job
- Artifacts (coverage, stats) downloadable from workflow run

### Code Coverage

- Uploaded to Codecov.io automatically
- Coverage reports generated as artifacts
- Tracked over time for regression detection

## Future Improvements

### Planned Enhancements

1. **MSRV Testing**: Test against minimum supported Rust version (1.75)
2. **Cross-Platform Tests**: Run tests on additional architectures (ARM, etc.)
3. **Performance Regression Detection**: Track benchmark results over time
4. **Dependency Audit**: Daily checks for security advisories
5. **License Compliance**: Ensure all dependencies meet license requirements
6. **SBOM Generation**: Supply chain security (SLSA provenance)

### Scheduled Tasks

- **Daily (3 AM UTC)**: Code quality analysis
- **Daily (2 AM UTC)**: Security audit and dependency checks
- **Weekly**: Comprehensive cross-platform builds

## Configuration Files Reference

### Main Workflows

- `.github/workflows/ci.yml` - Core CI pipeline
- `.github/workflows/code-quality.yml` - Quality analysis
- `.github/workflows/ffi-tests.yml` - FFI binding tests

### Related Configuration

- `.github/workflows/release.yml` - Release automation
- `.github/workflows/security.yml` - Security scanning (complementary)
- `.github/workflows/docs.yml` - Documentation deployment
- `rust/Cargo.toml` - Workspace lints and dependencies
- `rust/Cargo.lock` - Dependency lock file

## Support and Questions

For questions about the CI/CD pipeline:

1. Check this guide's troubleshooting section
2. Review GitHub Actions logs for specific failures
3. Consult the project's development documentation
4. Open an issue in the project repository

---

**Last Updated**: 2026-02-10
**Rust Version**: 1.75+
**Status**: Production Ready
