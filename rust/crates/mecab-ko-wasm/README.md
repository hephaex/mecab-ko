# mecab-ko-wasm

WebAssembly bindings for MeCab-Ko, a Korean morphological analyzer.

This package enables Korean morphological analysis in web browsers and Node.js environments through WebAssembly.

## Features

- **Fast**: Compiled to WebAssembly for near-native performance
- **Lightweight**: No external dependencies required in the browser
- **Cross-platform**: Works in both browser and Node.js environments
- **Type-safe**: Full TypeScript type definitions included

## Installation

### Using npm

```bash
npm install mecab-ko-wasm
```

### Using yarn

```bash
yarn add mecab-ko-wasm
```

## Usage

### Browser (ES Modules)

```javascript
import init, { Mecab } from 'mecab-ko-wasm';

async function analyze() {
  // Initialize the WASM module
  await init();

  // Create a Mecab instance
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
```

### Node.js

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

### TypeScript

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

## API Reference

### `Mecab`

The main class for Korean morphological analysis.

#### Constructor

```typescript
new Mecab(): Mecab
```

Creates a new Mecab instance with the default dictionary.

**Throws**: Error if initialization fails

#### Methods

##### `tokenize(text: string): WasmToken[]`

Tokenizes the input text and returns detailed token information.

**Parameters:**
- `text`: Input text to analyze

**Returns**: Array of `WasmToken` objects containing surface form, POS tag, and position information

**Example:**
```javascript
const tokens = mecab.tokenize("안녕하세요");
// [
//   { surface: "안녕", pos: "NNG", start: 0, end: 6, ... },
//   { surface: "하", pos: "XSV", start: 6, end: 9, ... },
//   ...
// ]
```

##### `morphs(text: string): string[]`

Extracts morphemes from the input text.

**Parameters:**
- `text`: Input text to analyze

**Returns**: Array of morpheme strings

**Example:**
```javascript
const morphs = mecab.morphs("안녕하세요");
// ["안녕", "하", "세요"]
```

##### `pos(text: string): string`

Extracts part-of-speech tagged pairs as a JSON string.

**Parameters:**
- `text`: Input text to analyze

**Returns**: JSON string containing an array of `[surface, pos]` pairs

**Example:**
```javascript
const posJson = mecab.pos("안녕하세요");
const pos = JSON.parse(posJson);
// [["안녕", "NNG"], ["하", "XSV"], ["세요", "EP+EF"]]
```

##### `nouns(text: string): string[]`

Extracts only nouns from the input text.

**Parameters:**
- `text`: Input text to analyze

**Returns**: Array of noun strings

**Example:**
```javascript
const nouns = mecab.nouns("형태소 분석기입니다");
// ["형태소", "분석기"]
```

##### `wakati(text: string): string[]`

Performs wakati (space-separated) tokenization.

**Parameters:**
- `text`: Input text to analyze

**Returns**: Array of morpheme strings

**Example:**
```javascript
const words = mecab.wakati("형태소 분석");
// ["형태소", "분석"]
```

### `WasmToken`

Represents a single token with detailed morphological information.

#### Properties

- `surface: string` - The surface form (표면형) of the token
- `pos: string` - Part-of-speech tag (품사 태그)
- `start: number` - Start position in bytes
- `end: number` - End position in bytes
- `reading: string | undefined` - Reading of the token (if available)
- `lemma: string | undefined` - Base form/lemma (if available)

#### Methods

##### `toJSON(): string`

Converts the token to a JSON string.

**Returns**: JSON string representation of the token

## Building from Source

### Prerequisites

- Rust (1.75+)
- wasm-pack

```bash
cargo install wasm-pack
```

### Build

```bash
# Build for browser
wasm-pack build --target web

# Build for Node.js
wasm-pack build --target nodejs

# Build for bundlers (webpack, etc.)
wasm-pack build --target bundler
```

### Development

```bash
# Run tests
wasm-pack test --node

# Run tests in browser (requires Chrome/Firefox)
wasm-pack test --headless --firefox
```

## Part-of-Speech Tags

MeCab-Ko uses the Sejong corpus POS tag set. Common tags include:

- `NNG`: General noun (일반 명사)
- `NNP`: Proper noun (고유 명사)
- `VV`: Verb (동사)
- `VA`: Adjective (형용사)
- `MAG`: General adverb (일반 부사)
- `JKS`: Subjective case particle (주격 조사)
- `JKO`: Objective case particle (목적격 조사)
- `EP`: Pre-final ending (선어말 어미)
- `EF`: Final ending (어말 어미)

For a complete list, see [Sejong POS Tags](https://docs.google.com/spreadsheets/d/1-9blXKjtjeKZqsf4NzHeYJCrr49-nXeRF6D80udfcwY).

## Performance

MeCab-Ko WASM provides near-native performance through WebAssembly compilation:

- **Tokenization**: ~1-2ms for typical sentences (10-20 words)
- **Memory**: ~2-5MB WASM module size (with dictionary)
- **Initialization**: ~10-50ms first load (cached afterwards)

## Browser Compatibility

- Chrome/Edge 57+
- Firefox 52+
- Safari 11+
- Node.js 12+

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

## Related Projects

- [mecab-ko](https://bitbucket.org/eunjeon/mecab-ko) - Original C++ implementation
- [mecab-ko-dic](https://bitbucket.org/eunjeon/mecab-ko-dic) - Korean dictionary for MeCab
- [konlpy](https://konlpy.org/) - Python Korean NLP library

## Acknowledgments

This project is based on MeCab-Ko, originally developed by the Eunjeon project.
