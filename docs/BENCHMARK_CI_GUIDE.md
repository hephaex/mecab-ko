# Benchmark CI/CD Guide

## Overview

The benchmark CI/CD workflow (`benchmark.yml`) automatically runs performance tests on pull requests and commits, comparing results against the main branch and generating detailed reports.

## Features

### 1. Automated Compilation Check
- Validates benchmark compilation before running
- Uses caching for faster builds
- Runs on all bench-related changes

### 2. Benchmark Execution
- Runs on all commits to main/develop branches
- Runs on all pull requests
- Generates JSON results for historical tracking
- Archives results for 90 days

### 3. Performance Comparison (PRs only)
- Compares PR branch benchmarks against base branch (main)
- Posts detailed comparison table to PR comments
- Automatically detects performance regressions

### 4. Regression Detection

#### Status Levels

| Change | Status | Action | Threshold |
|--------|--------|--------|-----------|
| Improvement | ✅ 🟢 | Approved | < 0% |
| Stable | ✅ | Approved | 0-5% worse |
| Warning | ⚠️ | Review | 5-10% worse |
| Error | ❌ | Prevent Merge | > 10% worse |

#### Example Output

```markdown
## 🚀 Performance Comparison

| Benchmark | main | PR | Change | Status |
|-----------|------|----|---------|---------|
| tokenize_short | 5.2µs | 5.1µs | -1.9% | 🟢 |
| tokenize_medium | 42.0µs | 45.0µs | +7.1% | ⚠️ |
| cold_start | 130ms | 128ms | -1.5% | ✅ |

## Summary

### ⚠️ Performance Regression Warning

The following benchmarks show 5-10% regression:
- tokenize_medium: +7.1%

**Please verify this is acceptable before merging.**
```

### 5. Dashboard Updates
- Updates benchmark dashboard on main branch pushes
- Stores JSON results in `docs/book/src/benchmarks/latest.json`
- Enables historical tracking and visualization

### 6. Conditional Skip
- Skip benchmarks with `[skip bench]` in commit message
- Example: `git commit -m "fix: minor update [skip bench]"`

## Workflow Triggers

### On Push (main/develop)
1. Compile check
2. Run benchmarks
3. Convert to JSON
4. Archive results
5. Update dashboard (main only)

### On Pull Request
1. Compile check
2. Benchmark comparison job (parallel with run-benchmarks)
3. Generate comparison report with regression detection
4. Post PR comment with results

### Manual Trigger (Workflow Dispatch)
- `full_bench: true` - Run extended benchmarks with verbose output

## JSON Result Format

### Benchmark Results (`benchmark-results.json`)

```json
{
  "version": "main",
  "commit": "a4b3a6f",
  "timestamp": "2026-03-03T15:30:45.123456Z",
  "platform": "ubuntu-latest",
  "rustc": "stable",
  "results": {
    "tokenize::tokenize_short": {
      "time_ns": 5200,
      "time_us": 5.2,
      "time_ms": 0.0052
    },
    "tokenize::tokenize_medium": {
      "time_ns": 42000,
      "time_us": 42.0,
      "time_ms": 0.042
    }
  }
}
```

### Comparison Results (`benchmark-comparison.json`)

```json
{
  "pr_number": 123,
  "base_branch": "main",
  "pr_branch": "feature/optimization",
  "base_commit": "a4b3a6f",
  "pr_commit": "b5c4b7g",
  "timestamp": "2026-03-03T15:30:45.123456Z",
  "base_results": { /* ... */ },
  "pr_results": { /* ... */ },
  "comparison": {
    "tokenize::tokenize_short": {
      "base_ns": 5200,
      "pr_ns": 5100,
      "diff_ns": -100,
      "diff_pct": -1.9,
      "status": "pass"
    },
    "tokenize::tokenize_medium": {
      "base_ns": 42000,
      "pr_ns": 45000,
      "diff_ns": 3000,
      "diff_pct": 7.1,
      "status": "warning"
    }
  }
}
```

## Usage Examples

### Run Benchmarks Manually

```bash
# Manual trigger with extended benchmarks
gh workflow run benchmark.yml -f full_bench=true
```

### Skip Benchmarks in Commit

```bash
# Skip benchmark CI for a commit
git commit -m "docs: update README [skip bench]"
```

### Download Benchmark Artifacts

```bash
# View PR benchmark results
gh run download <run_id> -n benchmark-comparison-<run_id>

# Compare JSON results
cat /tmp/benchmark-comparison.json | jq '.comparison'
```

### Analyze Historical Results

```bash
# View latest benchmark data
cat docs/book/src/benchmarks/latest.json | jq '.results'

# Check specific benchmark
cat docs/book/src/benchmarks/latest.json | jq '.results."tokenize::tokenize_short"'
```

## Troubleshooting

### Benchmark Fails to Compile
- Check `benchmark-check` job logs
- Verify no recent breaking changes in dependencies
- Try `cargo clean` and rebuild locally

### PR Comparison Missing
- Ensure base branch has benchmark results
- Check if benchmark-compare job ran (needs PR event)
- Verify `/tmp/base-benchmark.txt` exists in logs

### Regression Detected in PR
- Compare against main branch locally: `cargo bench`
- Investigate code changes affecting performance
- Add optimization or revert changes
- Rerun benchmarks by pushing new commit

### JSON Conversion Fails
- Check Python availability (installed by default in Ubuntu)
- Verify benchmark output format (uses bencher format)
- Check `/tmp/current-benchmark.txt` exists

## Performance Improvement Workflow

1. Create feature branch
2. Make optimization changes
3. Push to PR
4. Review benchmark comparison in PR comments
5. If regression detected (⚠️/❌):
   - Investigate root cause
   - Add profiling or flamegraph
   - Optimize further
   - Commit and push again
6. After merge:
   - Check main branch dashboard update
   - Verify improvement in historical data

## CI/CD Integration

### Status Checks
- `benchmark-check` - Compilation validation (required)
- `run-benchmarks` - Main benchmark execution (informational)
- `benchmark-compare` - PR performance comparison (informational)
- `update-dashboard` - Dashboard data (main only)

### Artifact Retention
- **Default**: 90 days
- **Format**: tar.gz
- **Download**: GitHub Actions tab → Artifacts

### Caching Strategy
- Separate cache keys for check/run/compare jobs
- Fallback to previous build if no exact match
- Invalidates on Cargo.lock changes

## Best Practices

1. **Keep Benchmarks Updated**
   - Add new benchmarks when adding features
   - Update when algorithms change significantly
   - Remove obsolete benchmarks

2. **Investigate Regressions**
   - Don't ignore ⚠️ warnings (5-10%)
   - Always investigate ❌ errors (>10%)
   - Provide explanation in PR comments

3. **Use Skip Sparingly**
   - Only for documentation/config changes
   - Not for code changes
   - Mention reason in commit message

4. **Monitor Dashboard**
   - Check historical trends
   - Identify performance regressions over time
   - Plan optimization work based on trends

5. **Review Comparison Details**
   - Check exact percentage changes
   - Understand variance in measurements
   - Consider outliers or system variations

## Related Documentation

- [Performance Benchmarks](./benchmarks/) - Benchmark results and history
- [CHANGELOG.md](./CHANGELOG.md) - Version history with performance notes
- Project Benchmark Suite: `rust/benches/`

## Future Enhancements

- [ ] Historical trend analysis
- [ ] Benchmark regression alerts
- [ ] Multi-platform benchmarking (macOS, Windows)
- [ ] Comparative benchmarks with other libraries
- [ ] Detailed memory profiling
- [ ] Automated optimization suggestions
- [ ] PR merge blocking on critical regressions
