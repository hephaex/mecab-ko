# MeCab-Ko Rust Getting Started Guide

A comprehensive guide to getting started with the MeCab-Ko Rust implementation - a high-performance Korean morphological analyzer.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Installation](#installation)
  - [Installing the Rust Library](#installing-the-rust-library)
  - [Installing the CLI](#installing-the-cli)
  - [Installing Python Bindings](#installing-python-bindings)
  - [Installing Node.js Bindings](#installing-nodejs-bindings)
  - [Installing WebAssembly Package](#installing-webassembly-package)
- [Quick Start Examples](#quick-start-examples)
  - [mecab-ko-core (Rust Library)](#mecab-ko-core-rust-library)
  - [mecab-ko-cli (Command Line)](#mecab-ko-cli-command-line)
  - [mecab-ko-python (Python Bindings)](#mecab-ko-python-python-bindings)
  - [mecab-ko-wasm (WebAssembly)](#mecab-ko-wasm-webassembly)
  - [mecab-ko-node (Node.js Bindings)](#mecab-ko-node-nodejs-bindings)
- [Configuration](#configuration)
- [Common Use Cases](#common-use-cases)
- [Troubleshooting](#troubleshooting)

---

## Prerequisites

### Rust Toolchain

MeCab-Ko requires Rust 1.80 or later. Install using rustup:

```bash
# Install Rust (Linux/macOS)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version  # Should be 1.80+
cargo --version
```

For Windows, download and run the [rustup-init.exe](https://rustup.rs/) installer.

### System Dependencies

#### Linux (Ubuntu/Debian)

```bash
# Build essentials
sudo apt update
sudo apt install build-essential pkg-config

# For Python bindings
sudo apt install python3-dev python3-pip

# For WebAssembly (optional)
sudo apt install binaryen  # For wasm-opt
```

#### Linux (Fedora/RHEL)

```bash
sudo dnf install gcc gcc-c++ make pkgconfig
sudo dnf install python3-devel python3-pip
```

#### macOS

```bash
# Install Xcode command line tools
xcode-select --install

# For WebAssembly optimization (optional)
brew install binaryen
```

#### Windows

- Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- Select "C++ build tools" workload
- Install Python from [python.org](https://www.python.org/) if needed

### Dictionary

MeCab-Ko requires the mecab-ko-dic dictionary for morphological analysis.

**Environment Variable:**
```bash
export MECAB_DICDIR=/path/to/mecab-ko-dic
```

**Default Search Paths:**
- `/usr/local/lib/mecab/dic/mecab-ko-dic`
- `/usr/lib/mecab/dic/mecab-ko-dic`
- `/opt/mecab/dic/mecab-ko-dic`
- `./dic/mecab-ko-dic`

---

## Installation

### Installing the Rust Library

#### From crates.io

```toml
# In your Cargo.toml
[dependencies]
mecab-ko = "0.1.0"
```

Or install directly:

```bash
cargo add mecab-ko
```

#### From Source

```bash
# Clone the repository
git clone https://github.com/hephaex/mecab-ko.git
cd mecab-ko/rust

# Build the library
cargo build --release

# Run tests
cargo test

# Build documentation
cargo doc --no-deps --open
```

### Installing the CLI

#### From crates.io

```bash
cargo install mecab-ko-cli
```

#### From Source

```bash
cd mecab-ko/rust
cargo install --path crates/mecab-ko-cli

# Or build without installing
cargo build --release --bin mecab-ko
# Binary will be at: target/release/mecab-ko
```

### Installing Python Bindings

#### From PyPI (Recommended)

```bash
pip install mecab-ko-python
```

Pre-built wheels are available for:
- Linux (x86_64, aarch64)
- macOS (x86_64, Apple Silicon)
- Windows (x86_64)

#### From Source

```bash
# Install maturin
pip install maturin

# Clone and build
git clone https://github.com/hephaex/mecab-ko.git
cd mecab-ko/rust/crates/mecab-ko-python

# Development install
maturin develop --release

# Build wheel for distribution
maturin build --release
```

### Installing Node.js Bindings

#### From npm

```bash
npm install @mecab-ko/node
# or
yarn add @mecab-ko/node
# or
pnpm add @mecab-ko/node
```

#### From Source

```bash
git clone https://github.com/hephaex/mecab-ko.git
cd mecab-ko/rust/crates/mecab-ko-node

# Install dependencies
npm install

# Build native module
npm run build

# Run tests
npm test
```

### Installing WebAssembly Package

#### From npm

```bash
npm install mecab-ko-wasm
# or
yarn add mecab-ko-wasm
```

#### Building from Source

```bash
# Install wasm-pack
cargo install wasm-pack

# Add WASM target
rustup target add wasm32-unknown-unknown

# Navigate to the wasm crate
cd mecab-ko/rust/crates/mecab-ko-wasm

# Build for web browsers
wasm-pack build --target web --release

# Build for Node.js
wasm-pack build --target nodejs --release

# Build for bundlers (webpack, rollup, etc.)
wasm-pack build --target bundler --release
```

---

## Quick Start Examples

### mecab-ko-core (Rust Library)

#### Basic Tokenization

```rust
use mecab_ko::Tokenizer;

fn main() -> Result<(), mecab_ko::Error> {
    // Initialize tokenizer (loads default dictionary)
    let tokenizer = Tokenizer::new()?;

    // Tokenize Korean text
    let tokens = tokenizer.tokenize("안녕하세요, 형태소 분석기입니다.");

    for token in tokens {
        println!("{}\t{}\t{}", token.surface, token.pos, token.reading);
    }

    Ok(())
}

// Output:
// 안녕    NNG    안녕
// 하      XSV    하
// 세요    EF     세요
// ,       SC     ,
// 형태소  NNG    형태소
// 분석기  NNG    분석기
// 입니다  VCP+EF 입니다
// .       SF     .
```

#### Wakati Mode (Space-Separated)

```rust
use mecab_ko::Tokenizer;

fn main() -> Result<(), mecab_ko::Error> {
    let tokenizer = Tokenizer::new()?;

    let words = tokenizer.wakati("한국어 형태소 분석");
    println!("{}", words.join(" "));
    // Output: 한국어 형태소 분석

    Ok(())
}
```

#### Using User Dictionary

```rust
use mecab_ko::{Tokenizer, UserDictionaryBuilder};

fn main() -> Result<(), mecab_ko::Error> {
    // Build user dictionary
    let user_dict = UserDictionaryBuilder::new()
        .default_cost(-1000)
        .add("딥러닝", "NNG")           // Add as general noun
        .add("챗GPT", "NNP")            // Add as proper noun
        .add_with_cost("머신러닝", "NNG", -800)
        .add_full("AI", "SL", -1000, Some("에이아이"))
        .build()?;

    // Create tokenizer with user dictionary
    let tokenizer = Tokenizer::with_user_dict(user_dict)?;

    let tokens = tokenizer.tokenize("딥러닝과 챗GPT는 AI의 발전입니다");
    for token in &tokens {
        println!("{} / {}", token.surface, token.pos);
    }

    Ok(())
}
```

#### Specifying Dictionary Path

```rust
use mecab_ko::Tokenizer;

fn main() -> Result<(), mecab_ko::Error> {
    let tokenizer = Tokenizer::with_dict_path("/usr/local/lib/mecab/dic/mecab-ko-dic")?;
    let tokens = tokenizer.tokenize("사전 경로 지정 예제");

    Ok(())
}
```

### mecab-ko-cli (Command Line)

#### Basic Analysis

```bash
# Analyze text directly
mecab-ko "안녕하세요"

# Output:
# 안녕    NNG
# 하      XSV
# 세요    EF
# EOS

# From stdin
echo "형태소 분석" | mecab-ko

# From file
cat input.txt | mecab-ko
```

#### Output Formats

```bash
# Wakati mode (space-separated morphemes)
mecab-ko -O wakati "형태소 분석 테스트"
# Output: 형태소 분석 테스트

# JSON format
mecab-ko -O json "형태소"
# Output: [{"surface":"형태소","pos":"NNG","start":0,"end":9}]

# CSV format
mecab-ko -O csv "형태소"
# Output:
# surface,pos,start,end,reading,lemma
# 형태소,NNG,0,9,,

# POS format (surface/POS pairs)
mecab-ko -O pos "형태소 분석"
# Output:
# 형태소/NNG
# 분석/NNG

# Simple format
mecab-ko -O simple "형태소 분석"
# Output: 형태소/NNG 분석/NNG
```

#### Using User Dictionary

```bash
# Create user dictionary CSV
cat > custom.csv << EOF
surface,pos,cost,reading
카카오톡,NNP,-1000,
아이폰,NNP,-1000,
챗GPT,NNP,-1000,
EOF

# Use user dictionary
mecab-ko --user-dic custom.csv "카카오톡으로 메시지 보내기"
```

#### Interactive REPL Mode

```bash
mecab-ko --repl

# MeCab-Ko REPL v0.1.0
# 한국어 형태소 분석기 대화형 모드
#
# mecab-ko> 안녕하세요
# 안녕    NNG
# 하      XSV
# 세요    EF
# EOS
#
# mecab-ko> :format
# [Format selection menu]
#
# mecab-ko> :quit
```

#### Batch Processing

```bash
# Process multiple files
mecab-ko -i file1.txt -i file2.txt -i file3.txt -o output_dir/

# With JSON format
mecab-ko -O json -i input1.txt -i input2.txt -o results/
```

#### Dictionary Management

```bash
# Add custom word
mecab-ko dict add "카카오톡" NNP -1000

# List user dictionary entries
mecab-ko dict list

# Export user dictionary
mecab-ko dict export my-dictionary.csv

# Import user dictionary
mecab-ko dict import my-dictionary.csv

# Show dictionary info
mecab-ko dict info
```

### mecab-ko-python (Python Bindings)

#### Basic Usage

```python
from mecab_ko import Mecab

# Create tokenizer instance
mecab = Mecab()

# Extract morphemes
morphemes = mecab.morphs("안녕하세요")
print(morphemes)
# ['안녕', '하', '세요']

# Extract nouns only
nouns = mecab.nouns("아버지가방에들어가신다")
print(nouns)
# ['아버지', '가방']

# Part-of-speech tagging
tagged = mecab.pos("나는 학생입니다")
print(tagged)
# [('나', 'NP'), ('는', 'JX'), ('학생', 'NNG'), ('이', 'VCP'), ('ㅂ니다', 'EF')]

# MeCab format output
result = mecab.parse("안녕하세요")
print(result)
# 안녕    NNG,*,*,안녕,*,*,*,*
# 하      XSV,*,*,하,*,*,*,*
# 세요    EF,*,*,세요,*,*,*,*
# EOS
```

#### With Custom Dictionary Path

```python
from mecab_ko import Mecab

# Specify dictionary path
mecab = Mecab(dicpath="/path/to/mecab-ko-dic")
```

#### Migration from KoNLPy

```python
# Before (KoNLPy)
from konlpy.tag import Mecab
mecab = Mecab()

# After (mecab-ko-python) - Same API!
from mecab_ko import Mecab
mecab = Mecab()

# API is identical
mecab.morphs("안녕하세요")
mecab.nouns("아버지가방에들어가신다")
mecab.pos("나는 학생입니다")
```

### mecab-ko-wasm (WebAssembly)

#### Browser (ES Modules)

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>MeCab-Ko WASM Demo</title>
</head>
<body>
    <script type="module">
        import init, { Mecab } from './pkg/mecab_ko_wasm.js';

        async function analyze() {
            // Initialize WASM module
            await init();

            // Create Mecab instance
            const mecab = new Mecab();

            // Extract morphemes
            const morphs = mecab.morphs("안녕하세요");
            console.log(morphs); // ["안녕", "하", "세요"]

            // Get part-of-speech tags
            const posJson = mecab.pos("형태소 분석");
            const pos = JSON.parse(posJson);
            console.log(pos); // [["형태소", "NNG"], ["분석", "NNG"]]

            // Get detailed token information
            const tokens = mecab.tokenize("한국어 분석기");
            tokens.forEach(token => {
                console.log(`${token.surface}: ${token.pos}`);
            });
        }

        analyze();
    </script>
</body>
</html>
```

#### Node.js

```javascript
const { Mecab } = require('mecab-ko-wasm');

const mecab = new Mecab();

// Extract morphemes
const morphs = mecab.morphs("안녕하세요");
console.log(morphs); // ["안녕", "하", "세요"]

// Extract nouns
const nouns = mecab.nouns("형태소 분석기입니다");
console.log(nouns); // ["형태소", "분석기"]

// Wakati tokenization
const words = mecab.wakati("한국어 처리");
console.log(words); // ["한국어", "처리"]
```

#### TypeScript

```typescript
import init, { Mecab, WasmToken } from 'mecab-ko-wasm';

async function analyze(text: string): Promise<void> {
    await init();

    const mecab = new Mecab();

    // Tokenize with full information
    const tokens: WasmToken[] = mecab.tokenize(text);
    tokens.forEach((token: WasmToken) => {
        console.log({
            surface: token.surface,
            pos: token.pos,
            start: token.start,
            end: token.end,
        });
    });

    // Extract morphemes
    const morphs: string[] = mecab.morphs(text);
    console.log('Morphemes:', morphs);
}

analyze("한국어 형태소 분석");
```

### mecab-ko-node (Node.js Bindings)

#### Basic Usage (TypeScript)

```typescript
import { Mecab } from '@mecab-ko/node';

const mecab = new Mecab();

// Tokenize text
const tokens = mecab.tokenize('형태소 분석기');
console.log(tokens);
// [
//   { surface: '형태소', pos: 'NNG', start: 0, end: 9, ... },
//   { surface: '분석기', pos: 'NNG', start: 12, end: 21, ... }
// ]

// Extract morphemes
const morphs = mecab.morphs('안녕하세요');
console.log(morphs); // ['안녕하세요']

// Extract nouns
const nouns = mecab.nouns('대한민국의 수도는 서울입니다');
console.log(nouns); // ['대한민국', '수도', '서울']

// POS tagging
const pairs = mecab.pos('좋은 아침입니다');
console.log(pairs); // [['좋은', 'VA+ETM'], ['아침', 'NNG'], ['입니다', 'VCP+EF']]

// MeCab format output
const parsed = mecab.parse('형태소 분석');
console.log(parsed);
// 형태소\tNNG,*,*,*,*,*,*,*
// 분석\tNNG,*,*,*,*,*,*,*
// EOS
```

#### CommonJS

```javascript
const { Mecab, getVersion } = require('@mecab-ko/node');

const mecab = new Mecab();
console.log(getVersion()); // "0.1.0"

const nouns = mecab.nouns('서울은 대한민국의 수도입니다');
console.log(nouns); // ['서울', '대한민국', '수도']
```

#### With Custom Dictionary

```typescript
const mecab = Mecab.withDict('/path/to/custom/dict');
```

---

## Configuration

### Dictionary Paths

#### Environment Variable

```bash
export MECAB_DICDIR=/path/to/mecab-ko-dic
```

#### Programmatic Configuration

**Rust:**
```rust
let tokenizer = Tokenizer::with_dict_path("/path/to/dict")?;
```

**Python:**
```python
mecab = Mecab(dicpath="/path/to/dict")
```

**Node.js:**
```typescript
const mecab = Mecab.withDict('/path/to/dict');
```

**CLI:**
```bash
mecab-ko -d /path/to/dict "텍스트"
```

### User Dictionary Format

User dictionaries use CSV format:

```csv
surface,pos,cost,reading
딥러닝,NNG,-1000,딥러닝
챗GPT,NNP,-1000,챗지피티
AI,SL,-1500,에이아이
```

**Fields:**
- `surface` (required): Surface form of the word
- `pos` (required): Part-of-speech tag
- `cost` (optional): Word cost (lower = higher priority, default: -1000)
- `reading` (optional): Pronunciation/reading

### Common POS Tags

| Tag | Korean | English |
|-----|--------|---------|
| NNG | 일반 명사 | General noun |
| NNP | 고유 명사 | Proper noun |
| NNB | 의존 명사 | Dependent noun |
| NP | 대명사 | Pronoun |
| VV | 동사 | Verb |
| VA | 형용사 | Adjective |
| VX | 보조 용언 | Auxiliary verb |
| MAG | 일반 부사 | General adverb |
| JKS | 주격 조사 | Subject particle |
| JKO | 목적격 조사 | Object particle |
| JX | 보조사 | Auxiliary particle |
| EF | 종결 어미 | Final ending |
| SL | 외국어 | Foreign language |
| SN | 숫자 | Number |
| SF | 마침표 | Period |
| SC | 쉼표 | Comma |

---

## Common Use Cases

### Text Preprocessing for NLP

```python
from mecab_ko import Mecab

mecab = Mecab()

def preprocess(text):
    """Extract nouns and verbs for NLP tasks."""
    morphs = []
    for surface, pos in mecab.pos(text):
        if pos.startswith('N') or pos.startswith('V'):
            morphs.append(surface)
    return morphs

# Example
text = "딥러닝 모델을 학습시키는 것은 어렵지만 보람찹니다."
print(preprocess(text))
# ['딥러닝', '모델', '학습', '시키', '것', '어렵', '보람차']
```

### Keyword Extraction

```python
from mecab_ko import Mecab
from collections import Counter

mecab = Mecab()

def extract_keywords(text, top_n=10):
    """Extract top N keywords (nouns) from text."""
    nouns = mecab.nouns(text)
    return Counter(nouns).most_common(top_n)

# Example
article = """
한국어 자연어 처리는 형태소 분석부터 시작합니다.
형태소 분석은 문장을 의미있는 최소 단위로 분리하는 작업입니다.
한국어는 교착어로서 형태소 분석이 특히 중요합니다.
"""
print(extract_keywords(article, 5))
# [('형태소', 3), ('분석', 3), ('한국어', 2), ('자연어', 1), ('처리', 1)]
```

### Search Index Tokenization

```typescript
import { Mecab } from '@mecab-ko/node';

const mecab = new Mecab();

function tokenizeForSearch(text: string): string[] {
    // Use morphs for search indexing
    return mecab.morphs(text);
}

// Example: Index document
const document = "서울특별시는 대한민국의 수도입니다.";
const searchTerms = tokenizeForSearch(document);
console.log(searchTerms);
// ['서울특별시', '는', '대한민국', '의', '수도', '입니다', '.']
```

### Batch Document Processing

```bash
#!/bin/bash
# Process all text files in a directory

INPUT_DIR="./documents"
OUTPUT_DIR="./analyzed"

mkdir -p "$OUTPUT_DIR"

for file in "$INPUT_DIR"/*.txt; do
    filename=$(basename "$file")
    mecab-ko -O json < "$file" > "$OUTPUT_DIR/${filename%.txt}.json"
done
```

### Data Pipeline with JSON Processing

```bash
# Extract nouns from text using jq
mecab-ko -O json "텍스트 분석 예제" | \
    jq -r '.[] | select(.pos | startswith("NN")) | .surface'

# Count word frequencies
cat corpus.txt | mecab-ko -O wakati | tr ' ' '\n' | sort | uniq -c | sort -rn | head -20
```

### Web Application with WASM

```typescript
// React component example
import { useState, useEffect } from 'react';
import init, { Mecab } from 'mecab-ko-wasm';

export function KoreanTokenizer() {
    const [mecab, setMecab] = useState<Mecab | null>(null);
    const [input, setInput] = useState('');
    const [tokens, setTokens] = useState<string[]>([]);

    useEffect(() => {
        async function initMecab() {
            await init();
            setMecab(new Mecab());
        }
        initMecab();
    }, []);

    const handleAnalyze = () => {
        if (mecab && input) {
            setTokens(mecab.morphs(input));
        }
    };

    return (
        <div>
            <input
                type="text"
                value={input}
                onChange={(e) => setInput(e.target.value)}
                placeholder="한국어 텍스트를 입력하세요"
            />
            <button onClick={handleAnalyze}>분석</button>
            <div>
                {tokens.map((token, i) => (
                    <span key={i} className="token">{token}</span>
                ))}
            </div>
        </div>
    );
}
```

---

## Troubleshooting

### Dictionary Not Found

**Symptoms:**
```
Error: Failed to load dictionary
Error: Dictionary not found at default paths
```

**Solutions:**

1. Set the environment variable:
   ```bash
   export MECAB_DICDIR=/path/to/mecab-ko-dic
   ```

2. Specify dictionary path explicitly:
   ```bash
   # CLI
   mecab-ko -d /path/to/mecab-ko-dic "텍스트"
   ```
   ```python
   # Python
   mecab = Mecab(dicpath="/path/to/mecab-ko-dic")
   ```
   ```rust
   // Rust
   let tokenizer = Tokenizer::with_dict_path("/path/to/dict")?;
   ```

3. Check default paths:
   ```bash
   ls /usr/local/lib/mecab/dic/mecab-ko-dic
   ls /usr/lib/mecab/dic/mecab-ko-dic
   ```

### Python Import Error

**Symptoms:**
```
ModuleNotFoundError: No module named 'mecab_ko'
```

**Solutions:**

1. Install the package:
   ```bash
   pip install mecab-ko-python
   ```

2. Build from source:
   ```bash
   cd mecab-ko/rust/crates/mecab-ko-python
   pip install maturin
   maturin develop --release
   ```

3. Check Python environment:
   ```bash
   which python
   pip list | grep mecab
   ```

### Node.js Module Not Found

**Symptoms:**
```
Error: Cannot find module '@mecab-ko/node'
```

**Solutions:**

1. Install the package:
   ```bash
   npm install @mecab-ko/node
   ```

2. Rebuild native module:
   ```bash
   npm rebuild @mecab-ko/node
   ```

3. Check supported platforms:
   - macOS (x64, ARM64)
   - Linux (x64, ARM64, glibc/musl)
   - Windows (x64, ARM64)

### WebAssembly Loading Issues

**Symptoms:**
```
RuntimeError: Failed to instantiate WASM module
TypeError: Failed to fetch WASM file
```

**Solutions:**

1. Make sure WASM is initialized before use:
   ```javascript
   import init, { Mecab } from 'mecab-ko-wasm';

   // MUST await init() first
   await init();
   const mecab = new Mecab();
   ```

2. For bundlers, use the correct target:
   ```bash
   # For webpack/rollup
   wasm-pack build --target bundler

   # For direct browser use
   wasm-pack build --target web
   ```

3. Serve WASM with correct MIME type:
   ```
   Content-Type: application/wasm
   ```

### Rust Compilation Errors

**Symptoms:**
```
error[E0432]: unresolved import
error: could not compile `mecab-ko`
```

**Solutions:**

1. Update Rust toolchain:
   ```bash
   rustup update
   ```

2. Check MSRV (Minimum Supported Rust Version):
   ```bash
   rustc --version  # Should be 1.80+
   ```

3. Clean and rebuild:
   ```bash
   cargo clean
   cargo build
   ```

### User Dictionary Format Error

**Symptoms:**
```
Error: Invalid CSV format
Error: Missing required field
```

**Solutions:**

Ensure CSV follows the correct format:
```csv
surface,pos,cost,reading
단어1,NNG,-1000,
단어2,NNP,-1500,읽기
```

**Common mistakes:**
- Missing header row
- Wrong column order
- Invalid POS tag
- Non-numeric cost value

### Performance Issues

**For large texts:**
- Use streaming/batch processing
- Consider chunking very large inputs
- Use release builds (`--release`)

```bash
# CLI: Use batch mode
mecab-ko -i large_file.txt -o output.txt

# Build with optimizations
cargo build --release
```

**For memory issues:**
```typescript
// Node.js: Chunk large texts
function chunkText(text: string, size: number): string[] {
    const chunks: string[] = [];
    for (let i = 0; i < text.length; i += size) {
        chunks.push(text.slice(i, i + size));
    }
    return chunks;
}

const chunks = chunkText(veryLongText, 1000);
const allTokens = chunks.flatMap(chunk => mecab.tokenize(chunk));
```

---

## Additional Resources

- **GitHub Repository:** https://github.com/hephaex/mecab-ko
- **API Documentation:** https://docs.rs/mecab-ko
- **Issue Tracker:** https://github.com/hephaex/mecab-ko/issues
- **Original MeCab:** https://taku910.github.io/mecab/
- **KoNLPy (Python NLP):** https://konlpy.org/

## License

MeCab-Ko Rust is dual-licensed under MIT and Apache-2.0. Choose the license that best fits your needs.
