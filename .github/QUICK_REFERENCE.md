# MeCab-Ko CI/CD Quick Reference

## Key Workflows

| Workflow | File | Trigger | Duration | Purpose |
|----------|------|---------|----------|---------|
| **Main CI** | `ci.yml` | PR/Push | 20-30 min | Core library tests + checks |
| **Quality** | `code-quality.yml` | Schedule/PR | 30-40 min | Deep analysis (daily) |
| **FFI Tests** | `ffi-tests.yml` | FFI changes | 40-50 min | Language binding tests |

## Crate Classification

### Core Library (9 crates - Main CI)
```
mecab-ko, mecab-ko-core, mecab-ko-dict, mecab-ko-hangul,
mecab-ko-cli, mecab-ko-dict-builder, mecab-ko-dict-validator,
mecab-ko-profiler, benchmarks
```

### FFI Bindings (4 crates - Separate Workflow)
```
mecab-ko-python (5 Python versions)
mecab-ko-node (3 Node versions)
mecab-ko-wasm (WebAssembly)
mecab-ko-elasticsearch (JNI)
```

## Local Development Commands

```bash
# Format check
cargo fmt --check

# Strict linting
cargo clippy -- -D warnings

# Run tests
cargo test --release

# Build documentation
cargo doc --no-deps

# Security audit
cargo audit --deny warnings

# Code coverage
cargo tarpaulin --out Html
```

## CI Pipeline Stages

```
fmt (formatting)
  ↓
clippy + test + build (parallel, 2-10 min each)
  ↓
docs + security-audit (parallel, 1-5 min each)
  ↓
coverage (5-10 min)
  ↓
ci-status (summary)
```

## Performance Metrics

| Scenario | Duration |
|----------|----------|
| First run on new platform | 30-60 min |
| Typical run (cache hit) | 15-25 min |
| FFI tests (parallel) | 40-50 min |
| Quality analysis (daily) | 30-40 min |

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Formatting fails | Run `cargo fmt --manifest-path rust/Cargo.toml` |
| Clippy warnings | Run `cargo clippy -- -D warnings` locally first |
| Tests fail | Run `RUST_BACKTRACE=1 cargo test` locally |
| Docs fail | Run `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` |
| Cache issues | Clear in GitHub Actions → Settings → General |

## Environment Variables

```yaml
CARGO_TERM_COLOR: always        # Colored output
RUST_BACKTRACE: 1              # Backtraces enabled
RUSTFLAGS: -D warnings         # Warnings as errors
CARGO_INCREMENTAL: 0           # Clean builds
```

## Caching

- **Tool**: Swatinem/rust-cache@v2
- **Speed**: 30-50% faster with cache hits
- **Keys**: Per-platform, per-toolchain
- **Scope**: ~/.cargo registry, target/ artifacts

## Required Status Checks

These checks must pass before merging:

- [x] fmt (formatting)
- [x] clippy (linting)
- [x] test (unit tests)
- [x] build (multi-platform)
- [x] docs (documentation)
- [x] security-audit (vulnerabilities)
- [x] coverage (code metrics)

## Documentation

| File | Purpose |
|------|---------|
| `CI_WORKFLOW_GUIDE.md` | Comprehensive reference (400+ lines) |
| `CRATE_CI_MATRIX.md` | Crate organization matrix (320+ lines) |
| `IMPLEMENTATION_CHECKLIST.md` | Deployment guide (360+ lines) |
| `QUICK_REFERENCE.md` | This file |

## Platform Support

| Platform | OS | Arch | Rust |
|----------|-------|------|------|
| Linux | Ubuntu | x86_64 | stable, beta, nightly |
| macOS | latest | x86_64 | stable |
| Windows | latest | x86_64 | stable |

## Python Versions (FFI)
- 3.8, 3.9, 3.10, 3.11, 3.12

## Node.js Versions (FFI)
- 18.x, 20.x, 21.x

## Monitoring

- **GitHub Actions**: repo → Actions tab
- **Codecov**: Coverage trends
- **RustSec**: Security database updates
- **Artifacts**: Available for 90 days

## Tips for Fast Feedback

1. Run `cargo fmt --check` before pushing
2. Run `cargo clippy -- -D warnings` locally
3. Run `cargo test --release` before PR
4. Use `--lib` flag to skip binary/example tests
5. Check GitHub Actions dashboard before asking questions

## Common Issues

**Problem**: "warnings treated as errors"
**Solution**: Fix warnings or disable RUSTFLAGS locally

**Problem**: "cache not working"
**Solution**: Second run should be faster, clear if needed

**Problem**: "FFI tests not running"
**Solution**: FFI tests only run when FFI code changes

**Problem**: "documentation warnings"
**Solution**: Run `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps`

## Performance Tips

- **First run**: Can take 30-60 minutes
- **Cached run**: ~15-25 minutes
- **Multiple PRs**: Share cache for faster builds
- **Local testing**: Always test locally first

## Getting Help

1. Check `CI_WORKFLOW_GUIDE.md` for detailed info
2. Review `CRATE_CI_MATRIX.md` for crate details
3. See workflow files for configuration
4. Open issue for bugs or enhancements

---

**Last Updated**: 2026-02-10
**Status**: Production Ready
