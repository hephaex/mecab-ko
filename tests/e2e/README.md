# End-to-End Tests

This directory contains end-to-end tests for the mecab-ko project across multiple binding interfaces.

## Test Suites

### CLI Tests (`cli/`)
Tests for the MeCab command-line interface binary.
- Tokenization and morphological analysis
- Command-line arguments and options
- Version and help information

### Python Bindings (`python/`)
Tests for the Python binding interface (mecab-ko-py).
- Module imports and exports
- Core API functionality
- Integration with the underlying Rust engine

### Node.js Bindings (`node/`)
Tests for the Node.js/WASM binding interface (mecab-ko-node).
- Module loading and initialization
- API surface validation
- Cross-platform compatibility

## Running E2E Tests

E2E tests are run via CI workflows defined in `.github/workflows/`.

### Locally (if you have dependencies installed)

```bash
# CLI tests
bash tests/e2e/cli/test_basic.sh

# Python tests (requires Python binding built)
pytest tests/e2e/python/

# Node.js tests (requires Node.js binding built)
node tests/e2e/node/test_basic.mjs
```

## Notes

- Tests assume binaries/modules are built and available in standard locations
- Use environment variables (e.g., `MECAB_BIN`) to override paths if needed
- Each test suite is independent and can be run separately
