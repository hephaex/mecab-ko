# PyPI Release Guide

This document describes how to release mecab-ko-python to PyPI.

## Prerequisites

1. **PyPI Account Setup**
   - Create accounts on [PyPI](https://pypi.org/) and [TestPyPI](https://test.pypi.org/)
   - Configure API tokens for secure publishing

2. **GitHub Repository Setup**
   - Add PyPI API token to GitHub Secrets as `PYPI_API_TOKEN`
   - Add TestPyPI API token to GitHub Secrets as `TEST_PYPI_API_TOKEN`
   - Enable GitHub Actions in repository settings

3. **Trusted Publishing (Recommended)**
   - Configure PyPI trusted publishing for the repository
   - This eliminates the need for API tokens
   - See: https://docs.pypi.org/trusted-publishers/

## Release Process

### Automated Release (Recommended)

The GitHub Actions workflow handles building and publishing automatically.

#### 1. Prepare the Release

```bash
# Update version numbers
cd rust/crates/mecab-ko-python

# Edit Cargo.toml
version = "0.2.0"  # Update this

# Edit pyproject.toml
version = "0.2.0"  # Update this

# Commit changes
git add Cargo.toml pyproject.toml
git commit -m "chore: bump version to 0.2.0"
```

#### 2. Create Git Tag

```bash
# Create and push tag
git tag v0.2.0
git push origin v0.2.0
```

This will automatically trigger the PyPI publish workflow which will:
- Build wheels for Linux (x86_64, aarch64)
- Build wheels for macOS (x86_64, Apple Silicon)
- Build wheels for Windows (x86_64)
- Build source distribution (sdist)
- Run tests on all platforms
- Publish to PyPI
- Create GitHub Release with artifacts

#### 3. Test PyPI Upload (Optional)

To test the release process without publishing to production PyPI:

```bash
# Manually trigger workflow with test mode
gh workflow run pypi-publish.yml -f test_pypi=true
```

### Manual Release (Fallback)

If you need to release manually:

#### 1. Install Tools

```bash
pip install maturin twine
```

#### 2. Build Wheels

```bash
cd rust/crates/mecab-ko-python

# Build for current platform
maturin build --release --strip

# Or use Docker for Linux builds
docker run --rm -v $(pwd):/io \
  ghcr.io/pyo3/maturin build --release \
  --strip \
  --manylinux 2_28
```

#### 3. Build Source Distribution

```bash
maturin sdist
```

#### 4. Test Upload to TestPyPI

```bash
# Upload to TestPyPI
twine upload --repository testpypi dist/*

# Test installation
pip install --index-url https://test.pypi.org/simple/ \
  --extra-index-url https://pypi.org/simple/ \
  mecab-ko-python
```

#### 5. Upload to PyPI

```bash
# Upload to production PyPI
twine upload dist/*

# Verify installation
pip install mecab-ko-python
```

## Version Numbering

Follow Semantic Versioning (SemVer):

- **Major version** (X.0.0): Breaking changes
- **Minor version** (0.X.0): New features, backward compatible
- **Patch version** (0.0.X): Bug fixes, backward compatible

Examples:
- `0.1.0` - Initial release
- `0.1.1` - Bug fix
- `0.2.0` - New feature
- `1.0.0` - First stable release

## Pre-release Versions

For testing purposes, use pre-release versions:

```
0.2.0a1  # Alpha release
0.2.0b1  # Beta release
0.2.0rc1 # Release candidate
```

## Workflow Configuration

The PyPI publish workflow (`.github/workflows/pypi-publish.yml`) includes:

### Build Matrix

| Platform | Architecture | Runner |
|----------|-------------|--------|
| Linux | x86_64 | ubuntu-latest |
| Linux | aarch64 | ubuntu-latest (QEMU) |
| macOS | x86_64 | macos-13 |
| macOS | Apple Silicon | macos-14 |
| Windows | x86_64 | windows-latest |

### Python Versions

Wheels are tested on Python 3.8, 3.9, 3.10, 3.11, 3.12

### Security

- Uses **Trusted Publishing** (OIDC) - no API tokens needed
- Requires `id-token: write` permission
- Protected environment: `pypi`

## Troubleshooting

### Build Failures

1. **Rust compilation errors**
   ```bash
   # Check Rust version
   rustc --version  # Should be 1.75+

   # Update Rust
   rustup update stable
   ```

2. **maturin errors**
   ```bash
   # Update maturin
   pip install --upgrade maturin

   # Check maturin version
   maturin --version  # Should be 1.0+
   ```

3. **Cross-compilation issues**
   - For aarch64 Linux: Ensure QEMU is installed
   - For macOS: Ensure correct SDK is available

### Upload Failures

1. **Version already exists**
   - PyPI does not allow re-uploading the same version
   - Bump version number and retry

2. **Authentication errors**
   - Check PyPI API token in GitHub Secrets
   - Verify trusted publishing configuration
   - Ensure `id-token: write` permission is set

3. **File size limits**
   - PyPI has a 100MB limit per file
   - Consider stripping debug symbols: `--strip`

### Testing Failures

1. **Import errors**
   ```python
   # Check installed files
   pip show mecab-ko-python

   # Verify import
   python -c "from mecab_ko import Mecab; print(Mecab())"
   ```

2. **Runtime errors**
   - Check Python version compatibility
   - Verify wheel platform compatibility
   - Check for missing dependencies

## Verification Checklist

Before releasing:

- [ ] Update version in `Cargo.toml` and `pyproject.toml`
- [ ] Update `CHANGELOG.md` with release notes
- [ ] Run tests locally: `cargo test`
- [ ] Run clippy: `cargo clippy`
- [ ] Format code: `cargo fmt`
- [ ] Build locally: `maturin build --release`
- [ ] Test local wheel: `pip install target/wheels/*.whl`
- [ ] Verify basic functionality
- [ ] Commit and push changes
- [ ] Create and push git tag

After releasing:

- [ ] Verify PyPI page: https://pypi.org/project/mecab-ko-python/
- [ ] Test installation: `pip install mecab-ko-python`
- [ ] Check GitHub Release: https://github.com/hephaex/mecab-ko/releases
- [ ] Update documentation if needed

## Resources

- [maturin documentation](https://www.maturin.rs/)
- [PyPA packaging guide](https://packaging.python.org/)
- [PyPI trusted publishing](https://docs.pypi.org/trusted-publishers/)
- [Semantic Versioning](https://semver.org/)
- [GitHub Actions documentation](https://docs.github.com/en/actions)

## Support

For issues or questions:
- GitHub Issues: https://github.com/hephaex/mecab-ko/issues
- Discussions: https://github.com/hephaex/mecab-ko/discussions
