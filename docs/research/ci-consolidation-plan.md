# GitHub Actions Workflows Consolidation Plan

**Document Date**: 2026-04-21  
**Project**: mecab-ko (Rust Korean Morphological Analyzer)  
**Current State**: 23 workflows, 7136 lines of YAML  
**Objective**: Consolidate workflows into a maintainable set using reusable workflows and composition patterns

> **Note (2026-04-23):** This analysis was from Sprint 72-73. Since then: `elasticsearch-plugin-tests.yml` deleted (S77), `pypi-publish.yml` merged into `python-wheels.yml` (S77), `npm-publish-wasm.yml` merged into `npm-publish.yml` (S78), `e2e-tests.yml` + `ffi-tests.yml` merged into `e2e-ffi-tests.yml` (S79). Current workflow count: 16.

---

## Executive Summary

The mecab-ko project currently maintains 23 GitHub Actions workflow files with significant duplication and overlapping functionality. This analysis identifies opportunities to consolidate these workflows into ~6-7 core workflows using GitHub's `workflow_call` reusable workflow pattern, reducing maintenance burden by 65-70% while improving clarity and consistency.

### Key Findings

- **Rust setup duplication**: 8 workflows independently manage Rust toolchain + cargo cache
- **Dict validation duplicates**: 3 separate dict validation workflows with identical CSV parsing logic
- **Dead workflows**: `dependabot.yml` may be obsolete if GitHub's native Dependabot is already configured
- **Unrelated stacks**: `search-plugins.yml` uses Java/Gradle (separate from Rust), should remain isolated
- **Publishing pipeline fragmentation**: npm, PyPI, and release workflows could benefit from unified artifact handling

### Proposed Target Architecture

```
Reusable Workflows (_reusable-*.yml)
├── _reusable-rust-setup.yml              # Toolchain + cache (shared by all Rust jobs)
├── _reusable-dict-setup.yml              # Dict download + validation (shared by dict jobs)
├── _reusable-publish-artifact.yml        # Generic artifact publishing (npm/PyPI/release)

Primary Workflows
├── ci.yml                                # Main: build + test + clippy + security audit
├── publish.yml                           # Unified: npm + PyPI + release (triggered by tags)
├── scheduled.yml                         # Periodic: security + weekly comprehensive tests
├── search-plugins.yml                    # Keep isolated (Java/Gradle stack)

Dictionary Workflows
├── dict-build.yml                        # Enhanced: integrated validation + tokenize test
├── accuracy-gate.yml                     # Keep as-is (specialized for PR validation)

Specialized Workflows
├── docs.yml                              # Documentation build (kept as-is)
├── e2e-tests.yml                         # E2E across platforms (kept as-is)
├── docker.yml                            # Docker build (kept as-is or reduce)
```

---

## Current Workflow Inventory

### Category: Core CI/Build (3 workflows)

| Workflow | Trigger | Purpose | Overlap | Lines | Priority |
|----------|---------|---------|---------|-------|----------|
| **ci.yml** | push/PR on rust/ | Main: fmt, clippy, test (stable/beta/nightly), profiler, docs, multi-platform build, audit, coverage | core_ci | 308 | KEEP |
| **code-quality.yml** | push/PR + daily 3am UTC | Unused deps, outdated deps, code metrics, docs check | fmt/clippy/audit duplicates | 177 | MERGE into ci.yml |
| **security.yml** | push/PR + daily 2am UTC | RustSec, cargo-audit, cargo-deny, unsafe-code-check, clippy-strict | audit duplicates | 197 | MERGE into ci.yml or keep separate for daily schedule |

**Overlap Analysis**:
- `ci.yml` already runs security-audit (RustSec + cargo-audit)
- `code-quality.yml` repeats some Rust setup and cargo operations
- `security.yml` overlaps with `ci.yml` on audit steps

**Recommendation**: Merge code-quality checks into `ci.yml` as optional jobs. Keep scheduled security checks in a separate lightweight workflow.

---

### Category: Testing - Specialized (4 workflows)

| Workflow | Trigger | Purpose | Stack | Overlap | Lines | Priority |
|----------|---------|---------|-------|---------|-------|----------|
| **accuracy-gate.yml** | PR on rust/data/eval | Token accuracy evaluation (95%+ gate) | Rust | None | KEEP |
| **e2e-tests.yml** | push/PR on main/develop | Multi-platform CLI/Python/Node/WASM tests | Multi-stack | None | KEEP |
| **ffi-tests.yml** | push/PR on rust/crates/mecab-ko-* | Python, Node, WASM, Elasticsearch bindings | Python/Node/WASM/Java | Some Rust setup | 231 | MERGE Rust setup |
| **elasticsearch-plugin-tests.yml** | push/PR on elasticsearch-plugin/rust | ES plugin: unit + integration + compatibility + perf + Docker | Java/Gradle | Rust build duplication | 301 | CONSIDER MERGE with search-plugins.yml |

**Overlap Analysis**:
- `ffi-tests.yml` and `e2e-tests.yml` both test Python/Node/WASM with independent Rust setup
- Elasticsearch plugin tests duplicate across two files
- All repeat Rust toolchain installation

**Recommendation**: 
- Keep `accuracy-gate.yml` (specialized constraint validation)
- Keep `e2e-tests.yml` (critical user flows, end-to-end)
- Merge `ffi-tests.yml` into `e2e-tests.yml` (both are binding tests)
- Move Elasticsearch plugin tests to `search-plugins.yml` (consolidate Java ecosystem)

---

### Category: Dictionary Build (3 workflows)

| Workflow | Trigger | Purpose | Overlap | Lines | Priority |
|----------|---------|---------|---------|-------|----------|
| **dict-build.yml** | push on data/user-dict/, dict-builder, Cargo.lock + manual | User dict validation → build → tokenize test → accuracy eval → report + cleanup | core_dict | 685 | KEEP & enhance |
| **validate-domain-dict.yml** | push/PR on data/domain-dic/ | Domain-specific CSV validation (news/government NNP-only) | CSV validation duplicates | 182 | MERGE into dict-build.yml with domain flag |
| **neologism-sync.yml** | weekly/monthly schedule + manual | OpenDict API sync → filter → merge → create PR | Neologism collection only | 595 | KEEP (scheduled, independent) |

**Overlap Analysis**:
- CSV validation logic repeated in `dict-build.yml` and `validate-domain-dict.yml`
- Both validate POS tags, check duplicates, validate encoding
- `neologism-sync.yml` is independent but could share validation code via reusable workflow

**Recommendation**:
- Enhance `dict-build.yml` with domain dictionary support (conditional path handling)
- Create `_reusable-dict-validation.yml` shared by both
- Keep `neologism-sync.yml` (separate trigger schedule and purpose)

---

### Category: Publishing (4 workflows)

| Workflow | Trigger | Purpose | Platform | Overlap | Lines | Priority |
|----------|---------|---------|----------|---------|-------|----------|
| **npm-publish.yml** | tag npm-v* + manual dry-run | Build multi-platform native bindings (NAPI), test, publish to npm | Node.js (NAPI) | Artifact handling | 237 | CONSOLIDATE |
| **npm-publish-wasm.yml** | tag v* + manual dry-run | Build WASM (bundler/nodejs/web targets), test, publish to npm | WASM | Artifact handling | 233 | CONSOLIDATE |
| **pypi-publish.yml** | tag v* + manual test_pypi | Build wheels (manylinux/macOS/Windows), test, publish to PyPI | Python | Artifact handling | 313 | CONSOLIDATE |
| **python-wheels.yml** | release + push main + manual | Build wheels + sdist, test, verify, publish to PyPI, update release | Python (alternative) | DUPLICATE with pypi-publish.yml | 505 | MERGE |
| **release.yml** | tag v* + manual | Create GitHub Release, build CLI binaries (5 platforms), publish to crates.io | Rust CLI | Artifact handling | 171 | CONSOLIDATE |

**Overlap Analysis**:
- `pypi-publish.yml` and `python-wheels.yml` are DUPLICATE pipelines (both build wheels)
- All publishing workflows repeat: artifact download, platform selection, publish with similar patterns
- npm/WASM/PyPI all use nearly identical artifact → publish flow

**Recommendation**: 
- **CRITICAL**: Consolidate `pypi-publish.yml` + `python-wheels.yml` (determine which is primary, delete one)
- Create `publish.yml` with conditional jobs:
  - `publish-npm-node` (triggered by npm-v* tags)
  - `publish-npm-wasm` (triggered by v* tags)
  - `publish-pypi` (triggered by v* tags + release event)
  - `publish-release` (triggered by v* tags)
- Create `_reusable-publish-artifact.yml` to handle: download, verify, publish to registry

---

### Category: Maintenance & Scheduled (5 workflows)

| Workflow | Trigger | Purpose | Overlap | Lines | Priority |
|----------|---------|---------|---------|-------|----------|
| **benchmark.yml** | push/PR on rust/benches/ + weekly Sun 00:00 UTC | Compile check, run benches, compare (PR only), regression alert (scheduled), update dashboard | benchmark_only | 544 | KEEP |
| **scheduled.yml** | daily 00:00 UTC + weekly 02:00 UTC + manual | Daily security audit + weekly dependency check + weekly comprehensive test + cleanup artifacts + health check | cross-cutting | 190 | REFACTOR |
| **neologism-multi-source.yml** | weekly Mon 00:00 UTC + manual | Multi-source collection (OpenDict, Baram, corpus, wiki) → merge → create issue | collection_only | 591 | KEEP (independent) |
| **docs.yml** | push/PR on docs/ + manual | Build rustdoc + mdBook, combine, deploy to GitHub Pages | docs_only | 351 | KEEP (self-contained) |
| **dependabot.yml** | PR opened (Dependabot) | Auto-merge + approve Dependabot PRs | dead? | 33 | VERIFY & REMOVE (likely redundant) |

**Overlap Analysis**:
- `scheduled.yml` mixes unrelated concerns (security, dependencies, testing, cleanup, status)
- `benchmark.yml` is self-contained, no duplication
- `dependabot.yml` appears to be dead if GitHub native Dependabot is handling merges

**Recommendation**:
- Keep `benchmark.yml` as-is (specialized performance tracking)
- Split `scheduled.yml` into focused workflows:
  - `security-scheduled.yml` (daily RustSec, weekly cargo-deny)
  - `maintenance.yml` (weekly dependency checks, artifact cleanup)
- Remove `dependabot.yml` after verifying GitHub's native Dependabot is active
- Keep `neologism-multi-source.yml` and `docs.yml` as independent specializations

---

### Category: Docker & Search Plugins (2 workflows)

| Workflow | Trigger | Purpose | Stack | Overlap | Lines | Priority |
|----------|---------|---------|-------|---------|-------|----------|
| **docker.yml** | push to main + tag v* + PR + manual | Build + push multi-platform Docker image (QEMU) + attestation | Docker/OCI | None | KEEP or REDUCE |
| **search-plugins.yml** | push/PR on search-plugins/ + manual | Build native libs (5 platforms), ES/OpenSearch plugins, integration tests (ES versions + OpenSearch), Docker test, release | Java/Gradle/Rust | None | KEEP (different stack) |

**Overlap Analysis**:
- Docker and search-plugins are sufficiently different (different stacks)
- search-plugins uses Gradle for Java/Elasticsearch ecosystem
- No overlap with Rust workflows

**Recommendation**:
- Keep `docker.yml` for main application image
- Keep `search-plugins.yml` isolated (Java/Gradle ecosystem is separate)

---

## Proposed Consolidation Strategy

### Phase 1: Create Reusable Workflows (Low Risk)

**1. `_reusable-rust-setup.yml`** (New)
```yaml
name: Rust Setup
on:
  workflow_call:
    inputs:
      toolchain:
        type: string
        default: 'stable'
      components:
        type: string
        default: ''
      targets:
        type: string
        default: ''
      cache-key:
        type: string
        default: 'cargo-default'

jobs:
  setup:
    runs-on: ubuntu-latest
    steps:
      - uses: dtolnay/rust-toolchain@master
      - uses: Swatinem/rust-cache@v2
```

**Benefits**: 
- Eliminates 8 independent Rust setup blocks
- Single source of truth for toolchain version management
- Consistent cache strategy across all workflows

**2. `_reusable-dict-validation.yml`** (New)
```yaml
name: Dictionary Validation
on:
  workflow_call:
    inputs:
      csv_pattern:
        type: string
      expected_fields:
        type: string
      dict_type:
        type: string
        default: 'default'

jobs:
  validate:
    # Shared validation logic for all dictionary workflows
```

**Benefits**:
- Single CSV validation implementation
- Supports domain-specific field counts
- Reusable across dict-build, validate-domain-dict, neologism-sync

**3. `_reusable-publish-artifact.yml`** (New)
```yaml
name: Publish Artifact
on:
  workflow_call:
    inputs:
      registry:
        type: string  # npm, pypi, crates
      package:
        type: string
      version:
        type: string

jobs:
  publish:
    # Generic publish logic with registry selection
```

**Benefits**:
- Single publish workflow supports npm, PyPI, crates.io
- Consistent artifact verification and publishing
- Easier to add new registries

---

### Phase 2: Consolidate Primary Workflows (Medium Risk)

**1. Merge `code-quality.yml` into `ci.yml`**
- Add conditional jobs for quality checks (optional on PR, mandatory on main)
- Keep existing ci.yml structure but add:
  - `unused-dependencies` (from code-quality)
  - `dependency-outdated` (from code-quality)
  - `code-metrics` (from code-quality)
  - `docs-check` (from code-quality)
- Result: Single comprehensive CI workflow

**2. Refactor `scheduled.yml` → `security-scheduled.yml` + `maintenance.yml`**
- `security-scheduled.yml`: daily 00:00 UTC (RustSec, cargo-deny)
- `maintenance.yml`: weekly 02:00 UTC (dependency checks, cleanup, health check)
- Separation of concerns improves clarity

**3. Create unified `publish.yml`**
- Consolidate npm-publish, npm-publish-wasm, pypi-publish, release
- Use conditional job execution based on tag pattern:
  - `npm-v*` → publish npm packages
  - `v*` (not npm-v*, not pre-release) → publish PyPI + crates.io + release
  - `v*-alpha/beta/rc` → prerelease handling
- Reuse `_reusable-publish-artifact.yml` for all registries

**4. Decide: `pypi-publish.yml` vs `python-wheels.yml`**
- **BOTH DUPLICATE**: Both build wheels, run tests, publish to PyPI
- **Action**: Analyze git history to determine which is primary
- **Recommendation**: Keep pypi-publish.yml (cleaner structure), delete python-wheels.yml
  - Move any unique logic (wheel report, commit comment) into pypi-publish or shared workflow

---

### Phase 3: Consolidate Testing Workflows (Medium-High Risk)

**1. Merge `ffi-tests.yml` into `e2e-tests.yml`**
- `ffi-tests.yml` tests: Python, Node, WASM, Elasticsearch bindings
- `e2e-tests.yml` tests: CLI, Python, Node, WASM
- Both are integration tests across platforms
- Consolidate into single e2e workflow with:
  - `cli-tests` (existing)
  - `python-tests` (existing + FFI)
  - `nodejs-tests` (existing + FFI)
  - `wasm-tests` (existing)
  - `elasticsearch-plugin-tests` (from ffi-tests)

**2. Consolidate Elasticsearch plugin tests**
- Move Elasticsearch binding tests from ffi-tests.yml into search-plugins.yml
- search-plugins.yml already tests ES plugin, so consolidate ES-related tests there
- Keeps Java/Gradle ecosystem together

---

### Phase 4: Verify & Remove Dead Workflows (Low Risk)

**1. `dependabot.yml`**
- **Status**: Likely redundant if GitHub's native Dependabot auto-merge is enabled
- **Action**:
  1. Check Settings > Code security and analysis > Dependabot settings
  2. Verify if "Auto-merge" is configured for Dependabot PRs
  3. If yes, **delete dependabot.yml**
  4. If no, keep it

---

## Risk Assessment & Mitigation

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|-----------|
| **Breaking CI during consolidation** | HIGH | MEDIUM | Test reusable workflows in separate branch before merge. Run CI against test branch for 1 week. |
| **Lost job parallelization** | MEDIUM | LOW | Verify job dependencies aren't introduced. Benchmark build time before/after. |
| **Unclear workflow trigger conditions** | MEDIUM | MEDIUM | Add comprehensive comments to consolidated workflows. Document tag/branch patterns. |
| **Duplicate pypi publish definitions** | MEDIUM | HIGH | Audit both workflows thoroughly. Choose primary. Test publish to TestPyPI before production. |
| **Regression in dict validation logic** | HIGH | MEDIUM | Validate all domain dict formats with reusable workflow. Run on sample CSVs first. |
| **Loss of emergency manual triggers** | LOW | LOW | Document manual trigger commands in workflow comments. Test manual dispatch before removing old workflows. |

---

## Implementation Roadmap

### Week 1-2: Planning & Validation
- [ ] Verify Dependabot status (check if native auto-merge is active)
- [ ] Confirm which pypi publish workflow is primary (audit git history)
- [ ] Test reusable workflow syntax in feature branch
- [ ] Get team approval on consolidation plan

### Week 3-4: Phase 1 - Reusable Workflows
- [ ] Create `_reusable-rust-setup.yml`
- [ ] Test rust-setup in 2-3 workflows
- [ ] Create `_reusable-dict-validation.yml`
- [ ] Create `_reusable-publish-artifact.yml`
- [ ] Verify all reusable workflows work correctly

### Week 5-6: Phase 2 - Primary Consolidation
- [ ] Merge code-quality → ci.yml
- [ ] Test consolidated ci.yml
- [ ] Refactor scheduled.yml → security-scheduled + maintenance
- [ ] Create unified publish.yml
- [ ] Test publish.yml with dry-run triggers

### Week 7-8: Phase 3 - Testing Consolidation
- [ ] Merge ffi-tests → e2e-tests (or keep separate if too complex)
- [ ] Move ES plugin tests to search-plugins
- [ ] Comprehensive E2E testing on all platforms
- [ ] Verify no regression in test coverage

### Week 9-10: Phase 4 - Cleanup
- [ ] Delete deprecated workflows
- [ ] Delete python-wheels.yml (if consolidating into pypi-publish)
- [ ] Delete dependabot.yml (if confirmed redundant)
- [ ] Update workflow documentation

### Week 11-12: Stabilization
- [ ] Monitor CI stability for 2 weeks
- [ ] Fix any issues discovered
- [ ] Document final workflow architecture
- [ ] Prepare lessons learned

---

## Migration Checklist

Before running consolidated workflows in production:

### Testing Phase (In Feature Branch)
- [ ] All reusable workflows compile without syntax errors
- [ ] Each reusable workflow tested with at least one caller
- [ ] ci.yml runs successfully with new setup reusable workflow
- [ ] Dict validation works with both default and domain CSVs
- [ ] publish.yml successfully publishes to TestPyPI and test npm registry

### Documentation Phase
- [ ] Update `.github/WORKFLOWS.md` with new architecture
- [ ] Add comments to each reusable workflow explaining inputs/outputs
- [ ] Document tag/branch patterns that trigger each workflow
- [ ] Create troubleshooting guide for new workflow structure
- [ ] Update CONTRIBUTING.md with any new trigger conditions

### Transition Phase
- [ ] Disable old workflows by adding `if: false` (don't delete, just disable)
- [ ] Monitor for 1 week to ensure no edge cases fail
- [ ] Review logs for any missed conditions or edge cases
- [ ] Get final team approval
- [ ] Delete disabled workflows

---

## Target Metrics

### Before Consolidation
- 23 workflow files
- 7136 lines of YAML
- 8 independent Rust setup blocks
- ~25 cargo cache strategy variations
- ~40 minutes CI runtime

### After Consolidation (Target)
- 6-7 primary workflows
- ~3000 lines of YAML (60% reduction)
- 1 centralized Rust setup (reusable)
- 3 consistent cache strategy patterns
- ~35 minutes CI runtime (maintain or improve)

---

## Questions for Review

1. **Is dependabot.yml truly dead?** - Check if GitHub's native Dependabot auto-merge is configured
2. **Which pypi-publish.yml is canonical?** - Choose between `pypi-publish.yml` vs `python-wheels.yml`
3. **Should ffi-tests merge into e2e-tests?** - Or keep separate due to complexity?
4. **Tag naming convention for publishing?** - Confirm v*, npm-v*, and pre-release patterns
5. **Scheduled job frequency acceptable?** - Are daily/weekly security checks the right cadence?

---

## References

- [GitHub Reusable Workflows Documentation](https://docs.github.com/en/actions/using-workflows/reusing-workflows)
- [Workflow Composition Best Practices](https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions)
- [Jobs Dependencies & Parallelization](https://docs.github.com/en/actions/using-jobs/using-jobs-in-a-workflow)

---

**Document Author**: Deployment Engineer  
**Last Updated**: 2026-04-21  
**Status**: Research Phase - Ready for Review
