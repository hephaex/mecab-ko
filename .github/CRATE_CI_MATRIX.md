# MeCab-Ko Crate CI/CD Matrix

This document maps each crate in the workspace to its CI/CD testing strategy.

## Workspace Structure

The project is organized as a Cargo workspace with the following crates:

```
rust/
├── Cargo.toml (workspace root)
└── crates/
    ├── mecab-ko                    (main library - umbrella)
    ├── mecab-ko-core               (core tokenizer engine)
    ├── mecab-ko-dict               (dictionary management)
    ├── mecab-ko-dict-builder       (CLI: dictionary builder)
    ├── mecab-ko-dict-validator     (CLI: dictionary validator)
    ├── mecab-ko-hangul             (hangul text utilities)
    ├── mecab-ko-cli                (CLI: morphological analyzer)
    ├── mecab-ko-python             (Python bindings - FFI)
    ├── mecab-ko-node               (Node.js bindings - FFI)
    ├── mecab-ko-wasm               (WebAssembly bindings - FFI)
    ├── mecab-ko-elasticsearch      (Elasticsearch plugin - FFI)
    ├── mecab-ko-profiler           (profiling tools)
    └── benchmarks                  (benchmark suite)
```

## Crate Classification and CI Strategy

### Core Library Crates (Included in Main CI)

These crates are tested in the main `ci.yml` workflow.

| Crate | Type | Dependencies | CI Tests | Clippy | Docs | Coverage |
|-------|------|--------------|----------|--------|------|----------|
| **mecab-ko** | Library | Core + others | ✓ | ✓ | ✓ | ✓ |
| **mecab-ko-core** | Library | hangul, dict | ✓ | ✓ | ✓ | ✓ |
| **mecab-ko-dict** | Library | hangul, bincode | ✓ | ✓ | ✓ | ✓ |
| **mecab-ko-hangul** | Library | none | ✓ | ✓ | ✓ | ✓ |
| **mecab-ko-cli** | Binary | core, dict, etc | ✓ | ✓ | ✓ | ✓ |
| **mecab-ko-dict-builder** | Binary | dict, csv | ✓ | ✓ | ✓ | ✓ |
| **mecab-ko-dict-validator** | Binary | dict, csv | ✓ | ✓ | ✓ | ✓ |
| **mecab-ko-profiler** | Library | core, dict | ✓ | ✓ | ✓ | ✓ |
| **benchmarks** | Benchmark | core, dict | ✓ | ✓ | ✗ | ✓ |

### FFI/Binding Crates (Separate in ffi-tests.yml)

These crates require special environments and are tested separately.

| Crate | Type | Environment | Build Tool | CI Tests | Clippy | Docs |
|-------|------|-------------|------------|----------|--------|------|
| **mecab-ko-python** | FFI | Python 3.8-3.12 | maturin | ✓ | ✓ | ✗ |
| **mecab-ko-node** | FFI | Node.js 18-21 | npm/napi-rs | ✓ | ✓ | ✗ |
| **mecab-ko-wasm** | FFI | wasm32 target | wasm-pack | ✓ | ✓ | ✗ |
| **mecab-ko-elasticsearch** | FFI | JDK (partial) | cargo | ✓ | ✓ | ✓ |

## CI/CD Workflow Routing

### Main CI Workflow (`ci.yml`)

**Triggers**: Push to main/master/develop, PRs, manual

**Included Crates**: All except mecab-ko-python, mecab-ko-node, mecab-ko-wasm, mecab-ko-elasticsearch

**Jobs**:
1. `fmt` - rustfmt check (all crates)
2. `clippy` - linter (library crates only)
3. `test` - unit/integration tests (library crates)
4. `build` - release builds (multi-platform)
5. `docs` - documentation compilation
6. `security-audit` - vulnerability scanning
7. `coverage` - code coverage analysis
8. `ci-status` - summary and pass/fail

**Expected Duration**: 20-30 minutes

### Code Quality Workflow (`code-quality.yml`)

**Triggers**: Push/PR to main/master/develop, daily schedule, manual

**Included Crates**: Core crates only (excludes FFI)

**Jobs**:
1. `unused-dependencies` - cargo-udeps (nightly)
2. `dependency-outdated` - cargo-outdated
3. `code-metrics` - tokei statistics
4. `docs-check` - documentation coverage
5. `summary` - results aggregation

**Expected Duration**: 30-40 minutes (scheduled daily)

### FFI Tests Workflow (`ffi-tests.yml`)

**Triggers**: Push/PR when FFI crate code changes, manual

**Included Crates**: mecab-ko-python, mecab-ko-node, mecab-ko-wasm, mecab-ko-elasticsearch

**Jobs**:
1. `python-bindings` - 5 Python versions
2. `node-bindings` - 3 Node.js versions
3. `wasm-bindings` - WebAssembly targets
4. `elasticsearch-plugin` - JNI library
5. `ffi-status` - summary

**Expected Duration**: 40-50 minutes

## Dependency Graph

```
┌─ benchmarks
│  └─ core, dict
│
├─ mecab-ko (umbrella)
│  └─ core, dict, hangul, etc
│
├─ mecab-ko-core
│  ├─ hangul
│  └─ dict
│
├─ mecab-ko-dict
│  └─ hangul
│
├─ mecab-ko-hangul (no deps)
│
├─ mecab-ko-cli
│  └─ core (transitively: hangul, dict)
│
├─ mecab-ko-dict-builder
│  └─ dict, csv
│
├─ mecab-ko-dict-validator
│  └─ dict, csv
│
├─ mecab-ko-profiler
│  └─ core, dict
│
├─ mecab-ko-python (FFI)
│  └─ core, dict, hangul (as libs)
│
├─ mecab-ko-node (FFI)
│  └─ core, dict, hangul (as libs)
│
├─ mecab-ko-wasm (FFI)
│  └─ core, dict, hangul (as libs)
│
└─ mecab-ko-elasticsearch (FFI)
   └─ core, dict, hangul (as libs)
```

## Crate Testing Matrix

### Unit/Integration Tests

| Crate | Debug | Release | Clippy | Docs |
|-------|-------|---------|--------|------|
| mecab-ko-hangul | ✓ | ✓ | ✓ | ✓ |
| mecab-ko-dict | ✓ | ✓ | ✓ | ✓ |
| mecab-ko-core | ✓ | ✓ | ✓ | ✓ |
| mecab-ko | ✓ | ✓ | ✓ | ✓ |
| mecab-ko-cli | ✓ | ✓ | ✓ | ✓ |
| mecab-ko-dict-builder | ✓ | ✓ | ✓ | ✓ |
| mecab-ko-dict-validator | ✓ | ✓ | ✓ | ✓ |
| mecab-ko-profiler | ✓ | ✓ | ✓ | ✓ |
| benchmarks | ✓ | ✓ | ✓ | ✗ |

### Multi-Platform Builds

All core library crates build on:
- Ubuntu (x86_64-unknown-linux-gnu)
- macOS (x86_64-apple-darwin)
- Windows (x86_64-pc-windows-msvc)

## Platform Coverage

### Core CI Platforms

- **Linux**: Primary development platform, full test suite
- **macOS**: Cross-platform verification
- **Windows**: Windows compatibility testing

### FFI CI Platforms

- **Python**: All 3 major platforms (Linux, macOS, Windows)
- **Node.js**: All 3 major platforms (Linux, macOS, Windows)
- **WASM**: Linux (wasm32-unknown-unknown target)
- **Elasticsearch**: Linux (JDK not required for basic build)

## Rust Toolchain Testing

### Main Tests

- **stable**: Default toolchain for all jobs
- **beta**: Catch future incompatibilities
- **nightly**: Latest Rust features (in test job)

### FFI Specific

- **Python**: stable only (compatibility requirement)
- **Node.js**: stable only (NAPI stability)
- **WASM**: stable + specific wasm target

## Build Profiles

### Main CI

| Job | Profile | Optimizations |
|-----|---------|---------------|
| fmt | N/A | N/A |
| clippy | check | None |
| test | debug & release | Full |
| build | release | LTO + single codegen unit |
| docs | release | LTO |
| coverage | debug | Coverage instrumentation |

### Code Quality

| Job | Profile | Optimizations |
|-----|---------|---------------|
| unused-deps | check | None |
| dependency-outdated | check | None |
| code-metrics | check | None |
| docs-check | release | LTO |

### FFI Tests

| Crate | Build Tool | Profile |
|-------|------------|---------|
| Python | maturin | release |
| Node.js | npm | release |
| WASM | wasm-pack | release |
| Elasticsearch | cargo | release |

## Caching Strategy

### Cargo Caches

All workflows use `Swatinem/rust-cache@v2` for:
- Cargo registry cache (~/.cargo/registry)
- Cargo git index cache (~/.cargo/git)
- Build artifacts (rust/target)

### Cache Keys

- **Main**: `${{ runner.os }}-cargo-build-target-${{ matrix.rust }}-${{ hashFiles('**/Cargo.lock') }}`
- **Clippy**: Separate key for faster incremental checks
- **FFI**: Per-platform/per-version caches

## Failure Handling

### Blocking Failures (PR cannot merge)

1. **fmt**: Code formatting
2. **clippy**: Linter warnings
3. **test**: Test failures
4. **build**: Compilation failures
5. **docs**: Documentation warnings
6. **security-audit**: Vulnerable dependencies

### Non-Blocking (Informational)

1. **coverage**: Code coverage metrics
2. **unused-dependencies**: Unused imports
3. **dependency-outdated**: Version updates available
4. **code-metrics**: Complexity analysis

## Performance Characteristics

### Typical Run Times (on Ubuntu)

| Job | Single Run | With Cache |
|-----|-----------|-----------|
| fmt | 5s | 5s |
| clippy | 2-3 min | 1-2 min |
| test | 5-10 min | 3-5 min |
| build (all) | 15-20 min | 5-10 min |
| docs | 3-5 min | 2-3 min |
| security | 1-2 min | 1-2 min |
| coverage | 10-15 min | 5-10 min |
| **Total** | ~45-60 min | ~25-35 min |

### FFI Tests

| Job | Duration | Parallelism |
|-----|----------|------------|
| Python (5 versions) | ~20 min | Parallel |
| Node.js (3 versions) | ~15 min | Parallel |
| WASM | ~10 min | Single |
| Elasticsearch | ~5 min | Single |
| **Total** | ~40-50 min | Parallel |

## Maintenance Guidelines

### Adding New Crates

1. **Library crate**: Automatically included in main CI
   - Ensure Cargo.toml is in workspace members
   - Add tests and documentation
   - No special CI configuration needed

2. **FFI crate**: Add to ffi-tests.yml
   - Create new job with appropriate setup
   - Document environment requirements
   - Add to ci.yml exclusions if needed

### Updating Crates

- Core crates: Run full CI locally before push
- FFI crates: Run FFI tests locally if possible
- Use `cargo fmt --check` before pushing

### Troubleshooting

See `.github/CI_WORKFLOW_GUIDE.md` for detailed troubleshooting.

---

**Last Updated**: 2026-02-10
**Workspace Members**: 13 crates
**CI Workflows**: 3 (main, quality, FFI)
**Status**: Production Ready
