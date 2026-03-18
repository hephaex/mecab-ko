# MeCab-Ko Python Examples

This directory contains practical examples demonstrating how to use mecab-ko-python for Korean morphological analysis.

## Examples Overview

### 1. Basic Examples

- **example.py** - Simple usage examples of morphs(), nouns(), pos(), and parse()
- **advanced_usage.py** - Advanced features and configuration

### 2. FastAPI Server (fastapi_server.py)

A production-ready REST API server for Korean text analysis.

**Features:**
- `/analyze` - Full morphological analysis
- `/morphs` - Extract morphemes
- `/nouns` - Extract nouns
- `/pos` - Part-of-speech tagging
- Interactive API documentation
- Request/response validation with Pydantic
- Error handling and health checks

**Installation:**
```bash
pip install fastapi uvicorn pydantic
```

**Usage:**
```bash
python fastapi_server.py
```

Access the API at:
- Server: http://localhost:8000
- API docs: http://localhost:8000/docs
- Alternative docs: http://localhost:8000/redoc

**Example requests:**
```bash
# Extract morphemes
curl -X POST "http://localhost:8000/morphs" \
     -H "Content-Type: application/json" \
     -d '{"text": "안녕하세요"}'

# Extract nouns
curl -X POST "http://localhost:8000/nouns" \
     -H "Content-Type: application/json" \
     -d '{"text": "자연어 처리는 재미있습니다"}'

# POS tagging
curl -X POST "http://localhost:8000/pos" \
     -H "Content-Type: application/json" \
     -d '{"text": "나는 학생입니다"}'
```

### 3. Jupyter Tutorial (tutorial.ipynb)

An interactive tutorial demonstrating all features of mecab-ko-python.

**Contents:**
- Setup and initialization
- Basic usage: morphs()
- Noun extraction: nouns()
- Part-of-speech tagging: pos()
- Full analysis: parse()
- Word frequency analysis
- Filtering by POS tags
- Batch processing

**Installation:**
```bash
pip install jupyter
```

**Usage:**
```bash
jupyter notebook tutorial.ipynb
```

## Prerequisites

All examples require mecab-ko-python:

```bash
pip install mecab-ko-python
```

## Quick Start

1. **Run basic example:**
   ```bash
   python example.py
   ```

2. **Start FastAPI server:**
   ```bash
   python fastapi_server.py
   ```

3. **Open Jupyter tutorial:**
   ```bash
   jupyter notebook tutorial.ipynb
   ```

## Use Cases

### Web Applications
Use **fastapi_server.py** as a starting point for building REST APIs that process Korean text.

### Data Analysis
Use **tutorial.ipynb** with pandas and numpy for Korean text analytics.

### Batch Processing
Adapt the batch processing examples for processing large Korean text datasets.

### Keyword Extraction
Use nouns() method for extracting keywords from Korean documents.

## Common POS Tags

| Tag | Korean | Description |
|-----|--------|-------------|
| NNG | 일반 명사 | Common noun |
| NNP | 고유 명사 | Proper noun |
| VV | 동사 | Verb |
| VA | 형용사 | Adjective |
| JX | 보조사 | Auxiliary particle |
| JKS | 주격조사 | Subject particle |
| EF | 종결 어미 | Final ending |

See the MeCab-Ko documentation for a complete list of POS tags.

## Performance Tips

1. **Reuse Mecab instance**: Create once, use many times
2. **Batch processing**: Process multiple texts in a loop
3. **Use appropriate method**:
   - nouns() is faster than filtering pos() results
   - morphs() is faster than pos() if you don't need tags

## Troubleshooting

### Import Error
```python
ImportError: No module named 'mecab_ko'
```
**Solution:** Install mecab-ko-python:
```bash
pip install mecab-ko-python
```

### FastAPI Dependencies
```python
ImportError: No module named 'fastapi'
```
**Solution:** Install required dependencies:
```bash
pip install fastapi uvicorn pydantic
```

### Jupyter Not Found
```bash
jupyter: command not found
```
**Solution:** Install Jupyter:
```bash
pip install jupyter
```

## Contributing

Found a bug or have a suggestion? Please open an issue on GitHub:
https://github.com/hephaex/mecab-ko/issues

## License

These examples are provided under the same license as mecab-ko-python (MIT OR Apache-2.0).

---

**Happy Korean text processing!** 🇰🇷
