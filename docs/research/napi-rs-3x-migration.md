# napi-rs 3.x Migration Research for mecab-ko-node

**Date**: 2026-04-20
**Branch**: test/ci-validation-news-nnp
**Researcher**: Researcher Agent

---

## Summary

napi-rs 3.x is fully released and actively maintained. The latest stable version is **napi 3.8.4** (2026-03-28), and **@napi-rs/cli 3.6.2** (2026-04-15). The mecab-ko-node crate is a good migration candidate: it does **not** use ThreadsafeFunction, does **not** use the deprecated JsValue low-level APIs, and the `#[napi]` macro-based surface it exposes is largely compatible. The migration is **small effort** with well-defined, mechanical changes.

---

## 1. Release Status

| Package | Current in mecab-ko-node | Latest 3.x | Status |
|---------|--------------------------|------------|--------|
| `napi` | 2.16 | **3.8.4** (2026-03-28) | Stable, actively maintained |
| `napi-derive` | 2.16 | **3.8.4** (2026-03-28) | Stable |
| `napi-build` | 2.1 | **3.0.0-beta.0** (2025-06-04); stable 2.3.1 | 3.x beta only; stable line is 2.x |
| `@napi-rs/cli` | ^2.18.0 | **3.6.2** (2026-04-15) | Stable |

**Key finding**: `napi-build` does not have a stable 3.x release yet (only `3.0.0-beta.0`). The stable napi-build track is `2.3.1`. For Cargo crate upgrades, pin `napi-build = "2.3"` initially.

---

## 2. Breaking Changes: 2.x → 3.x

### 2.1 package.json (HIGH IMPACT — required changes)

| Item | Before (2.x) | After (3.x) |
|------|-------------|-------------|
| Binary name key | `"napi": { "name": "mecab-ko-node" }` | `"napi": { "binaryName": "mecab-ko-node" }` |
| Targets config | `"triples": { "defaults": true, "additional": [...] }` | `"targets": ["x86_64-apple-darwin", ...]` flat array |

Current `package.json` (lines 28-39) uses both deprecated fields and must be updated.

### 2.2 CLI / npm scripts (HIGH IMPACT — required changes)

| Item | Before (2.x) | After (3.x) |
|------|-------------|-------------|
| Dev dependency | `@napi-rs/cli: ^2.18.0` | `@napi-rs/cli: ^3.6.2` |
| Build flag | `--cargo-name mecab-ko-node` | use `--manifest-path` pattern |
| Cargo cwd | `--cargo-cwd` | removed; use `--manifest-path ./Cargo.toml` |
| Cargo flags | `--cargo-flags="..."` | flags go after `--` |
| Dir creation | `napi create-npm-dir` | `napi create-npm-dirs` |
| Universal | `napi universal` | `napi universalize` |

The build script in `package.json` line 42 is:
```
"build": "napi build --platform --release --cargo-name mecab-ko-node"
```
This becomes:
```
"build": "napi build --platform --release --manifest-path ./Cargo.toml"
```

### 2.3 Cargo.toml (LOW IMPACT — version bump only)

No structural changes needed for this crate. The `#[napi]`, `#[napi(object)]`, `#[napi(constructor)]`, `#[napi(factory)]` attributes are stable across 3.x. Only version strings change.

```toml
# Before
napi = "2.16"
napi-derive = "2.16"
napi-build = "2.1"   # build-dependencies

# After
napi = "3"
napi-derive = "3"
napi-build = "2.3"   # stay on stable 2.x until napi-build 3.x stabilizes
```

### 2.4 Rust source code (NO CHANGES REQUIRED for mecab-ko-node)

mecab-ko-node uses only:
- `napi::bindgen_prelude::*` — stable in 3.x
- `napi_derive::napi` attribute macro — stable in 3.x
- `napi::Error::from_reason(...)` — stable in 3.x
- `napi::Result<T>` — stable in 3.x

The crate does **not** use:
- `ThreadsafeFunction` (completely rewritten in 3.x; would require migration)
- `JsObject`, `JsFunction`, `JsNull`, etc. (now behind `compat-mode` feature)
- `napi::module_init` (moved to `napi_derive::module_init`)
- `#[module_exports]` (replaced by `#[napi(module_exports)]`)

No changes to `src/lib.rs` or `build.rs` are required.

### 2.5 Wasm support (OPTIONAL — new feature in 3.x)

napi-rs 3.x adds WebAssembly compilation support targeting `wasm32-wasip1-threads`. This is not required for the migration but is a future opportunity if browser-side tokenization becomes a goal.

---

## 3. File-by-File Analysis

### `rust/crates/mecab-ko-node/Cargo.toml`

Lines requiring change:
- Line 25: `napi = "2.16"` → `napi = "3"`
- Line 26: `napi-derive = "2.16"` → `napi-derive = "3"`
- Line 31: `napi-build = "2.1"` → `napi-build = "2.3"` (stable; 3.x is beta-only)

### `rust/crates/mecab-ko-node/package.json`

Lines requiring change:
- Line 29: `"name": "mecab-ko-node"` → `"binaryName": "mecab-ko-node"`
- Lines 30-39: Replace `"triples"` block with flat `"targets"` array
- Line 42: `napi build --platform --release --cargo-name mecab-ko-node` → update flag syntax
- Line 43: `napi build --platform --cargo-name mecab-ko-node` → update flag syntax
- Line 59: `"@napi-rs/cli": "^2.18.0"` → `"@napi-rs/cli": "^3.6.2"`

### `rust/crates/mecab-ko-node/build.rs`

No changes required. `napi_build::setup()` is the same in 2.x and 3.x.

### `rust/crates/mecab-ko-node/src/lib.rs`

No changes required.

---

## 4. Concrete Migration Plan

### Step 1: Update Cargo.toml

```toml
[dependencies]
napi = "3"
napi-derive = "3"
parking_lot = "0.12"

[build-dependencies]
napi-build = "2.3"
```

### Step 2: Update package.json

```json
"napi": {
  "binaryName": "mecab-ko-node",
  "targets": [
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu"
  ]
},
"scripts": {
  "build": "napi build --platform --release --manifest-path ./Cargo.toml",
  "build:debug": "napi build --platform --manifest-path ./Cargo.toml",
  ...
},
"devDependencies": {
  "@napi-rs/cli": "^3.6.2",
  ...
}
```

### Step 3: Verify build

```bash
cd rust/crates/mecab-ko-node
npm install
cargo build -p mecab-ko-node
npm run build:debug
npm test
```

### Step 4: Run existing tests

The existing Rust unit tests in `src/lib.rs` (lines 313-351) require no modification. Run:

```bash
cargo test -p mecab-ko-node
```

---

## 5. Effort & Risk Assessment

**Effort: SMALL**
- 3 lines changed in `Cargo.toml`
- ~6 lines changed in `package.json`
- 0 lines changed in Rust source
- No API surface changes to callers

**Risk: LOW**
- The `#[napi]` attribute macro API is stable and unchanged between 2.x and 3.x
- mecab-ko-node avoids all deprecated/rewritten subsystems (ThreadsafeFunction, raw JsValue types)
- napi-build stays on the stable 2.x line; the 3.x beta is not needed
- The only non-trivial risk: `@napi-rs/cli` 3.x changed build command syntax; scripts must be updated before attempting a build

**Blockers: NONE**
- napi 3.x is stable and has been released since July 2025
- No Rust source refactoring needed

---

## 6. Version Recommendation

If a full 3.x migration is not desired immediately, the recommended minimum upgrade is:

```toml
napi = "2.16"         # current — acceptable
napi-derive = "2.16"  # current — acceptable
napi-build = "2.3"    # upgrade from 2.1; bug fixes
```

But given the small migration cost, upgrading to 3.x is recommended for access to WebAssembly targets and future-proofing.

---

## References

- [NAPI-RS V2 to V3 Migration Guide](https://napi.rs/docs/more/v2-v3-migration-guide)
- [Announcing NAPI-RS v3](https://napi.rs/blog/announce-v3)
- [napi crate on crates.io](https://crates.io/crates/napi)
- [napi-build on crates.io](https://crates.io/crates/napi-build)
- [napi docs.rs (latest = 3.8.4)](https://docs.rs/crate/napi/latest)
- [@napi-rs/cli changelog](https://napi.rs/changelog/napi-cli)
- [ThreadsafeFunction redesign issue](https://github.com/napi-rs/napi-rs/issues/1220)
- [NAPI-RS ThreadsafeFunction docs (3.x)](https://napi.rs/docs/concepts/threadsafe-function)
