# Contributing to mecab-ko-python

Thank you for your interest in contributing to mecab-ko-python! This document provides guidelines and instructions for contributing.

## Development Setup

### Prerequisites

1. **Rust Toolchain** (1.80+)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. **Python** (3.8+)
   ```bash
   # Check Python version
   python --version
   ```

3. **Development Tools**
   ```bash
   # Install maturin
   pip install maturin

   # Install development dependencies
   pip install -r requirements-dev.txt
   ```

### Building from Source

```bash
# Clone repository
git clone https://github.com/hephaex/mecab-ko.git
cd mecab-ko/rust/crates/mecab-ko-python

# Build in development mode
maturin develop

# Or build with optimizations
maturin develop --release
```

### Running Tests

```bash
# Run Rust tests
cargo test

# Run Python tests
pytest

# Run tests with coverage
pytest --cov=mecab_ko --cov-report=html

# Run specific test
pytest tests/test_mecab.py::TestMecab::test_morphs -v
```

### Code Quality

```bash
# Format Rust code
cargo fmt

# Lint Rust code
cargo clippy -- -D warnings

# Format Python code
black tests/

# Lint Python code
ruff tests/

# Type check Python code
mypy tests/
```

## Making Changes

### Workflow

1. **Fork the Repository**
   - Fork on GitHub
   - Clone your fork locally

2. **Create a Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make Changes**
   - Write code following the style guidelines
   - Add tests for new features
   - Update documentation

4. **Test Your Changes**
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   pytest
   ```

5. **Commit Changes**
   ```bash
   git add .
   git commit -m "feat: add new feature"
   ```

6. **Push and Create PR**
   ```bash
   git push origin feature/your-feature-name
   ```
   - Create Pull Request on GitHub
   - Fill in the PR template
   - Wait for review

### Commit Message Convention

We follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `style:` Code style changes (formatting, etc.)
- `refactor:` Code refactoring
- `test:` Test additions or modifications
- `chore:` Build process or auxiliary tool changes

Examples:
```
feat: add support for custom dictionary paths
fix: handle empty strings correctly
docs: update API documentation
test: add tests for nouns() method
```

## Coding Standards

### Rust Code

1. **Follow Rust API Guidelines**
   - https://rust-lang.github.io/api-guidelines/

2. **Error Handling**
   - Use `Result` for fallible operations
   - Provide meaningful error messages
   - Convert errors properly using `PyErr`

3. **Documentation**
   - Add rustdoc comments for all public APIs
   - Include examples in documentation
   - Document safety requirements for unsafe code

4. **Safety**
   - Minimize use of `unsafe`
   - Document all unsafe blocks with `// SAFETY:` comments
   - Never use `unwrap()` or `expect()` in library code

Example:
```rust
/// Extract morphemes from text.
///
/// # Arguments
///
/// * `text` - Input text to analyze
///
/// # Returns
///
/// List of morphemes (surface forms)
///
/// # Example
///
/// ```python
/// mecab = Mecab()
/// result = mecab.morphs("안녕하세요")
/// print(result)  # ['안녕', '하', '세요']
/// ```
#[pyo3(text_signature = "($self, text)")]
fn morphs(&self, text: &str) -> PyResult<Vec<String>> {
    Ok(self.tokenizer.morphs(text))
}
```

### Python Code

1. **Type Hints**
   - Add type hints to all functions
   - Use `typing` module for complex types

2. **Documentation**
   - Add docstrings to all functions and classes
   - Follow Google or NumPy docstring style

3. **Testing**
   - Write tests for all new features
   - Aim for high test coverage
   - Include edge cases

Example:
```python
def test_morphs_with_empty_string(self, mecab):
    """Test morphs() with empty string input."""
    result = mecab.morphs("")
    assert isinstance(result, list)
    assert len(result) == 0
```

## Areas for Contribution

### High Priority

- [ ] Performance optimizations
- [ ] Better error messages
- [ ] Dictionary loading improvements
- [ ] More comprehensive tests
- [ ] Benchmark suite

### Documentation

- [ ] API reference improvements
- [ ] Usage examples
- [ ] Migration guide from KoNLPy
- [ ] Performance comparison benchmarks
- [ ] Troubleshooting guide

### Features

- [ ] Support for custom POS tag mappings
- [ ] Batch processing API
- [ ] Async support
- [ ] Dictionary building tools
- [ ] Integration examples

### Testing

- [ ] Integration tests
- [ ] Performance benchmarks
- [ ] Cross-platform testing
- [ ] Memory leak testing
- [ ] Fuzzing

## Pull Request Guidelines

### Before Submitting

- [ ] Code passes all tests: `cargo test && pytest`
- [ ] Code passes linting: `cargo clippy && ruff tests/`
- [ ] Code is formatted: `cargo fmt && black tests/`
- [ ] Documentation is updated
- [ ] CHANGELOG.md is updated (if applicable)
- [ ] Commit messages follow convention

### PR Description

Include:
1. **What** - What does this PR do?
2. **Why** - Why is this change needed?
3. **How** - How does it work?
4. **Testing** - How was it tested?

Example:
```markdown
## Summary
Add support for batch processing multiple texts efficiently.

## Motivation
Processing texts one by one is inefficient for large datasets.

## Implementation
- Add `morphs_batch()` method to process multiple texts
- Utilize rayon for parallel processing
- Maintain API compatibility

## Testing
- Added unit tests for batch processing
- Benchmarked against sequential processing (3x faster)
- Tested with various batch sizes
```

## Review Process

1. **Automated Checks**
   - CI runs tests on multiple platforms
   - Code quality checks run automatically
   - All checks must pass

2. **Manual Review**
   - Maintainers review code quality
   - Check for API compatibility
   - Verify documentation

3. **Feedback**
   - Address review comments
   - Push updates to same branch
   - Request re-review

4. **Merge**
   - Maintainers merge approved PRs
   - Delete feature branch after merge

## Getting Help

- **Questions**: Open a [Discussion](https://github.com/hephaex/mecab-ko/discussions)
- **Bugs**: Open an [Issue](https://github.com/hephaex/mecab-ko/issues)
- **Chat**: Join our community chat (if available)

## Code of Conduct

Please be respectful and constructive in all interactions. We follow the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct).

## License

By contributing, you agree that your contributions will be licensed under the MIT OR Apache-2.0 license.
