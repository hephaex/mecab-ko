# Dictionary Build Workflow - Quick Reference

## What Does This Workflow Do?

Automatically compiles Korean dictionary CSV files into optimized binary format whenever:
- Dictionary builder code changes
- Dictionary data changes
- User custom dictionaries change
- Or manually triggered

## Key Features

✓ **Automatic Dictionary Compilation**
- Converts CSV to Double-Array Trie binary
- Generates morpheme cost matrix
- Compresses output with configurable zstd levels

✓ **Validation Testing**
- Tests built dictionary with sample Korean text
- Verifies all required binary files generated
- Checks file integrity and size

✓ **Artifact Management**
- Stores compiled dictionaries for 30 days
- Generates comprehensive build reports
- Auto-cleanup of old artifacts

✓ **Build Reporting**
- Step Summary with file sizes and statistics
- Markdown report with build metadata
- Download links for all generated files

## File Organization

```
.github/workflows/
├── dict-build.yml              ← Main workflow definition
└── DICT_BUILD_README.md        ← This file

Related files:
├── DICT_BUILD_WORKFLOW.md      ← Complete documentation
├── rust/crates/mecab-ko-dict-builder/   ← Dictionary compiler
└── data/
    ├── mecab-ko-dic-2.1.1-20180720/    ← System dictionary
    └── user-dict/                       ← Custom entries
```

## Quick Start

### Manually Trigger Build
1. Go to **GitHub** → **Actions** → **Dictionary Build & Validation**
2. Click **Run workflow** button
3. (Optional) Set compression level (0-22) and encoding
4. Click **Run workflow**

### Using GitHub CLI
```bash
# Run with defaults (compression=3, encoding=auto)
gh workflow run dict-build.yml

# Run with custom settings
gh workflow run dict-build.yml \
  -f compression=5 \
  -f encoding=utf8
```

### Download Built Dictionary
```bash
# Get latest workflow run
gh run list --workflow dict-build.yml --limit 1

# Extract run ID and download
gh run download <RUN_ID> -n dict-binary-<RUN_NUMBER>

# Or download all dictionary artifacts
gh run download <RUN_ID> --pattern "dict-*"
```

## What Gets Generated

### Binary Files (in dict-binary-{N} artifact)
- **sys.dic** - Main morpheme dictionary (Trie structure)
- **matrix.bin** - Morpheme transition costs
- **char.bin** - Character type definitions
- **unk.bin** - Unknown word handling rules

### Reports & Logs
- **dict-build-report-{N}** - Build summary markdown
- **dict-build-logs-{N}** - Complete dictionary files and logs
- **Step Summary** - In-line workflow summary on Actions page

## Configuration

### Compression Level
```
0   = No compression        (fastest build)
3   = Balanced (DEFAULT)    (recommended)
10  = Better compression    (slower)
22  = Maximum compression   (very slow)
```

Typical sizes at compression level 3: 25-35 MB (sys.dic)

### Encoding
```
utf8    = UTF-8 encoded CSV
euc-kr  = EUC-KR encoded CSV
auto    = Auto-detect (DEFAULT)
```

## Automatic Triggers

Workflow runs automatically when:
- `/data/user-dict/` changes
- `/rust/crates/mecab-ko-dict-builder/` changes
- `/rust/crates/mecab-ko-dict/` changes
- `/rust/crates/mecab-ko-hangul/` changes
- `/rust/Cargo.lock` changes (dependency updates)
- This workflow file changes

## Build Status

### Check Latest Build
```bash
gh run list --workflow dict-build.yml
```

### View Detailed Logs
1. Go to **Actions** → **Dictionary Build & Validation**
2. Click latest run
3. Expand job sections to view detailed logs

### Common Status Values
- ✓ **Success** - All jobs passed, artifacts available
- ✗ **Failure** - One or more jobs failed, check logs
- ⏭️ **Skipped** - Workflow conditions not met

## Artifact Retention

- **Default retention**: 30 days
- **Auto-cleanup**: Runs weekly to remove old artifacts
- **Download**: Available via GitHub Web UI or API

## Typical Workflow Duration

| Condition | Duration |
|-----------|----------|
| Warm cache (Cargo.lock unchanged) | 1-2 minutes |
| Cold cache (first run or deps changed) | 3-5 minutes |
| Compression level 22 | +10 minutes |
| Full workflow (build + tests + report) | 5-15 minutes |

## Troubleshooting

### Dictionary Build Failed
```bash
# Check build logs
1. Actions → Dictionary Build & Validation → Latest run
2. Expand "Build dictionary" job
3. Review error messages
4. Check input CSV format (must be 12 columns)
```

### Artifacts Not Available
```
Check:
- Build job completed successfully
- At least sys.dic and matrix.bin exist
- Artifact retention period (max 30 days)
- GitHub storage limits (rarely an issue)
```

### Tokenization Tests Failed
```
Likely not critical - indicates dictionary works but
sample tests may have minor issues. Check:
- Dictionary files are non-empty
- char.bin and unk.bin if present
- Builder logs for warnings
```

## Integration Examples

### Use in CI/CD Pipeline
```yaml
# Download dictionary in another workflow
- name: Download Dictionary
  run: |
    gh run download <RUN_ID> -n dict-binary-<RUN_NUMBER>
    ls -la dict-binary-<RUN_NUMBER>/
```

### Docker Build Integration
```dockerfile
# In Dockerfile
ARG DICT_ARTIFACT=dict-binary-1234
COPY ${DICT_ARTIFACT}/ /opt/mecab-ko/dict/
```

### Local Development
```bash
# Download latest dictionary
gh run download $(gh run list --workflow dict-build.yml -L1 -q) \
  -n dict-binary-$(gh run list --workflow dict-build.yml -L1 -q)

# Test locally
./mecab --dicdir ./dict-binary-1234 < test.txt
```

## Step Summary Output Example

```
## Dictionary Build Report

| Field | Value |
|-------|-------|
| Version | 0.2.0 |
| Build Date | 2026-03-02T14:30:00Z |
| Git Commit | abc1234 |
| Compression | 3 |
| Encoding | auto |
| Dictionary Size | 15234567 bytes |
| Estimated Entries | 152345 |

### Build Status
✓ Dictionary built successfully
✓ All required files validated
✓ Artifacts uploaded
```

## Key Metrics

### File Sizes (Typical)
| File | Size |
|------|------|
| sys.dic (compressed, L3) | 25-35 MB |
| matrix.bin | 2-4 MB |
| char.bin | <1 KB |
| unk.bin | 10-20 KB |

### Build Statistics
| Metric | Value |
|--------|-------|
| Morpheme entries | ~150,000-200,000 |
| Build time (warm cache) | 1-2 minutes |
| Build time (cold cache) | 3-5 minutes |
| Artifact retention | 30 days |

## Job Dependency Chain

```
build-dictionary ─┐
                  ├─→ tokenize-test ─┐
cleanup ─────────┐                    ├─→ notify-status
                  │                  │
generate-report ──┘────────────────┘
```

- **build-dictionary**: Main compilation job
- **tokenize-test**: Validates with sample Korean text
- **generate-report**: Creates documentation
- **cleanup-old-artifacts**: Maintenance
- **notify-status**: Final summary

All jobs complete before final status reported.

## More Information

- Full documentation: `DICT_BUILD_WORKFLOW.md`
- Dictionary builder: `rust/crates/mecab-ko-dict-builder/README.md`
- Dictionary format: `rust/crates/mecab-ko-dict/README.md`

## Support

For issues:
1. Check workflow logs: Actions → Dictionary Build & Validation
2. Review error messages in failed jobs
3. Check Troubleshooting section above
4. Open GitHub issue if needed
