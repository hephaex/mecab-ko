# @mecab-ko/node

Node.js bindings for MeCab-Ko Korean morphological analyzer.

High-performance Korean text analysis powered by Rust and N-API.

## Features

- **Fast**: Native Rust implementation with zero-copy operations
- **Type-safe**: Full TypeScript type definitions included
- **Cross-platform**: Supports Windows, macOS, and Linux (x64, ARM64)
- **Thread-safe**: Safe to use in concurrent scenarios
- **Easy to use**: Simple and intuitive API

## Installation

```bash
npm install @mecab-ko/node
# or
yarn add @mecab-ko/node
# or
pnpm add @mecab-ko/node
```

## Requirements

- Node.js >= 18
- No external dependencies (MeCab binary not required)

## Quick Start

```typescript
import { Mecab } from '@mecab-ko/node';

const mecab = new Mecab();

// Tokenize text
const tokens = mecab.tokenize('형태소 분석기');
console.log(tokens);
// Output:
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
// Output:
// 형태소\tNNG,*,*,*,*,*,*,*
// 분석\tNNG,*,*,*,*,*,*,*
// EOS
```

## API Reference

### Class: `Mecab`

The main interface for Korean morphological analysis.

#### Constructor

##### `new Mecab()`

Creates a new Mecab instance with the default dictionary.

```typescript
const mecab = new Mecab();
```

**Throws:** Error if the dictionary cannot be loaded or initialized.

##### `Mecab.withDict(dictPath: string): Mecab`

Creates a new Mecab instance with a custom dictionary path.

```typescript
const mecab = Mecab.withDict('/path/to/custom/dict');
```

**Parameters:**
- `dictPath`: Path to the dictionary directory

**Throws:** Error if the dictionary cannot be loaded.

#### Methods

##### `tokenize(text: string): Token[]`

Tokenizes the input text and returns an array of tokens.

```typescript
const tokens = mecab.tokenize('한국어 형태소 분석');
```

**Parameters:**
- `text`: The text to analyze

**Returns:** An array of `Token` objects.

**Token Interface:**
```typescript
interface Token {
  surface: string;      // The surface form (actual text)
  pos: string;          // Part-of-speech tag
  start: number;        // Start position in bytes
  end: number;          // End position in bytes
  reading?: string;     // Reading (optional)
  lemma?: string;       // Lemma/base form (optional)
}
```

##### `morphs(text: string): string[]`

Extracts morphemes (surface forms) from the input text.

```typescript
const morphs = mecab.morphs('형태소 분석');
// Returns: ['형태소', '분석']
```

**Parameters:**
- `text`: The text to analyze

**Returns:** An array of morpheme strings.

##### `nouns(text: string): string[]`

Extracts nouns from the input text.

Returns only tokens whose POS tag starts with 'NN'.

```typescript
const nouns = mecab.nouns('서울은 대한민국의 수도입니다');
// Returns: ['서울', '대한민국', '수도']
```

**Parameters:**
- `text`: The text to analyze

**Returns:** An array of noun strings.

##### `pos(text: string): string[][]`

Returns part-of-speech tagged pairs.

Each pair consists of `[surface, pos]`.

```typescript
const pairs = mecab.pos('안녕하세요');
// Returns: [['안녕하세요', 'NNG']]
```

**Parameters:**
- `text`: The text to analyze

**Returns:** An array of `[surface, pos]` tuples.

##### `parse(text: string): string`

Parses text and returns MeCab-compatible format string.

The output format follows the original MeCab format:
```
surface\tfeature1,feature2,...
EOS
```

```typescript
const result = mecab.parse('형태소');
console.log(result);
// Output:
// 형태소\tNNG,*,*,*,*,*,*,*
// EOS
```

**Parameters:**
- `text`: The text to analyze

**Returns:** A formatted string in MeCab format.

### Function: `getVersion()`

Returns the version of the mecab-ko-node library.

```typescript
import { getVersion } from '@mecab-ko/node';

console.log(getVersion()); // "0.7.1"
```

**Returns:** The version string.

## POS Tags

This library uses the Sejong POS tag set for Korean:

### Nouns (명사)
- `NNG`: General noun (일반 명사)
- `NNP`: Proper noun (고유 명사)
- `NNB`: Dependent noun (의존 명사)

### Verbs (동사)
- `VV`: Verb (동사)
- `VA`: Adjective (형용사)
- `VX`: Auxiliary verb (보조 용언)
- `VCP`: Copula (긍정 지정사)
- `VCN`: Negative copula (부정 지정사)

### Particles (조사)
- `JKS`: Subject case particle (주격 조사)
- `JKC`: Complement case particle (보격 조사)
- `JKG`: Adnominal case particle (관형격 조사)
- `JKO`: Object case particle (목적격 조사)
- `JKB`: Adverbial case particle (부사격 조사)

For a complete list, see the [Sejong POS tag documentation](https://docs.google.com/spreadsheets/d/1OGAjUvalBuX-oZvZ_-9tEfYD2gQe7hTGsgUpiiBSXI8).

## Performance

This library is built with Rust and uses N-API for optimal performance:

- **Tokenization**: ~1-10ms for typical sentences (50-100 characters)
- **Memory efficient**: Zero-copy operations where possible
- **Thread-safe**: Can be used in multi-threaded environments

## Examples

### Basic Usage

```typescript
import { Mecab } from '@mecab-ko/node';

const mecab = new Mecab();
const text = '아버지가 방에 들어가신다.';

console.log('Tokens:', mecab.tokenize(text));
console.log('Morphs:', mecab.morphs(text));
console.log('Nouns:', mecab.nouns(text));
console.log('POS:', mecab.pos(text));
console.log('MeCab format:\n', mecab.parse(text));
```

### Processing Multiple Texts

```typescript
const mecab = new Mecab();
const texts = [
  '첫 번째 문장',
  '두 번째 문장',
  '세 번째 문장'
];

const results = texts.map(text => ({
  text,
  nouns: mecab.nouns(text),
  morphs: mecab.morphs(text)
}));

console.log(results);
```

### Async Processing

```typescript
import { Mecab } from '@mecab-ko/node';

async function analyzeText(text: string) {
  const mecab = new Mecab();

  // Tokenization is synchronous but fast
  return mecab.tokenize(text);
}

// Can be used in async contexts
const tokens = await analyzeText('한국어 문장');
```

### Error Handling

```typescript
import { Mecab } from '@mecab-ko/node';

try {
  const mecab = new Mecab();
  const tokens = mecab.tokenize('텍스트');
  console.log(tokens);
} catch (error) {
  console.error('Failed to initialize or tokenize:', error);
}
```

## CommonJS vs ESM

This library supports both CommonJS and ES Modules:

### CommonJS

```javascript
const { Mecab, getVersion } = require('@mecab-ko/node');

const mecab = new Mecab();
console.log(getVersion());
```

### ES Modules

```typescript
import { Mecab, getVersion } from '@mecab-ko/node';

const mecab = new Mecab();
console.log(getVersion());
```

## Building from Source

If you want to build the native module from source:

```bash
# Clone the repository
git clone https://github.com/hephaex/mecab-ko.git
cd mecab-ko/rust/crates/mecab-ko-node

# Install dependencies
npm install

# Build the native module
npm run build

# Run tests
npm test
```

### Prerequisites for Building

- Rust toolchain (>= 1.80)
- Node.js (>= 18)
- Cargo

## Troubleshooting

### Module Not Found

If you get a "Cannot find module" error, make sure the native binary is built for your platform:

```bash
npm rebuild @mecab-ko/node
```

### Platform Not Supported

Check if your platform is in the supported list:
- macOS (x64, ARM64)
- Linux (x64, ARM64, with glibc or musl)
- Windows (x64, ARM64)

### Memory Issues

For very large texts, consider splitting them into smaller chunks:

```typescript
function chunkText(text: string, chunkSize: number): string[] {
  const chunks: string[] = [];
  for (let i = 0; i < text.length; i += chunkSize) {
    chunks.push(text.slice(i, i + chunkSize));
  }
  return chunks;
}

const mecab = new Mecab();
const chunks = chunkText(veryLongText, 1000);
const allTokens = chunks.flatMap(chunk => mecab.tokenize(chunk));
```

## Related Projects

- [mecab-ko](https://bitbucket.org/eunjeon/mecab-ko) - Original MeCab-Ko (C++)
- [mecab-ko-dic](https://bitbucket.org/eunjeon/mecab-ko-dic) - Korean dictionary
- [konlpy](https://konlpy.org/) - Python NLP library

## Development Status

This is part of the MeCab-Ko Rust rewrite project. Current status:

- ✅ Basic tokenization API
- ✅ Node.js bindings
- ✅ TypeScript definitions
- ✅ Cross-platform support
- 🚧 Advanced features (N-best, lattice)
- 🚧 Custom dictionary support

## Contributing

Contributions are welcome! Please see the main [MeCab-Ko repository](https://github.com/hephaex/mecab-ko) for guidelines.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

- Original MeCab by Taku Kudo
- MeCab-Ko by Yongwoon Lee and Youngho Yoo
- Eunjeon Project

## Support

- Issues: [GitHub Issues](https://github.com/hephaex/mecab-ko/issues)
- Discussions: [GitHub Discussions](https://github.com/hephaex/mecab-ko/discussions)
