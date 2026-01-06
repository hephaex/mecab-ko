# Quick Start Guide

## 30-Second Installation

```bash
cd /home/mare/mecab-ko/rust/crates/mecab-ko-python
pip install maturin
maturin develop --release
```

## 10-Second Test

```bash
python3 verify_installation.py
```

## 5-Second Usage

```python
from mecab_ko import Mecab
mecab = Mecab()
print(mecab.morphs("안녕하세요"))
```

## Complete Example

```python
#!/usr/bin/env python3
from mecab_ko import Mecab

# Initialize
mecab = Mecab()

# Extract morphemes
print(mecab.morphs("자연어 처리는 재미있다"))
# ['자연어', '처리', '는', '재미', '있', '다']

# Extract nouns
print(mecab.nouns("아버지가방에들어가신다"))
# ['아버지', '가방']

# POS tagging
print(mecab.pos("나는 학생입니다"))
# [('나', 'NP'), ('는', 'JX'), ('학생', 'NNG'), ...]

# MeCab format
print(mecab.parse("형태소"))
# 형태소    NNG,*,*,형태소,*,*,*,*
# EOS
```

## Commands Reference

### Build Commands
```bash
# Development build (fast compile, slower runtime)
cargo build -p mecab-ko-python

# Release build (slow compile, fast runtime)
cargo build -p mecab-ko-python --release

# Install for Python (development)
maturin develop

# Install for Python (release)
maturin develop --release

# Build wheel
maturin build --release
```

### Test Commands
```bash
# Rust tests (note: may have linking issues due to cdylib)
cargo test -p mecab-ko-python

# Python tests (requires installation first)
pytest tests/test_mecab.py -v

# Verification script
python3 verify_installation.py

# Example script
python3 examples/example.py
```

### Clean Commands
```bash
# Clean Rust build artifacts
cargo clean

# Uninstall Python package
pip uninstall mecab-ko
```

## API Quick Reference

| Method | Input | Output | Description |
|--------|-------|--------|-------------|
| `Mecab()` | `dicpath: Optional[str]` | `Mecab` | Create tokenizer |
| `.morphs(text)` | `str` | `List[str]` | Extract morphemes |
| `.nouns(text)` | `str` | `List[str]` | Extract nouns |
| `.pos(text)` | `str` | `List[Tuple[str, str]]` | POS tagging |
| `.parse(text)` | `str` | `str` | MeCab format |
| `.wakati(text)` | `str` | `List[str]` | Alias for morphs |

## File Locations

```
/home/mare/mecab-ko/rust/crates/mecab-ko-python/
├── src/lib.rs                  # Main implementation
├── Cargo.toml                  # Rust config
├── pyproject.toml              # Python config
├── README.md                   # User guide
├── INSTALL.md                  # Installation guide
├── IMPLEMENTATION.md           # Technical docs
├── SUMMARY.md                  # Project summary
├── QUICKSTART.md               # This file
├── examples/example.py         # Usage example
├── tests/test_mecab.py         # Test suite
└── verify_installation.py      # Verification script
```

## Troubleshooting

### Import Error
```bash
# Solution: Install the package
cd /home/mare/mecab-ko/rust/crates/mecab-ko-python
maturin develop --release
```

### Build Error
```bash
# Solution: Update Rust and install dependencies
rustup update
pip install maturin
```

### Test Failure
```bash
# Note: cargo test may fail for cdylib crates
# Use Python tests instead
maturin develop
pytest tests/test_mecab.py -v
```

## Next Steps

1. Read [README.md](README.md) for detailed usage
2. Read [INSTALL.md](INSTALL.md) for installation options
3. Read [IMPLEMENTATION.md](IMPLEMENTATION.md) for technical details
4. Run `python3 examples/example.py` to see examples
5. Run `pytest tests/test_mecab.py -v` to run tests

## Support

- Repository: https://github.com/hephaex/mecab-ko
- Documentation: https://docs.rs/mecab-ko
- Issues: https://github.com/hephaex/mecab-ko/issues
