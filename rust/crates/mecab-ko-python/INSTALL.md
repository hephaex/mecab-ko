# Installation Guide

## Prerequisites

### System Requirements

- Python 3.8 or later
- Rust toolchain (1.75 or later)
- Maturin (Python package builder for Rust)

### Install Maturin

```bash
pip install maturin
```

Or with pipx for isolated installation:

```bash
pipx install maturin
```

## Installation Methods

### Method 1: Development Installation (Recommended for Development)

This method installs the package in development mode, allowing you to make changes and test immediately.

```bash
# Navigate to the Python bindings directory
cd /home/mare/mecab-ko/rust/crates/mecab-ko-python

# Install in development mode
maturin develop

# Or for release mode (faster but slower to build)
maturin develop --release
```

### Method 2: Build Wheel and Install

This method builds a wheel file that can be distributed and installed.

```bash
# Navigate to the Python bindings directory
cd /home/mare/mecab-ko/rust/crates/mecab-ko-python

# Build the wheel
maturin build --release

# Install the wheel
pip install target/wheels/mecab_ko-*.whl
```

### Method 3: Build from Workspace Root

You can also build from the workspace root:

```bash
# From the workspace root
cd /home/mare/mecab-ko/rust

# Build the Python bindings
cargo build -p mecab-ko-python --release

# The shared library will be at:
# target/release/libmecab_ko.so (Linux)
# target/release/libmecab_ko.dylib (macOS)
# target/release/mecab_ko.dll (Windows)
```

## Verification

After installation, verify the package works:

```python
# Start Python interpreter
python3

# Try importing and using the module
>>> from mecab_ko import Mecab
>>> mecab = Mecab()
>>> mecab.morphs("안녕하세요")
['안녕', '하', '세요']
```

## Running Tests

```bash
# Install pytest if not already installed
pip install pytest

# Run the Python tests
cd /home/mare/mecab-ko/rust/crates/mecab-ko-python
pytest tests/test_mecab.py -v
```

## Running Examples

```bash
# Run the example script
cd /home/mare/mecab-ko/rust/crates/mecab-ko-python
python3 examples/example.py
```

## Troubleshooting

### Import Error: cannot import name 'Mecab'

**Solution:** Make sure you've run `maturin develop` or installed the wheel package.

### Build Errors

**Solution:** Ensure you have:
- Latest Rust toolchain: `rustup update`
- Python development headers: `sudo apt-get install python3-dev` (Ubuntu/Debian)
- Maturin installed: `pip install maturin`

### Missing Dictionary Errors

**Solution:** The current implementation uses a stub tokenizer. Full dictionary support will be added in future releases.

### Platform-Specific Issues

#### Linux
- Ensure Python development packages are installed
- For Ubuntu/Debian: `sudo apt-get install python3-dev`
- For Fedora/RHEL: `sudo dnf install python3-devel`

#### macOS
- Install Python via Homebrew: `brew install python`
- Ensure Xcode command line tools: `xcode-select --install`

#### Windows
- Install Microsoft Visual C++ Build Tools
- Use Python from python.org (not Microsoft Store version)

## Uninstallation

```bash
pip uninstall mecab-ko
```

## Development Workflow

For active development:

```bash
# 1. Make changes to src/lib.rs or other Rust files

# 2. Rebuild and reinstall
maturin develop --release

# 3. Test your changes
python3 examples/example.py
pytest tests/test_mecab.py

# 4. Repeat as needed
```

## Building for Distribution

To create a wheel for distribution:

```bash
# Build release wheel
maturin build --release

# The wheel will be in target/wheels/
# You can upload this to PyPI or distribute it manually
```

## Performance Tips

- Use `--release` flag for production builds (much faster at runtime)
- Development builds are faster to compile but slower to run
- The first build will take longer due to dependency compilation

## Additional Resources

- [Maturin Documentation](https://www.maturin.rs/)
- [PyO3 Documentation](https://pyo3.rs/)
- [MeCab-Ko Project](https://github.com/hephaex/mecab-ko)
