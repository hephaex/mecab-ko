# Performance Baselines & Thresholds

## Overview

This document defines the performance baselines and regression thresholds used by the automated benchmark CI/CD system.

## Baseline Performance Metrics (v0.3.0)

### Tokenization Performance

| Benchmark | Metric | Baseline | Target | Threshold |
|-----------|--------|----------|--------|-----------|
| tokenize_short (15 chars) | µs/op | 1.84 | < 2.5 | 5% regression |
| tokenize_medium (75 chars) | µs/op | 11.49 | < 15 | 5% regression |
| tokenize_long (200 chars) | µs/op | 41.14 | < 50 | 5% regression |
| tokenize_xlarge (500 chars) | µs/op | 512 | < 600 | 5% regression |
| tokenize_batch (10 items) | µs/op | 58.33 | < 70 | 5% regression |

### Initialization & Memory

| Benchmark | Metric | Baseline | Target | Status |
|-----------|--------|----------|--------|--------|
| Cold start | ms | < 1 | < 200 | ✅ PASS |
| Dictionary load | ms | N/A | < 500 | TBD |
| Memory usage (mini-dict) | MB | ~20 | < 50 | ✅ PASS |
| Memory usage (full-dict) | MB | 215 | < 150 | TBD |
| Streaming throughput | MiB/s | 3.77 | > 3 | ✅ PASS |

### Dictionary Operations

| Operation | Metric | Baseline | Target |
|-----------|--------|----------|--------|
| Lookup (single) | µs/op | ~1.2 | < 2 |
| Lookup (multi-entry) | µs/op | ~2.5 | < 5 |
| Feature parsing | µs/op | ~0.8 | < 1.5 |

## Regression Thresholds

### Automated Detection Levels

```
Performance Change Range    │ Symbol │ CI Action              │ PR Mergeable
─────────────────────────────────────────────────────────────────────────────
Improvement (< 0%)          │ 🟢 ✅  │ Approved               │ Yes
Stable (0% - 5% worse)      │ ✅    │ Approved               │ Yes
Warning (5% - 10% worse)    │ ⚠️    │ Review required        │ Depends
Critical (> 10% worse)      │ ❌    │ Prevent merge           │ No
─────────────────────────────────────────────────────────────────────────────
```

### Regression Classification

#### Acceptable (0-5%)
- Natural variance in measurements
- May be acceptable for code clarity
- Monitor trend over multiple PRs
- Example: 42.0µs → 42.9µs (+2.1%)

#### Warning (5-10%)
- Needs justification in PR comments
- Review algorithm complexity changes
- May need reoptimization
- Example: 42.0µs → 44.1µs (+5.0%)

#### Critical (>10%)
- Blocks PR merge
- Requires investigation and fix
- May indicate algorithm change
- Example: 42.0µs → 46.2µs (+10.0%)

## Measurement Conditions

### Test Environment
- **Platform**: Ubuntu 22.04 LTS (GitHub Actions)
- **Processor**: 2-core CPU (standard runner)
- **Memory**: 7 GB RAM
- **Rust**: Latest stable toolchain
- **Dictionary**: mini-dict (3.5 MB, ~30K entries)

### Measurement Settings
- **Iterations**: Default (typically 100+)
- **Warmup**: Built-in (first few iterations discarded)
- **Format**: Bencher (ns/iter with ± variance)
- **Variance**: Automatic from Rust benchmark harness

### Variance Handling
- Bencher format includes +/- confidence interval
- Threshold based on point estimate (not upper bound)
- Multiple runs average out transient effects
- Consistent runner configuration for reliability

## Baseline History

### v0.1.0 (Initial Release)
- Cold start: 0.086ms
- Tokenize (50 chars): ~45µs
- Memory (mini-dict): ~20MB

### v0.1.1 (Bug fixes & optimization)
- No significant baseline changes
- Some improvements from compiler updates

### v0.2.0
- Optimized Viterbi decoder
- Improved tokenization performance
- Better cache locality

### v0.3.0 (Current)
- **3x+ performance improvement** over v0.2.0
- Memory optimization: PosTagInterner, FeatureCache
- Streaming API: 5x throughput with chunked processing
- New analysis modes: nouns, pos, morphs
- Tokenize (15 chars): ~1.84µs (was ~6µs)
- Tokenize (75 chars): ~11.49µs (was ~42µs)
- Current baselines as listed above

## Performance Optimization Priorities

### High Priority (Actively Optimizing)
- [ ] Full-dict memory usage (215MB → 150MB target)
- [ ] Dictionary lookup performance
- [ ] Feature parsing overhead

### Medium Priority (Monitor)
- [ ] Batch processing throughput
- [ ] Decomposition algorithm
- [ ] Unknown word handling

### Low Priority (Maintain)
- [ ] Cold start (already < 200ms target)
- [ ] Simple tokenization (fast enough)
- [ ] API overhead

## Guidelines for PR Authors

### When to Investigate Regression

1. **Any ❌ (>10%) regression**
   - MUST investigate and fix
   - Document findings in PR comment
   - Provide optimization strategy

2. **⚠️ (5-10%) regression with:**
   - Algorithmic changes
   - Dependency updates
   - New features affecting hot path
   - Document justification in PR

3. **Accepted without investigation:**
   - ✅ (0-5%) regression with clear cause
   - 🟢 Performance improvements
   - Unrelated documentation changes (use `[skip bench]`)

### Example PR Comments

```markdown
### Performance Analysis

The 7.1% regression in `tokenize_medium` is due to:
- Added validation for malformed input
- Trade-off: Safety improvement > performance cost
- Will optimize in follow-up PR using caching

Justification: Issue #123 requires input validation
```

## Continuous Monitoring

### Weekly Review
- Check benchmark dashboard trends
- Identify slow deterioration patterns
- Plan optimization sprints

### Monthly Review
- Analyze regressions vs improvements
- Update baselines if warranted
- Document performance engineering decisions

### Quarterly Review
- Major baseline updates
- Strategy adjustments
- New benchmark additions

## Tools & Commands

### Local Benchmarking

```bash
# Run full benchmark suite
cargo bench --manifest-path rust/Cargo.toml

# Run specific benchmark
cargo bench --manifest-path rust/Cargo.toml -- tokenize

# Run with detailed output
cargo bench --manifest-path rust/Cargo.toml -- --verbose --output-format bencher

# Compare against baseline (using profiler)
cargo run --manifest-path rust/crates/mecab-ko-profiler/Cargo.toml -- \
  baseline save
cargo run --manifest-path rust/crates/mecab-ko-profiler/Cargo.toml -- \
  baseline compare
```

### Analyzing Results

```bash
# Extract JSON from PR artifact
gh run download <run_id> -n benchmark-comparison-<run_id>
cat benchmark-comparison.json | jq '.comparison | to_entries[] | select(.value.diff_pct > 5)'

# Check latest dashboard
cat docs/book/src/benchmarks/latest.json | jq '.results'

# Find regressions in comparison
jq '.comparison[] | select(.status == "warning" or .status == "error")' benchmark-comparison.json
```

## Related Documentation

- [BENCHMARK_CI_GUIDE.md](./BENCHMARK_CI_GUIDE.md) - CI/CD workflow details
- [.github/workflows/benchmark.yml](../.github/workflows/benchmark.yml) - Workflow configuration
- [rust/benches/](../rust/benches/) - Benchmark source code
- [CHANGELOG.md](./CHANGELOG.md) - Version history with performance notes

## FAQ

**Q: Why is my PR showing a 6% regression?**
A: This is in the warning zone (5-10%). You should investigate if it's acceptable. Document your findings in the PR comment.

**Q: Can I skip benchmarks for documentation changes?**
A: Yes, use `[skip bench]` in your commit message for doc/config-only PRs.

**Q: What if variance is high and I'm at the threshold?**
A: The threshold is based on point estimate. High variance is normal. If multiple runs show consistent regression, investigate.

**Q: How do I compare with my local machine?**
A: Local results may differ due to hardware. Use the CI results as source of truth. Reference `PERFORMANCE_OPTIMIZATION_TIPS.md` for reproducible local testing.

**Q: When should we update baselines?**
A: When major version released, optimization completed, or algorithm significantly changed. Discuss with team first.
