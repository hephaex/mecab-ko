# Release Checklist -- mecab-ko v0.7.2

This document is the authoritative checklist for cutting the v0.7.2 release of the mecab-ko Rust workspace. Every item must be completed (or explicitly marked N/A with justification) before the `v0.7.2` tag is pushed.

---

## Table of Contents

1. [Pre-Release Checks](#1-pre-release-checks)
2. [Version Bumping](#2-version-bumping)
3. [crates.io Publishing Order](#3-cratesio-publishing-order)
4. [npm Publishing (WASM and Node.js)](#4-npm-publishing-wasm-and-nodejs)
5. [PyPI Publishing (Python Bindings)](#5-pypi-publishing-python-bindings)
6. [GitHub Release Creation](#6-github-release-creation)
7. [Post-Release Tasks](#7-post-release-tasks)

---

## 1. Pre-Release Checks

All checks must pass on the `main` branch **before** any version bumps.

### 1.1 Code Quality

- [ ] **Formatting** -- Run `cargo fmt --manifest-path rust/Cargo.toml -- --check` and confirm zero diffs.
- [ ] **Clippy (all features)** -- Run `cargo clippy --manifest-path rust/Cargo.toml --all-targets --all-features -- -D warnings` with no warnings or errors.
- [ ] **Clippy (pedantic/strict)** -- Run with `-W clippy::all -W clippy::pedantic` flags to catch additional issues.
- [ ] **No `unwrap()`/`expect()` in library crates** -- Verify that workspace lint `unwrap_used = "deny"` and `expect_used = "deny"` are enforced. Binding crates (`mecab-ko-python`, `mecab-ko-wasm`, `mecab-ko-node`) are exempt per their local lint overrides.
- [ ] **No `unsafe` in library crates** -- Confirm workspace lint `unsafe_code = "deny"` is active. `mecab-ko-node` is the only crate with `unsafe_code = "allow"` (required for napi FFI).

### 1.2 Test Suite

- [ ] **Unit tests (debug)** -- `cargo test --manifest-path rust/Cargo.toml --verbose` passes on Linux, macOS, and Windows.
- [ ] **Unit tests (release)** -- `cargo test --manifest-path rust/Cargo.toml --release --verbose` passes on all three platforms.
- [ ] **Tests on stable, beta, and nightly toolchains** -- Confirm CI matrix results for all nine combinations (3 OS x 3 toolchains).
- [ ] **WASM tests** -- `wasm-pack test --headless --firefox` passes in `rust/crates/mecab-ko-wasm/`.
- [ ] **Python binding smoke test** -- `python -c "from mecab_ko import Mecab; Mecab().morphs('테스트')"` works after building with `maturin develop`.
- [ ] **Node.js binding smoke test** -- `npm test` passes in `rust/crates/mecab-ko-node/`.
- [ ] **E2E and FFI tests** -- The `e2e-ffi-tests.yml` workflow completes successfully.
- [ ] **Elasticsearch integration tests** -- The `search-plugins.yml` workflow completes successfully.

### 1.3 Documentation

- [ ] **Rustdoc builds without warnings** -- `RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rust/Cargo.toml --no-deps --release` succeeds.
- [ ] **All public APIs have rustdoc comments** -- Workspace lint `missing_docs = "warn"` should report zero warnings.
- [ ] **README files present** -- Each publishable crate directory contains a `README.md`.
- [ ] **CHANGELOG.md updated** -- Move all items from `[Unreleased]` to `[0.7.2] - YYYY-MM-DD` with the actual release date.
- [ ] **mdBook documentation** -- `docs/book/` content is current and builds successfully.
- [ ] **Type stubs (Python)** -- `py.typed` and `__init__.pyi` are present in `rust/crates/mecab-ko-python/python/mecab_ko/`.
- [ ] **TypeScript declarations (Node.js)** -- `index.d.ts` is present and current in `rust/crates/mecab-ko-node/`.
- [ ] **TypeScript declarations (WASM)** -- `index.d.ts` and `mecab_ko_wasm.d.ts` are present and current in `rust/crates/mecab-ko-wasm/`.

### 1.4 Security Audit

- [ ] **cargo audit** -- `cargo audit --manifest-path rust/Cargo.toml` reports no actionable vulnerabilities. Known advisory exceptions:
  - None. bincode was removed from the dependency graph in v0.7.x; `RUSTSEC-2025-0141` no longer applies.
- [ ] **cargo deny** -- `cargo deny --manifest-path rust/Cargo.toml check all` passes (advisories, licenses, bans, sources).
- [ ] **cargo geiger** -- `cargo geiger --manifest-path rust/Cargo.toml` output reviewed; no unexpected `unsafe` in first-party code.
- [ ] **SBOM generated** -- `cargo sbom --manifest-path rust/Cargo.toml > sbom.json` produced and archived.
- [ ] **License compliance** -- All dependencies use licenses compatible with `MIT OR Apache-2.0`.
- [ ] **Dependency versions pinned** -- `Cargo.lock` is committed and up to date.

### 1.5 Performance Validation

- [ ] **Benchmarks run** -- Criterion benchmarks execute without regressions compared to the previous development baseline.
- [ ] **No cold-start regressions** -- `cold_start_bench` results are acceptable.
- [ ] **Memory usage acceptable** -- `memory_bench` results are within expected bounds.

---

## 2. Version Bumping

All crates in the workspace use `version.workspace = true`, inheriting from the workspace `Cargo.toml`. The single source of truth for the version is:

**File:** `rust/Cargo.toml` (workspace root)

```toml
[workspace.package]
version = "0.7.2"
```

### 2.1 Cargo Workspace Version

- [ ] Confirm `rust/Cargo.toml` has `version = "0.7.2"` under `[workspace.package]`.
- [ ] Confirm all 11 publishable crates inherit the workspace version (`version.workspace = true`):

| Crate | Cargo.toml Path |
|-------|----------------|
| `mecab-ko-hangul` | `rust/crates/mecab-ko-hangul/Cargo.toml` |
| `mecab-ko-dict` | `rust/crates/mecab-ko-dict/Cargo.toml` |
| `mecab-ko-dict-builder` | `rust/crates/mecab-ko-dict-builder/Cargo.toml` |
| `mecab-ko-dict-validator` | `rust/crates/mecab-ko-dict-validator/Cargo.toml` |
| `mecab-ko-core` | `rust/crates/mecab-ko-core/Cargo.toml` |
| `mecab-ko` | `rust/crates/mecab-ko/Cargo.toml` |
| `mecab-ko-cli` | `rust/crates/mecab-ko-cli/Cargo.toml` |
| `mecab-ko-elasticsearch` | `rust/crates/mecab-ko-elasticsearch/Cargo.toml` |
| `mecab-ko-python` | `rust/crates/mecab-ko-python/Cargo.toml` |
| `mecab-ko-wasm` | `rust/crates/mecab-ko-wasm/Cargo.toml` |
| `mecab-ko-node` | `rust/crates/mecab-ko-node/Cargo.toml` |

- [ ] Confirm `mecab-ko-benchmarks` has `publish = false` and does **not** need a version bump.

### 2.2 Inter-Crate Dependency Versions

Path dependencies also specify a version for crates.io resolution. Verify these match `0.7.2`:

- [ ] `mecab-ko-dict` depends on `mecab-ko-hangul = { version = "0.7.2", path = "..." }`
- [ ] `mecab-ko-core` depends on `mecab-ko-hangul = { version = "0.7.2", path = "..." }`
- [ ] `mecab-ko-core` depends on `mecab-ko-dict = { version = "0.7.2", path = "..." }`
- [ ] `mecab-ko-dict-validator` depends on `mecab-ko-dict = { version = "0.7.2", path = "..." }`
- [ ] `mecab-ko-dict-builder` depends on `mecab-ko-hangul = { version = "0.7.2", path = "..." }`
- [ ] `mecab-ko-dict-builder` depends on `mecab-ko-dict = { version = "0.7.2", path = "..." }`
- [ ] `mecab-ko-dict-sync` depends on `mecab-ko-dict = { version = "0.7.2", path = "..." }`
- [ ] `mecab-ko` depends on `mecab-ko-core = { version = "0.7.2", path = "..." }`
- [ ] `mecab-ko` depends on `mecab-ko-dict = { version = "0.7.2", path = "..." }`
- [ ] `mecab-ko` depends on `mecab-ko-hangul = { version = "0.7.2", path = "..." }`

### 2.3 Non-Rust Package Versions

- [ ] **Python** -- `rust/crates/mecab-ko-python/pyproject.toml` has `version = "0.7.2"`.
- [ ] **Node.js** -- `rust/crates/mecab-ko-node/package.json` has `"version": "0.7.2"`.
- [ ] **WASM** -- `rust/crates/mecab-ko-wasm/package.json` has `"version": "0.7.2"`.

### 2.4 Final Version Commit

- [ ] Create a single commit with message: `chore: release v0.7.2`
- [ ] Ensure the commit includes the updated `CHANGELOG.md` with the release date.

---

## 3. crates.io Publishing Order

Crates must be published in strict dependency order. Wait for each crate to appear on crates.io (typically 30-60 seconds) before publishing the next one. The `--allow-dirty` flag should **not** be used in a manual release; ensure the working tree is clean.

### Dependency Graph

```
mecab-ko-hangul          (leaf -- no internal deps)
       |
       v
mecab-ko-dict            (depends on: mecab-ko-hangul)
       |
       v
mecab-ko-core            (depends on: mecab-ko-hangul, mecab-ko-dict)
       |
       v
mecab-ko-dict-validator  (depends on: mecab-ko-dict)
       |
       v
mecab-ko-dict-builder    (depends on: mecab-ko-hangul, mecab-ko-dict)
       |
       v
mecab-ko-dict-sync       (depends on: mecab-ko-dict)
       |
       v
mecab-ko                 (depends on: mecab-ko-core, mecab-ko-dict, mecab-ko-hangul)
```

### Publishing Steps

Run each command from the repository root. Replace `$TOKEN` with your crates.io API token or use `cargo login` beforehand.

```bash
# Step 1: Leaf crate (no dependencies)
cargo publish --manifest-path rust/crates/mecab-ko-hangul/Cargo.toml
# Wait ~60s for crates.io index to update

# Step 2: Dictionary crate
cargo publish --manifest-path rust/crates/mecab-ko-dict/Cargo.toml
# Wait ~60s

# Step 3: Core engine
cargo publish --manifest-path rust/crates/mecab-ko-core/Cargo.toml
# Wait ~60s

# Step 4: Dictionary validator (depends on mecab-ko-dict)
cargo publish --manifest-path rust/crates/mecab-ko-dict-validator/Cargo.toml
# Wait ~60s

# Step 5: Dictionary builder (depends on mecab-ko-hangul, mecab-ko-dict)
cargo publish --manifest-path rust/crates/mecab-ko-dict-builder/Cargo.toml
# Wait ~60s

# Step 6: Dictionary sync (depends on mecab-ko-dict)
cargo publish --manifest-path rust/crates/mecab-ko-dict-sync/Cargo.toml
# Wait ~60s

# Step 7: Main library crate
cargo publish --manifest-path rust/crates/mecab-ko/Cargo.toml
# Wait ~60s

# Step 8: Consumer crates (parallel -- all depend on mecab-ko-core)
cargo publish --manifest-path rust/crates/mecab-ko-cli/Cargo.toml
cargo publish --manifest-path rust/crates/mecab-ko-elasticsearch/Cargo.toml
cargo publish --manifest-path rust/crates/mecab-ko-profiler/Cargo.toml
```

**Note:** The following crates are **not published to crates.io**:

| Crate | Reason |
|-------|--------|
| `mecab-ko-python` | Published to PyPI as a Python wheel (cdylib only) |
| `mecab-ko-wasm` | Published to npm as a WASM package (cdylib only) |
| `mecab-ko-node` | Published to npm as a native addon (cdylib only) |
| `mecab-ko-benchmarks` | Internal benchmarks (`publish = false`) |

### Verification

- [ ] After all publishes, verify each crate at `https://crates.io/crates/<name>`:
  - [ ] `mecab-ko-hangul` -- https://crates.io/crates/mecab-ko-hangul
  - [ ] `mecab-ko-dict` -- https://crates.io/crates/mecab-ko-dict
  - [ ] `mecab-ko-core` -- https://crates.io/crates/mecab-ko-core
  - [ ] `mecab-ko-dict-validator` -- https://crates.io/crates/mecab-ko-dict-validator
  - [ ] `mecab-ko-dict-builder` -- https://crates.io/crates/mecab-ko-dict-builder
  - [ ] `mecab-ko-dict-sync` -- https://crates.io/crates/mecab-ko-dict-sync
  - [ ] `mecab-ko` -- https://crates.io/crates/mecab-ko
  - [ ] `mecab-ko-cli` -- https://crates.io/crates/mecab-ko-cli
  - [ ] `mecab-ko-elasticsearch` -- https://crates.io/crates/mecab-ko-elasticsearch
  - [ ] `mecab-ko-profiler` -- https://crates.io/crates/mecab-ko-profiler
- [ ] Verify docs.rs builds for each published crate.
- [ ] Test installation from crates.io: `cargo install mecab-ko-cli` in a clean environment.

---

## 4. npm Publishing (WASM and Node.js)

### 4.1 Prerequisites

- [ ] `npm` CLI authenticated (`npm login` or `NPM_TOKEN` configured).
- [ ] npm organization `@mecab-ko` is created (for the scoped Node.js package).
- [ ] `wasm-pack` is installed (`cargo install wasm-pack`).
- [ ] `@napi-rs/cli` is installed in the Node.js crate (`npm install` in `rust/crates/mecab-ko-node/`).

### 4.2 WASM Package (`mecab-ko-wasm`)

The WASM package is published as an unscoped npm package.

```bash
cd rust/crates/mecab-ko-wasm

# Build for bundler (default target)
wasm-pack build --target bundler --out-dir pkg --release

# Build for Node.js
wasm-pack build --target nodejs --out-dir pkg-node --release

# Build for web (no bundler)
wasm-pack build --target web --out-dir pkg-web --release

# Verify package contents
ls -lh pkg/

# Publish the bundler package
cd pkg
npm publish
```

- [ ] Verify `mecab-ko-wasm@0.7.2` is available at https://www.npmjs.com/package/mecab-ko-wasm
- [ ] Test installation: `npm install mecab-ko-wasm` in a fresh project.
- [ ] Verify WASM module loads in browser and Node.js environments.

### 4.3 Node.js Package (`@mecab-ko/node`)

The Node.js package is published as a scoped package with public access.

```bash
cd rust/crates/mecab-ko-node

# Install dependencies
npm install

# Build native bindings for the current platform
npm run build

# Run tests
npm test

# Prepare platform-specific artifacts
npm run prepublishOnly

# Publish
npm publish --access public
```

For multi-platform pre-built binaries, the CI release workflow builds for these targets:
- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`

- [ ] Verify `@mecab-ko/node@0.7.2` is available at https://www.npmjs.com/package/@mecab-ko/node
- [ ] Test installation on Linux x86_64: `npm install @mecab-ko/node`
- [ ] Test installation on macOS ARM64: `npm install @mecab-ko/node`
- [ ] Verify TypeScript types resolve correctly.

---

## 5. PyPI Publishing (Python Bindings)

### 5.1 Prerequisites

- [ ] `maturin` is installed (`pip install maturin`).
- [ ] PyPI API token is configured (either via `~/.pypirc`, `MATURIN_PYPI_TOKEN`, or GitHub Actions secret `PYPI_API_TOKEN`).
- [ ] TestPyPI has been tested with a pre-release upload.

### 5.2 Automated Publishing (Recommended)

The `python-wheels.yml` workflow triggers automatically on tag push (`v*`). It performs:

1. Multi-platform wheel builds (Linux x86_64/aarch64, macOS x86_64/ARM64, Windows x86_64).
2. Source distribution (sdist) build.
3. Wheel testing across Python 3.8-3.12 on all platforms.
4. Upload to PyPI using trusted publishing.
5. Post-publish verification from PyPI.

- [ ] Verify GitHub Actions secret for PyPI trusted publishing is configured.
- [ ] Confirm the `pypi` environment is set up in the repository settings with the PyPI project URL.

### 5.3 Manual Publishing (Fallback)

If the automated workflow fails, publish manually:

```bash
cd rust/crates/mecab-ko-python

# Test on TestPyPI first
maturin publish --repository testpypi

# Verify TestPyPI installation
pip install --index-url https://test.pypi.org/simple/ mecab-ko-python

# Publish to production PyPI
maturin publish
```

### 5.4 Verification

- [ ] Verify `mecab-ko-python==0.7.2` is available at https://pypi.org/project/mecab-ko-python/
- [ ] Test installation in a clean virtual environment:
  ```bash
  python -m venv /tmp/mecab-test && source /tmp/mecab-test/bin/activate
  pip install mecab-ko-python
  python -c "from mecab_ko import Mecab; print(Mecab().morphs('한국어 형태소 분석'))"
  ```
- [ ] Verify wheel availability for all target platforms (check "Download files" on PyPI).
- [ ] Verify Python version compatibility (3.8 through 3.13).
- [ ] Confirm `pip show mecab-ko-python` displays correct metadata (version, author, license).

---

## 6. GitHub Release Creation

### 6.1 Tag Creation

```bash
# Ensure you are on main and up to date
git checkout main
git pull origin main

# Create annotated tag
git tag -a v0.7.2 -m "Release v0.7.2"

# Push tag (triggers release.yml and python-wheels.yml workflows)
git push origin v0.7.2
```

### 6.2 Automated Release (via `release.yml`)

The `release.yml` workflow performs the following on tag push:

1. Creates a GitHub Release with changelog body.
2. Builds CLI binaries for five targets:
   - `x86_64-unknown-linux-gnu` (tar.gz)
   - `aarch64-unknown-linux-gnu` (tar.gz)
   - `x86_64-apple-darwin` (tar.gz)
   - `aarch64-apple-darwin` (tar.gz)
   - `x86_64-pc-windows-msvc` (zip)
3. Uploads binaries as release assets.
4. Triggers crates.io publish (for non-prerelease tags).

### 6.3 Release Notes Content

The release notes should include:

- [ ] **Summary** -- Brief description of what mecab-ko v0.7.2 provides.
- [ ] **Highlights** -- Key features (Viterbi engine, Hangul utilities, dictionary management, multi-platform bindings).
- [ ] **Installation instructions** -- For each ecosystem (Rust, Python, Node.js, WASM).
- [ ] **Breaking changes** -- Document any API changes since v0.7.1.
- [ ] **Known issues** -- Document any known limitations or open issues.
- [ ] **Full changelog link** -- Link to `CHANGELOG.md` or the diff since repository creation.

### 6.4 Release Asset Verification

- [ ] All five binary archives are attached to the release.
- [ ] Linux x86_64 binary runs: `./mecab-ko --version` outputs `0.7.2`.
- [ ] macOS ARM64 binary runs correctly.
- [ ] Windows binary runs correctly.
- [ ] Release is **not** marked as pre-release (unless intentionally doing an alpha/beta/RC).

---

## 7. Post-Release Tasks

### 7.1 Immediate (within 24 hours)

- [ ] **Verify all package registries** -- Confirm packages are installable from crates.io, PyPI, and npm.
- [ ] **Verify docs.rs** -- All crate documentation pages render correctly.
- [ ] **Verify GitHub Pages** -- Documentation site (mdBook + rustdoc) is deployed and accessible.
- [ ] **Monitor CI** -- Ensure no regressions on the main branch after the release commit.
- [ ] **Test cross-ecosystem integration** -- Install from each registry in a clean environment and run a basic tokenization.

### 7.2 Communication

- [ ] **Update repository description** -- Ensure the GitHub repository description and topics reflect the released state.
- [ ] **Update README badges** -- Add/verify crates.io version badge, docs.rs badge, PyPI badge, npm badge, and CI status badge.
- [ ] **Announce release** -- Post to relevant channels:
  - [ ] GitHub Discussions / Release announcement
  - [ ] Rust community (if applicable): r/rust, Rust users forum
  - [ ] Korean NLP community channels
  - [ ] Python/Node.js ecosystem channels (if applicable)

### 7.3 Prepare for Next Development Cycle

- [ ] **Bump workspace version** -- Update `rust/Cargo.toml` to `version = "0.7.3-dev"` (or `0.8.0-dev` if the next release is a minor bump).
- [ ] **Update non-Rust versions** -- Bump `pyproject.toml`, `package.json` files to match.
- [ ] **Reset CHANGELOG.md** -- Add a new `[Unreleased]` section above the `[0.7.2]` entry.
- [ ] **Update comparison links** -- Add `[0.7.2]: https://github.com/hephaex/mecab-ko/releases/tag/v0.7.2` to CHANGELOG footer.
- [ ] **Create v0.7.3 milestone** (or v0.8.0) -- Set up a GitHub milestone for the next release with the planned roadmap items.
- [ ] **Review Dependabot PRs** -- Merge any pending dependency updates that were deferred for the release.
- [ ] **Archive release branch** -- If a release branch was used, merge it back to main and delete it.

### 7.4 Monitoring (first week)

- [ ] **Track download counts** -- Monitor crates.io, PyPI, and npm download metrics.
- [ ] **Watch for issue reports** -- Triage any bug reports related to the v0.7.2 release.
- [ ] **Check daily security audit** -- The `scheduled.yml` workflow runs daily; ensure no new advisories affect v0.7.2.
- [ ] **Verify Dependabot** -- Confirm automated dependency update PRs are being created.

---

## Quick Reference: Required Secrets and Tokens

| Secret | Where Used | Purpose |
|--------|-----------|---------|
| `CARGO_REGISTRY_TOKEN` | `release.yml` | crates.io API token |
| `GITHUB_TOKEN` | `release.yml`, `python-wheels.yml` | GitHub Releases, tag operations |
| `NPM_TOKEN` | npm publish step | npm registry authentication |
| PyPI trusted publishing | `python-wheels.yml` | PyPI OIDC-based publishing (no token needed if configured) |
| `CODECOV_TOKEN` | `ci.yml` | Code coverage upload |

---

## Quick Reference: Key File Paths

| File | Purpose |
|------|---------|
| `rust/Cargo.toml` | Workspace version (single source of truth) |
| `CHANGELOG.md` | Release notes source |
| `rust/crates/mecab-ko-python/pyproject.toml` | Python package metadata |
| `rust/crates/mecab-ko-node/package.json` | Node.js package metadata |
| `rust/crates/mecab-ko-wasm/package.json` | WASM package metadata |
| `.github/workflows/release.yml` | GitHub Release + crates.io workflow |
| `.github/workflows/python-wheels.yml` | PyPI multi-platform wheel build and publish |
| `.github/workflows/ci.yml` | Full CI pipeline (tests, lint, coverage, audit) |
| `.github/workflows/security.yml` | Security scanning (audit, deny, geiger, SAST) |
