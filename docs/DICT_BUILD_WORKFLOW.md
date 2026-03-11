# Dictionary Build Workflow Documentation

## Overview

The **Dictionary Build & Validation** GitHub Actions workflow (`dict-build.yml`) automates the compilation of Korean language dictionary files from CSV source data into optimized binary format.

## Workflow Triggers

The workflow is triggered automatically or manually:

### Automatic Triggers (Push)
- Changes to `data/user-dict/**` - Custom user dictionaries
- Changes to `rust/crates/mecab-ko-dict-builder/**` - Dictionary builder code
- Changes to `rust/crates/mecab-ko-dict/**` - Dictionary data structures
- Changes to `rust/crates/mecab-ko-hangul/**` - Korean text processing
- Changes to `rust/Cargo.lock` - Dependency updates
- Changes to `.github/workflows/dict-build.yml` - Workflow itself

### Manual Triggers (workflow_dispatch)
- **Compression**: Integer 0-22 (default: 3)
  - 0 = No compression
  - 3 = Balanced (default)
  - 22 = Maximum (slower, smaller)
- **Encoding**: utf8, euc-kr, or auto (default: auto)

## Workflow Jobs

### 1. build-dictionary (Primary Job)
**Purpose**: Compiles CSV dictionary data into binary format

**Steps**:
1. **Checkout** - Fetch repository with full history
2. **Install Rust** - Set up Rust toolchain (stable)
3. **Cache** - Restore/save cargo build cache
4. **Generate Metadata** - Extract version, build date, git hash
5. **Build CLI** - Compile mecab-ko-dict-builder binary
6. **Prepare Output** - Create dict-output directory
7. **Build Dictionary** - Run builder with compression/encoding options
8. **Calculate Stats** - Measure file sizes and estimate entry count
9. **List Files** - Display generated binary files
10. **Validate Structure** - Verify all required files exist and are non-empty
11. **Upload Artifacts** - Store dictionary files for 30 days
12. **Step Summary** - Generate markdown report

**Outputs**:
```yaml
dict_version: Project version from Cargo.toml
dict_size: Size of sys.dic in bytes
entry_count: Estimated morpheme entries
```

### 2. tokenize-test (Validation Job)
**Purpose**: Validates the built dictionary with tokenization tests

**Depends On**: build-dictionary

**Steps**:
1. **Checkout** - Fetch repository
2. **Install Rust** - Set up toolchain
3. **Download Artifacts** - Retrieve dictionary from previous job
4. **Build Tokenizer** - Compile mecab CLI binary
5. **Run Tests** - Test dictionary with sample Korean sentences:
   - `한국어 형태소 분석기입니다.`
   - `안녕하세요. 반갑습니다.`
   - `형태소 분석은 자연어 처리의 기초입니다.`
   - `MeCab-Ko는 빠르고 정확한 한국어 형태소 분석기입니다.`
6. **Verify Integrity** - Check file sizes and validity

**Status**: Continues on error to not block subsequent jobs

### 3. generate-report (Reporting Job)
**Purpose**: Creates comprehensive build documentation

**Depends On**: build-dictionary, tokenize-test

**Steps**:
1. **Checkout** - Fetch repository
2. **Download Artifacts** - Get dictionary files
3. **Create Report** - Generate markdown with:
   - Build number, trigger, branch, commit
   - Dictionary file summary table
   - Build configuration details
   - File statistics and sizes
4. **Upload Report** - Store for 30 days
5. **Archive Logs** - Save complete logs

### 4. cleanup-old-artifacts (Maintenance Job)
**Purpose**: Removes artifacts older than 30 days

**Triggers**: Only on push events

**Cleans**: dict-binary-*, dict-build-report-*, dict-build-logs-*

### 5. notify-status (Final Status Job)
**Purpose**: Summarizes overall workflow status

**Depends On**: All previous jobs

**Actions**:
- Checks all job results
- Fails if any job failed
- Creates final GitHub Step Summary

## Dictionary Output Files

### Required Files
| File | Format | Purpose |
|------|--------|---------|
| **sys.dic** | Double-Array Trie (binary) | Morpheme lookup structure |
| **matrix.bin** | Little-endian i16 array | Morpheme transition costs |

### Optional Files
| File | Format | Purpose |
|------|--------|---------|
| **char.bin** | Binary | Character type definitions |
| **unk.bin** | Binary | Unknown word handling rules |

## File Specifications

### sys.dic
- **Format**: Double-Array Trie binary (yada library)
- **Content**: All morpheme forms from CSV
- **Size**: Varies by dictionary size (typically 10-50MB uncompressed)
- **Compression**: zstd (configurable 0-22, default 3)
- **Reduction**: ~30-40% at level 3

### matrix.bin
- **Format**: Binary matrix
- **Header**: lsize (u16) + rsize (u16)
- **Data**: lsize × rsize i16 values
- **Endianness**: Little-endian
- **Purpose**: Morpheme transition cost lookup

### char.bin
- **Content**: Character type definitions
- **Format**: Binary encoding of char.def
- **Categories**: DEFAULT, SPACE, HANGUL, etc.
- **Optional**: Not required if no char.def provided

### unk.bin
- **Content**: Unknown morpheme handling rules
- **Format**: Binary encoding of unk.def
- **Structure**: Character type, costs, POS features
- **Optional**: Not required if no unk.def provided

## Build Configuration

### Compression Levels
```
0   = No compression        (fastest, largest file)
3   = Default balanced      (recommended)
10  = Medium compression    (balance)
22  = Maximum compression   (slowest, smallest file)
```

### Encoding Detection
- **utf8**: UTF-8 encoded CSV files
- **euc-kr**: EUC-KR (CP949) encoded CSV files
- **auto**: Auto-detect encoding (recommended)

## GitHub Step Summary Output

Each build generates a detailed markdown report:

```markdown
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
| Artifact | dict-binary-1234 |

### Generated Files
- sys.dic - Double-Array Trie binary
- matrix.bin - Morpheme transition costs
- char.bin - Character type definitions (if available)
- unk.bin - Unknown word handling rules (if available)

### Build Status
- ✓ Dictionary built successfully
- ✓ All required files validated
- ✓ Artifacts uploaded
```

## Artifacts

### Artifact Naming
- **Dictionary**: `dict-binary-{run_number}`
- **Report**: `dict-build-report-{run_number}`
- **Logs**: `dict-build-logs-{run_number}`

### Retention Policy
- **Default**: 30 days
- **Auto-cleanup**: Artifacts older than 30 days automatically deleted
- **Download**: Available via GitHub Actions interface or API

### Artifact Contents
```
dict-binary-{run_number}/
├── sys.dic              # Main dictionary (Trie structure)
├── matrix.bin           # Cost matrix
├── char.bin             # Character types (if available)
└── unk.bin              # Unknown word rules (if available)
```

## Accessing Artifacts

### Via GitHub Web UI
1. Go to Actions → Dictionary Build & Validation
2. Select latest run
3. Scroll to "Artifacts" section
4. Download desired artifact

### Via GitHub API
```bash
# List all artifacts for a workflow run
gh api repos/OWNER/REPO/actions/runs/RUN_ID/artifacts

# Download specific artifact
gh run download RUN_ID -n dict-binary-1234
```

### Via GitHub CLI
```bash
# Get latest workflow run
gh run list --workflow dict-build.yml --limit 1

# Download latest artifacts
gh run download --pattern "dict-*"
```

## Integration with CI/CD

### Using Built Dictionary
```bash
# Download artifacts from workflow
gh run download RUN_ID -n dict-binary-1234

# Use in tokenization
./mecab --dicdir ./dict-binary-1234 < input.txt
```

### Docker Integration
```dockerfile
# Download from artifact
COPY dict-binary-1234/ /opt/mecab-ko/dict/

# Use in container
CMD ["mecab", "--dicdir", "/opt/mecab-ko/dict/"]
```

### Release Integration
- Dictionary artifacts can be attached to GitHub releases
- Enable automated dictionary distribution with releases
- Version-locked dictionary files for reproducibility

## Troubleshooting

### Build Failures

**Problem**: "Dictionary build failed"
```
Solution:
1. Check input CSV format (12 columns required)
2. Verify CSV encoding (UTF-8 or EUC-KR)
3. Review matrix.def format (lsize rsize required)
4. Check disk space in runner
```

**Problem**: "Artifacts upload failed"
```
Solution:
1. Check dict-output directory exists
2. Verify at least sys.dic and matrix.bin generated
3. Check file permissions (read access)
4. Review artifact size (GitHub has 400MB limit per artifact)
```

### Validation Failures

**Problem**: "Missing required files"
```
Solution:
1. Re-run dict-builder with verbose flag
2. Check input CSV for format errors
3. Verify matrix.def exists in input directory
4. Review builder logs for parsing errors
```

**Problem**: "Tokenization test failed"
```
Solution:
1. Verify dictionary files are valid (file > 0 bytes)
2. Check char.bin and unk.bin if present
3. Test with simpler input first
4. Review builder error logs
```

## Environment Variables

```yaml
CARGO_TERM_COLOR: always      # Colored cargo output
RUST_BACKTRACE: 1             # Stack traces on panic
```

## Cache Strategy

**Cache Keys**:
- `{os}-cargo-dict-{Cargo.lock_hash}`
- `{os}-cargo-test-{Cargo.lock_hash}`

**Cached Paths**:
- `~/.cargo/registry` - Downloaded dependencies
- `~/.cargo/git` - Git dependencies
- `rust/target` - Compiled objects

**Hit Conditions**:
- Exact key match (Cargo.lock unchanged)
- Partial match (OS and stage match, older Cargo.lock)

## Performance Characteristics

### Build Time
- **Cold cache**: 3-5 minutes
- **Warm cache**: 1-2 minutes
- **Compression level impact**: +30s per level (e.g., level 22 adds 10 minutes)

### File Sizes (approx)
| File | Size | Notes |
|------|------|-------|
| sys.dic (level 0) | 45-60 MB | Uncompressed |
| sys.dic (level 3) | 25-35 MB | Default |
| sys.dic (level 22) | 10-15 MB | Maximum compression |
| matrix.bin | 2-4 MB | Typically small |
| char.bin | 1-2 KB | Usually minimal |
| unk.bin | 10-20 KB | Usually small |

## Security Considerations

### Artifact Access
- Public repositories: Artifacts accessible to all users
- Private repositories: Artifacts restricted to repository members
- Workflow triggered by PR: Artifacts available to PR author

### Secret Management
- No secrets used in workflow (none required)
- Dictionary data is public source material
- Build is reproducible and verifiable

## Advanced Usage

### Custom Input Dictionary
```bash
# Via workflow_dispatch (UI)
1. Go to Actions → Dictionary Build & Validation
2. Click "Run workflow"
3. Set compression and encoding
4. Click "Run workflow"

# Via GitHub CLI
gh workflow run dict-build.yml \
  -f compression=5 \
  -f encoding=utf8
```

### Scheduled Builds
To add scheduled builds (e.g., weekly), add to workflow:
```yaml
on:
  schedule:
    - cron: '0 2 * * 0'  # Sunday 2:00 UTC
```

### Matrix Builds (Optional)
For multi-platform dictionary optimization:
```yaml
strategy:
  matrix:
    compression: [0, 3, 10, 22]
    encoding: [utf8, euc-kr]
```

## Monitoring and Alerts

### Success Indicators
- All jobs complete with status = success
- Step Summary shows ✓ for all checks
- Artifacts available for download
- No build warnings in logs

### Failure Alerts
- GitHub creates workflow failure notification
- Check "Actions" tab for error details
- Review job logs for specific failure reason
- Common causes listed in Troubleshooting section

## Related Documentation

- **Dictionary Builder**: `rust/crates/mecab-ko-dict-builder/README.md`
- **Dictionary Format**: `rust/crates/mecab-ko-dict/README.md`
- **Dictionary Data**: `data/mecab-ko-dic-2.1.1-20180720/`
- **User Dictionary**: `data/user-dict/README.md`

## Version History

### Version 1.0 (Current)
- Automated dictionary building from CSV
- Configurable compression and encoding
- Tokenization validation tests
- Comprehensive reporting and artifacts
- 30-day artifact retention
- Automatic cleanup of old artifacts

## Support and Contribution

For issues or enhancements:
1. Check existing GitHub Issues
2. Create new issue with:
   - Workflow run number
   - Error messages
   - Steps to reproduce
3. Include relevant logs from Actions tab
