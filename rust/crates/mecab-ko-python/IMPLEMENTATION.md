# Python Bindings Implementation Summary

## Overview

This document provides a technical overview of the Python bindings implementation for MeCab-Ko.

## Architecture

### Components

```
mecab-ko-python/
├── src/
│   └── lib.rs              # Main PyO3 bindings implementation
├── tests/
│   └── test_mecab.py       # Python unit tests
├── examples/
│   └── example.py          # Usage examples
├── Cargo.toml              # Rust package configuration
├── pyproject.toml          # Python package configuration
└── README.md               # User documentation
```

### Technology Stack

- **PyO3**: Rust bindings for Python (v0.20)
- **Maturin**: Build system for Rust-Python projects
- **mecab-ko-core**: Underlying Rust tokenizer implementation

## API Design

The Python API is designed to be compatible with KoNLPy's Mecab interface:

### Class: `Mecab`

```python
class Mecab:
    def __init__(self, dicpath: Optional[str] = None) -> None: ...
    def morphs(self, text: str) -> List[str]: ...
    def nouns(self, text: str) -> List[str]: ...
    def pos(self, text: str) -> List[Tuple[str, str]]: ...
    def parse(self, text: str) -> str: ...
    def wakati(self, text: str) -> List[str]: ...
```

### Method Signatures

#### `__init__(dicpath=None)`
- Creates a new tokenizer instance
- Optional `dicpath` parameter for custom dictionary
- Raises `RuntimeError` if initialization fails

#### `morphs(text: str) -> List[str]`
- Extracts morphemes from text
- Returns list of surface forms
- Equivalent to `wakati()`

#### `nouns(text: str) -> List[str]`
- Extracts only nouns from text
- Filters tokens with POS tags starting with "NN"
- Returns list of noun surface forms

#### `pos(text: str) -> List[Tuple[str, str]]`
- Performs part-of-speech tagging
- Returns list of (surface, pos_tag) tuples
- Uses Sejong POS tag set

#### `parse(text: str) -> str`
- Returns MeCab format output
- Tab-separated values: `surface\tPOS,features...`
- Ends with "EOS\n" marker

#### `wakati(text: str) -> List[str]`
- Alias for `morphs()`
- Provided for API compatibility

## Implementation Details

### PyO3 Integration

The implementation uses PyO3 macros for seamless Python integration:

```rust
#[pyclass(name = "Mecab")]
struct PyMecab {
    tokenizer: Tokenizer,
}

#[pymethods]
impl PyMecab {
    #[new]
    #[pyo3(signature = (dicpath=None))]
    fn new(dicpath: Option<&str>) -> PyResult<Self> { ... }

    fn morphs(&self, text: &str) -> PyResult<Vec<String>> { ... }
    // ... other methods
}
```

### Error Handling

- Rust errors are converted to Python exceptions
- `PyRuntimeError` for initialization and analysis errors
- Error messages preserved from Rust layer

### Memory Management

- PyO3 handles memory management automatically
- No manual reference counting required
- Safe across Python GIL boundaries

## Build System

### Cargo Configuration

```toml
[lib]
name = "mecab_ko"
crate-type = ["cdylib"]  # Creates dynamic library for Python

[dependencies]
mecab-ko-core = { path = "../mecab-ko-core" }
pyo3 = { workspace = true }
```

### Maturin Configuration

```toml
[tool.maturin]
module-name = "mecab_ko"
python-source = "python"
features = ["pyo3/extension-module"]
```

### Lint Overrides

Python bindings relax some workspace lints:
- `unwrap_used`: allowed (PyO3 uses unwrap internally)
- `expect_used`: allowed (PyO3 uses expect internally)
- `panic`: allowed (PyO3 may panic on errors)

## Testing Strategy

### Rust Tests

Located in `src/lib.rs` under `#[cfg(test)]`:
- Unit tests for core functionality
- Test tokenizer creation, morphs, nouns, pos, parse methods
- Note: Cannot link as regular test due to cdylib crate type

### Python Tests

Located in `tests/test_mecab.py`:
- Comprehensive API testing
- KoNLPy compatibility verification
- Edge case handling (empty strings, special characters, etc.)
- Module metadata validation

### Test Execution

```bash
# Rust tests (note: linking issues expected for cdylib)
cargo test -p mecab-ko-python

# Python tests (requires installation first)
maturin develop
pytest tests/test_mecab.py -v
```

## Performance Considerations

### Zero-Copy Operations

- String passing uses PyO3's efficient conversion
- Minimal allocation for return values
- Tokenizer state cached in struct

### Release Optimization

Release builds use aggressive optimization:
- LTO enabled
- Single codegen unit
- Strip symbols
- Panic abort

### Benchmark Results

(To be added once full implementation is complete)

## Future Enhancements

### Planned Features

1. **Full Dictionary Support**
   - Load custom dictionaries
   - Multiple dictionary formats
   - Dictionary building tools

2. **Advanced Options**
   - Configurable output formats
   - N-best analysis
   - Lattice visualization

3. **Performance Improvements**
   - Parallel processing for batch operations
   - Streaming API for large texts
   - Memory-mapped dictionaries

4. **Python Packaging**
   - Publish to PyPI
   - Pre-built wheels for major platforms
   - Conda package

### Known Limitations

1. Current tokenizer is a stub implementation
2. Dictionary loading not yet implemented
3. Some advanced MeCab features not exposed
4. No batch processing API

## Compatibility Matrix

### Python Versions

| Version | Status |
|---------|--------|
| 3.8     | ✅ Supported |
| 3.9     | ✅ Supported |
| 3.10    | ✅ Supported |
| 3.11    | ✅ Supported |
| 3.12    | ✅ Supported |

### Platforms

| Platform | Status |
|----------|--------|
| Linux x86_64 | ✅ Tested |
| Linux ARM64 | ⚠️ Untested |
| macOS x86_64 | ⚠️ Untested |
| macOS ARM64 | ⚠️ Untested |
| Windows x64 | ⚠️ Untested |

### KoNLPy Compatibility

The API is designed to be a drop-in replacement for KoNLPy's Mecab:

```python
# KoNLPy
from konlpy.tag import Mecab
mecab = Mecab()

# mecab-ko (our implementation)
from mecab_ko import Mecab
mecab = Mecab()

# Same API
mecab.morphs("안녕하세요")
mecab.nouns("자연어 처리")
mecab.pos("형태소 분석")
```

## Contributing

### Development Setup

```bash
# Clone repository
git clone https://github.com/hephaex/mecab-ko
cd mecab-ko/rust/crates/mecab-ko-python

# Install development dependencies
pip install maturin pytest

# Build in development mode
maturin develop

# Run tests
pytest tests/test_mecab.py -v
```

### Code Style

- Follow Rust style guidelines (rustfmt)
- Follow Python PEP 8 style
- Add docstrings to all public APIs
- Include type hints in Python code

### Testing Requirements

- All new features must include tests
- Maintain test coverage above 90%
- Test edge cases and error conditions
- Verify KoNLPy compatibility

## References

- [PyO3 User Guide](https://pyo3.rs/v0.20/)
- [Maturin Documentation](https://www.maturin.rs/)
- [KoNLPy Documentation](https://konlpy.org/)
- [MeCab Original](https://taku910.github.io/mecab/)

## License

This implementation is dual-licensed under:
- MIT License
- Apache License 2.0

Choose the license that best suits your needs.
