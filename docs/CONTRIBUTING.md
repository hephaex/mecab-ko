# Contributing to MeCab-Ko

> **Project**: MeCab-Ko - Korean Morphological Analyzer
> **Maintainer**: hephaex (hephaex@gmail.com)
> **Repository**: https://github.com/hephaex/mecab-ko

---

Thank you for your interest in contributing to MeCab-Ko! This document provides guidelines for contributing to the project.

## Code of Conduct

This project follows the [Contributor Covenant](https://www.contributor-covenant.org/) code of conduct. By participating, you agree to uphold this code.

---

## How to Report Bugs

If you found a bug:

1. Check [existing issues](https://github.com/hephaex/mecab-ko/issues) to avoid duplicates
2. Create a new issue with the following information:
   - MeCab-Ko version
   - Rust version (`rustc --version`)
   - Operating system
   - Steps to reproduce
   - Expected behavior vs actual behavior
   - Error messages (if any)

## How to Suggest Features

To suggest new features:

1. Open a Discussion or Issue to share your idea
2. Explain the use case and expected benefits
3. If possible, suggest an implementation approach

## How to Add New Words (신조어 추가)

MeCab-Ko welcomes contributions of new words, especially:
- 신조어 (neologisms)
- 브랜드명 (brand names)
- IT/기술 용어
- K-문화 관련 용어
- 인터넷/SNS 용어

### Quick Add (단어 요청)

1. Open an Issue with the **"word-request"** template
2. Provide:
   - 단어 (word)
   - 품사 (POS tag): NNG(일반명사), NNP(고유명사), etc.
   - 읽기 (reading/pronunciation)
   - 사용 예시 (usage example)

### Direct Contribution (직접 기여)

1. Fork the repository
2. Edit `data/user-dict/neologisms.csv`
3. Follow the CSV format:
   ```
   표면형,0,0,0,품사,*,*,*,읽기,원형,읽기,*
   ```
4. Submit a PR with title: `feat(dict): add {word} to neologisms`

### POS Tag Reference (품사 태그)

| Tag | Description | Example |
|-----|-------------|---------|
| NNP | 고유명사 (Proper noun) | 챗GPT, BTS |
| NNG | 일반명사 (Common noun) | 메타버스, 워라밸 |
| VV | 동사 (Verb) | 플렉스하다 |
| VA | 형용사 (Adjective) | - |
| MAG | 일반부사 (Adverb) | - |
| IC | 감탄사 (Interjection) | ㅋㅋㅋ |

---

## Development Setup

### Prerequisites

- Rust toolchain (1.80.0 or later recommended)
- Git
- Python 3.8+ with development headers (for `mecab-ko-python` crate only)

### Getting Started

```bash
# 1. Fork the repository on GitHub

# 2. Clone locally
git clone https://github.com/YOUR_USERNAME/mecab-ko.git
cd mecab-ko

# 3. Navigate to Rust development directory
cd rust

# 4. Set up upstream
git remote add upstream https://github.com/hephaex/mecab-ko.git

# 5. Verify development environment
rustc --version
cargo --version
```

### Development Workflow

```bash
# 1. Sync with latest main
git checkout main
git pull upstream main

# 2. Create a branch
git checkout -b feature/RST-XXX-description

# 3. Develop and test
cargo build --all-features
cargo test
cargo clippy -- -D warnings
cargo fmt

# 4. Commit
git add .
git commit -m "feat(module): description"

# 5. Push and create PR
git push origin feature/RST-XXX-description
```

---

## Code Style Guidelines

### Rust Style Guide

```rust
// Good example
/// Decomposes a Korean syllable into jamo components.
///
/// # Arguments
/// * `syllable` - The Korean syllable to decompose
///
/// # Returns
/// Tuple of (choseong, jungseong, jongseong option)
///
/// # Examples
/// ```
/// use mecab_ko_hangul::decompose;
/// let (cho, jung, jong) = decompose('han').unwrap();
/// assert_eq!(cho, 'h');
/// ```
pub fn decompose(syllable: char) -> Option<(char, char, Option<char>)> {
    // implementation
}

// Bad example - avoid this
pub fn d(c: char) -> Option<(char, char, Option<char>)> {
    // No documentation, unclear naming
}
```

### Key Rules (from CLAUDE.md)

1. **Minimize `unsafe`**: Use `unsafe` only when absolutely necessary
   - Always include `// SAFETY:` comments explaining why it's safe
   - Prefer safe Rust alternatives when available

   ```rust
   // When necessary
   // SAFETY: ptr always points to valid memory and
   // length is guaranteed not to exceed allocated buffer size
   unsafe {
       std::slice::from_raw_parts(ptr, length)
   }
   ```

2. **No `unwrap()` or `expect()` in library code**
   - Use `Result<T, E>` or `Option<T>` for error handling
   - Use `thiserror` for custom error types

   ```rust
   // Good
   pub fn parse(input: &str) -> Result<Token, ParseError> {
       // ...
   }

   // Bad - can panic!
   pub fn parse(input: &str) -> Token {
       input.parse().unwrap()
   }
   ```

3. **Rustdoc required for all public APIs**
   - Every public function, struct, enum, and trait must have documentation comments
   - Include `# Arguments`, `# Returns`, and `# Examples` sections

### Required Checks

All checks must pass before submitting a PR:

```bash
# Build
cargo build --all-features

# Test (including doc tests)
cargo test --all-features

# Clippy (no warnings)
cargo clippy --all-features -- -D warnings

# Formatting
cargo fmt --all -- --check
```

---

## Pull Request Process

### PR Checklist

Before submitting:

- [ ] Link related issue number (`Closes #XXX`)
- [ ] All tests pass
- [ ] No Clippy warnings
- [ ] Formatting applied
- [ ] New features include tests
- [ ] Public APIs are documented
- [ ] CHANGELOG updated (if applicable)
- [ ] Breaking changes are noted

### PR Template

```markdown
## Summary
[Brief description of changes]

## Related Issue
Closes #[issue number]

## Changes
- Change 1
- Change 2

## Testing
Describe how to test the changes

## Checklist
- [ ] Tests pass
- [ ] Clippy clean
- [ ] Documentation updated
```

### Review Process

1. **Automated checks**: CI runs build, tests, and linting
2. **Code review**: Maintainers review code quality
3. **Change requests**: Modifications may be requested
4. **Approval and merge**: Merged via Squash and Merge

---

## Testing Requirements

- All new features must include unit tests
- Bug fixes should include regression tests
- All tests must pass before merging
- Integration tests go in the `tests/` directory
- Benchmarks go in the `benches/` directory

Run tests with:
```bash
cargo test --all-features
```

### Python Bindings (mecab-ko-python)

The `mecab-ko-python` crate requires additional setup:

```bash
# Install Python development headers
# Ubuntu/Debian: sudo apt install python3-dev
# Fedora/RHEL: sudo dnf install python3-devel

# Install maturin
pip install maturin

# Build and test
cd rust/crates/mecab-ko-python
maturin develop && pytest tests/
```

**Note:** `cargo test` alone does not work for `mecab-ko-python` because PyO3 cdylib requires a Python environment. Use `maturin develop` followed by `pytest` instead.

---

## Documentation Requirements

- **All public APIs must have rustdoc comments**
- Include `# Arguments`, `# Returns`, and `# Examples` sections
- Update README files when adding new features
- Keep CHANGELOG.md updated for significant changes
- Breaking changes must be clearly documented

---

## Commit Convention

Use [Conventional Commits](https://www.conventionalcommits.org/) format:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types

| Type | Description |
|------|-------------|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation changes |
| `style` | Formatting (no code behavior change) |
| `refactor` | Refactoring |
| `perf` | Performance improvement |
| `test` | Add/modify tests |
| `build` | Build system changes |
| `ci` | CI configuration changes |
| `chore` | Other changes |

### Scopes

- `hangul`: Hangul utilities
- `dict`: Dictionary related
- `core`: Core algorithms
- `cli`: CLI tool
- `python`: Python bindings
- `wasm`: WASM bindings

### Example

```
feat(hangul): add jamo decomposition function

Implement decompose() function that splits Korean syllables
into individual jamo components (choseong, jungseong, jongseong).

Closes #RST-008
```

---

## Project Structure

```
mecab-ko/
├── rust/                          # Rust implementation
│   ├── crates/
│   │   ├── mecab-ko-core/         # Core algorithms
│   │   ├── mecab-ko-dict/         # Dictionary management
│   │   ├── mecab-ko-hangul/       # Hangul utilities
│   │   ├── mecab-ko-cli/          # CLI tool
│   │   └── mecab-ko-python/       # Python bindings
│   ├── tests/                     # Integration tests
│   ├── benches/                   # Benchmarks
│   └── examples/                  # Example code
├── legacy/                        # Original C/C++ code
├── docs/                          # Documentation
├── CLAUDE.md                      # Project coding rules
├── CONTRIBUTING.md                # This file
├── LICENSE-MIT
└── LICENSE-APACHE
```

---

## Release Process

Version management follows [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking API changes
- **MINOR**: Backward-compatible feature additions
- **PATCH**: Backward-compatible bug fixes

### Pre-release

- `0.x.y`: Initial development stage
- `x.y.z-alpha.N`: Alpha release
- `x.y.z-beta.N`: Beta release
- `x.y.z-rc.N`: Release candidate

---

## Getting Help

- **Issues**: https://github.com/hephaex/mecab-ko/issues
- **Discussions**: https://github.com/hephaex/mecab-ko/discussions
- **Email**: hephaex@gmail.com

---

## License

By contributing to this project, you agree that your contributions will be distributed under the same license as the project (MIT OR Apache-2.0).

---

*Last Updated: 2026-03-02*
*Maintainer: hephaex (hephaex@gmail.com)*
