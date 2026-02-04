# MeCab-Ko Architecture

This document provides a comprehensive overview of the MeCab-Ko architecture, a Korean morphological analyzer implemented in pure Rust.

## Table of Contents

1. [High-Level System Overview](#high-level-system-overview)
2. [Crate Dependency Graph](#crate-dependency-graph)
3. [Core Components](#core-components)
   - [Tokenizer Pipeline](#tokenizer-pipeline)
   - [Dictionary System](#dictionary-system)
   - [Viterbi Algorithm](#viterbi-algorithm)
   - [Lattice Structure](#lattice-structure)
4. [Data Flow](#data-flow)
5. [Extension Points](#extension-points)
6. [Performance Considerations](#performance-considerations)

---

## High-Level System Overview

MeCab-Ko is a modern, pure-Rust reimplementation of the MeCab-Ko Korean morphological analyzer. The system is designed with a modular architecture that separates concerns across multiple crates.

```
+------------------------------------------------------------------+
|                        MeCab-Ko Architecture                      |
+------------------------------------------------------------------+
|                                                                   |
|  +-------------+   +----------------+   +--------------+          |
|  |   CLI Tool  |   | Python Binding |   | WASM Module  |          |
|  | (mecab-ko-  |   | (PyO3/maturin) |   |  (Browser)   |          |
|  |    cli)     |   |                |   |              |          |
|  +------+------+   +-------+--------+   +------+-------+          |
|         |                  |                   |                  |
|  +------+------------------+-------------------+-------+          |
|  |                                                     |          |
|  |           mecab-ko (Integration Crate)              |          |
|  |                                                     |          |
|  +-----------------------------------------------------+          |
|         |                  |                   |                  |
|  +------+------+   +-------+--------+   +------+-------+          |
|  | mecab-ko-   |   |  mecab-ko-     |   | mecab-ko-    |          |
|  |   core      |   |    dict        |   |   hangul     |          |
|  | (Tokenizer, |   | (Trie, Matrix, |   | (Jamo Utils) |          |
|  |  Viterbi,   |   |  User Dict)    |   |              |          |
|  |  Lattice)   |   |                |   |              |          |
|  +-------------+   +----------------+   +--------------+          |
|         |                  |                                      |
|  +------+------------------+-----------------------+              |
|  |                                                 |              |
|  |            Dictionary Layer (v3.0)              |              |
|  | +------------+  +-------------+  +------------+ |              |
|  | | Core Dict  |  | User Dict   |  | Domain Dict| |              |
|  | | (800K+     |  | (Custom     |  | (IT/Medical| |              |
|  | |  entries)  |  |  entries)   |  |  /Legal)   | |              |
|  | +------------+  +-------------+  +------------+ |              |
|  +---------------------------------------------------+            |
|                                                                   |
+-------------------------------------------------------------------+
```

### Key Design Principles

1. **Pure Rust**: No unsafe code (`#![deny(unsafe_code)]`), ensuring memory safety
2. **Zero-copy Operations**: Memory-mapped dictionaries and borrowed references where possible
3. **Modular Architecture**: Clean separation between core, dictionary, and binding crates
4. **Korean-optimized**: Space penalty handling, Jamo processing, and Jongseong-based rules

---

## Crate Dependency Graph

The project is organized as a Cargo workspace with the following dependency structure:

```
                           +------------------+
                           |    mecab-ko      |
                           | (integration lib)|
                           +--------+---------+
                                    |
          +-------------------------+-------------------------+
          |                         |                         |
          v                         v                         v
+------------------+     +-------------------+    +------------------+
|  mecab-ko-core   |     |   mecab-ko-dict   |    | mecab-ko-hangul  |
| - Tokenizer      |     | - Trie            |    | - Jamo decompose |
| - Lattice        |---->| - Matrix          |    | - Jamo compose   |
| - Viterbi        |     | - UserDictionary  |    | - CharType       |
| - UnknownHandler |     | - SystemDictionary|    | - has_jongseong  |
| - Pool Manager   |     | - HotReload       |    +------------------+
+------------------+     +-------------------+
          |                         |
          |                         |
          v                         v
+------------------+     +-------------------+
|   mecab-ko-cli   |     | mecab-ko-dict-    |
| - Command Line   |     |     builder       |
| - Interactive    |     | - CSV Parser      |
+------------------+     | - Binary Builder  |
                         +-------------------+

+------------------+     +-------------------+     +------------------+
|  mecab-ko-wasm   |     |  mecab-ko-python  |     |  mecab-ko-node   |
| - WebAssembly    |     | - PyO3 bindings   |     | - N-API bindings |
| - Browser/NodeJS |     | - KoNLPy compat   |     | - TypeScript     |
+------------------+     +-------------------+     +------------------+

+------------------+     +-------------------+
| mecab-ko-elastic |     |  mecab-ko-        |
| search           |     |    profiler       |
| - JNI bindings   |     | - Memory tracking |
| - ES Analyzer    |     | - Performance     |
+------------------+     +-------------------+
```

### Crate Descriptions

| Crate | Description |
|-------|-------------|
| `mecab-ko` | Integration crate re-exporting all public APIs |
| `mecab-ko-core` | Core tokenization engine (Lattice, Viterbi, Tokenizer) |
| `mecab-ko-dict` | Dictionary management (Trie, Matrix, User Dictionary) |
| `mecab-ko-hangul` | Korean Hangul utilities (Jamo decomposition/composition) |
| `mecab-ko-cli` | Command-line interface |
| `mecab-ko-dict-builder` | Dictionary compilation tools |
| `mecab-ko-wasm` | WebAssembly bindings |
| `mecab-ko-python` | Python bindings (PyO3) |
| `mecab-ko-node` | Node.js bindings (N-API) |
| `mecab-ko-elasticsearch` | Elasticsearch/Lucene integration |
| `mecab-ko-profiler` | Performance profiling tools |

---

## Core Components

### Tokenizer Pipeline

The `Tokenizer` is the main interface for morphological analysis. It coordinates all components to produce analysis results.

```
                     Tokenizer Pipeline
+------------------------------------------------------------------+
|                                                                   |
|   Input Text: "아버지가방에들어가신다"                              |
|                          |                                        |
|                          v                                        |
|   +--------------------------------------------------+           |
|   |           1. Text Preprocessing                  |           |
|   |  - Remove whitespace (store positions)           |           |
|   |  - Create CharPositions mapping                  |           |
|   |  - Identify space positions for penalty          |           |
|   +--------------------------------------------------+           |
|                          |                                        |
|                          v                                        |
|   +--------------------------------------------------+           |
|   |           2. Lattice Construction                |           |
|   |  - For each position: Common prefix search       |           |
|   |  - Add dictionary nodes to lattice               |           |
|   |  - Handle unknown words (UnknownHandler)         |           |
|   +--------------------------------------------------+           |
|                          |                                        |
|                          v                                        |
|   +--------------------------------------------------+           |
|   |           3. Viterbi Search                      |           |
|   |  - Forward pass: Calculate minimum costs         |           |
|   |  - Apply space penalties                         |           |
|   |  - Backward pass: Extract optimal path           |           |
|   +--------------------------------------------------+           |
|                          |                                        |
|                          v                                        |
|   +--------------------------------------------------+           |
|   |           4. Token Generation                    |           |
|   |  - Convert nodes to Token structs                |           |
|   |  - Parse features (POS, reading, lemma)          |           |
|   |  - Apply normalization (optional)                |           |
|   +--------------------------------------------------+           |
|                          |                                        |
|                          v                                        |
|   Output: [Token{surface: "아버지", pos: "NNG", ...},            |
|            Token{surface: "가", pos: "JKS", ...},                 |
|            Token{surface: "방", pos: "NNG", ...}, ...]            |
|                                                                   |
+------------------------------------------------------------------+
```

#### Tokenizer Components

```rust
pub struct Tokenizer {
    /// System dictionary (Trie + Matrix + Entries)
    dictionary: SystemDictionary,

    /// Unknown word handler
    unknown_handler: UnknownHandler,

    /// Viterbi path searcher
    viterbi_searcher: ViterbiSearcher,

    /// Reusable lattice (optimization)
    lattice: Lattice,

    /// Optional foreign word normalizer
    normalizer: Option<Normalizer>,

    /// Memory pool manager
    pool_manager: PoolManager,
}
```

### Dictionary System

The dictionary system consists of three main components:

#### 1. Double-Array Trie

The Trie provides efficient prefix search for dictionary lookups:

```
        Trie Structure (Double-Array)
+------------------------------------------+
|                                          |
|   Surface Form -> Entry Index Mapping    |
|                                          |
|   "가"     -> [0]                        |
|   "가다"   -> [1]                        |
|   "가방"   -> [2]                        |
|   "가방에" -> [3]                        |
|                                          |
+------------------------------------------+
|                                          |
|   Common Prefix Search:                  |
|   Input: "가방에서"                       |
|   Results: [(0, 3), (2, 6), (3, 9)]      |
|            (index, byte_length)          |
|                                          |
+------------------------------------------+
```

Key features:
- Uses `yada` crate for Double-Array implementation
- Supports exact match and common prefix search
- Compressed file support (zstd)

```rust
pub struct Trie<'a> {
    da: DoubleArray<Cow<'a, [u8]>>,
}

impl<'a> Trie<'a> {
    /// Exact match search
    pub fn exact_match(&self, key: &str) -> Option<u32>;

    /// Common prefix search for all matching prefixes
    pub fn common_prefix_search(&self, text: &str)
        -> impl Iterator<Item = (u32, usize)>;
}
```

#### 2. Connection Cost Matrix

The matrix stores bigram connection costs between context IDs:

```
           Connection Cost Matrix
+------------------------------------------+
|                                          |
|   Matrix[left_id][right_id] = cost       |
|                                          |
|   Dimensions: lsize x rsize              |
|   (typically ~2800 x 2800 for ko-dic)    |
|                                          |
|   BOS -> NNG (noun): low cost (natural)  |
|   NNG -> JKS (case particle): low cost   |
|   VV  -> NNG: high cost (unnatural)      |
|                                          |
+------------------------------------------+
|                                          |
|   Storage Options:                       |
|   - DenseMatrix: Full array (fast)       |
|   - SparseMatrix: HashMap (memory)       |
|   - MmapMatrix: Memory-mapped (large)    |
|                                          |
+------------------------------------------+
```

```rust
pub trait Matrix {
    fn get(&self, right_id: u16, left_id: u16) -> i32;
    fn left_size(&self) -> usize;
    fn right_size(&self) -> usize;
}

pub enum ConnectionMatrix {
    Dense(DenseMatrix),
    Sparse(SparseMatrix),
    Mmap(MmapMatrix),
}
```

#### 3. User Dictionary

User dictionaries allow custom entries to be added:

```rust
pub struct UserDictionary {
    entries: Vec<UserEntry>,
    surface_map: HashMap<String, Vec<usize>>,
    trie_cache: Option<Vec<u8>>,
    default_cost: i16,  // Typically -1000 for priority
}

pub struct UserEntry {
    pub surface: String,
    pub left_id: u16,
    pub right_id: u16,
    pub cost: i16,
    pub pos: String,
    pub reading: Option<String>,
    pub lemma: Option<String>,
}
```

CSV format for user dictionary:
```csv
# User Dictionary
# surface,pos,cost,reading
딥러닝,NNG,-1000,딥러닝
챗GPT,NNP,-1000,챗지피티
```

### Viterbi Algorithm

The Viterbi algorithm finds the optimal (minimum cost) path through the lattice:

```
                    Viterbi Algorithm
+------------------------------------------------------------------+
|                                                                   |
|   Total Cost = Sum(word_cost) + Sum(connection_cost)              |
|              + Sum(space_penalty)                                 |
|                                                                   |
+------------------------------------------------------------------+
|                                                                   |
|   Forward Pass (left to right):                                   |
|                                                                   |
|   BOS -----> [아버지] -----> [가] -----> [방] -----> EOS          |
|    0         +1000+100      +500+50    +800+80     +30            |
|              =1100          =1650      =2530       =2560          |
|                                                                   |
|   At each node, compute:                                          |
|   new_cost = prev_cost + connection_cost + word_cost              |
|            + space_penalty (if applicable)                        |
|                                                                   |
+------------------------------------------------------------------+
|                                                                   |
|   Backward Pass (right to left):                                  |
|                                                                   |
|   EOS <----- [방] <----- [가] <----- [아버지] <----- BOS          |
|                                                                   |
|   Follow prev_node_id links to reconstruct optimal path           |
|                                                                   |
+------------------------------------------------------------------+
```

#### Space Penalty

Korean-specific feature to handle spacing:

```rust
pub struct SpacePenalty {
    penalties: Vec<(u16, i32)>,  // (left_id, penalty)
}

impl SpacePenalty {
    /// Korean default: penalize particles/endings after space
    pub fn korean_default() -> Self {
        // JKS, JKC, JKG... (case particles): penalty 6000
        // EP, EF, EC... (endings): penalty 6000
    }
}
```

This helps avoid mis-segmentation like "아버지가 방에" being parsed as "아버지 가방에".

### Lattice Structure

The Lattice is a Directed Acyclic Graph (DAG) representing all possible segmentations:

```
                    Lattice Structure
+------------------------------------------------------------------+
|                                                                   |
|   Input: "아버지가방에들어가신다"                                  |
|                                                                   |
|   Position: 0   1   2   3   4   5   6   7   8   9   10           |
|   Char:     아  버  지  가  방  에  들  어  가  신  다            |
|                                                                   |
|   BOS ─┬─[아버지]─┬─[가]─┬─[방에]─────────────────────→ EOS       |
|        │         │      │                                         |
|        ├─[아버]──┼─[지가]┤                                        |
|        │         │      ├─[방]─[에]                               |
|        └─[아]────┘      │                                         |
|                         └─[가방]─[에]                             |
|                                                                   |
+------------------------------------------------------------------+
```

#### Node Structure

```rust
pub struct Node {
    pub id: NodeId,
    pub surface: Cow<'static, str>,
    pub start_pos: usize,      // Character position
    pub end_pos: usize,
    pub start_byte: usize,     // Byte position
    pub end_byte: usize,
    pub left_id: u16,          // Left context ID
    pub right_id: u16,         // Right context ID
    pub word_cost: i32,        // Dictionary cost
    pub total_cost: i32,       // Cumulative (Viterbi)
    pub prev_node_id: NodeId,  // Backtrack pointer
    pub node_type: NodeType,   // Known/Unknown/User/BOS/EOS
    pub feature: Cow<'static, str>,
    pub has_space_before: bool,
}

pub enum NodeType {
    Bos,      // Beginning of sentence
    Eos,      // End of sentence
    Known,    // Dictionary word
    Unknown,  // Unknown word
    User,     // User dictionary
}
```

#### Lattice Operations

```rust
impl Lattice {
    /// Create new lattice for text
    pub fn new(text: &str) -> Self;

    /// Add a node candidate
    pub fn add_node(&mut self, builder: NodeBuilder) -> NodeId;

    /// Get nodes ending at position
    pub fn nodes_ending_at(&self, pos: usize) -> impl Iterator<Item = &Node>;

    /// Get nodes starting at position
    pub fn nodes_starting_at(&self, pos: usize) -> impl Iterator<Item = &Node>;

    /// Extract best path after Viterbi
    pub fn best_path(&self) -> Vec<&Node>;

    /// Reset for reuse
    pub fn reset(&mut self, text: &str);
}
```

---

## Data Flow

### Complete Tokenization Flow

```
+------------------------------------------------------------------+
|                    Data Flow Diagram                              |
+------------------------------------------------------------------+
|                                                                   |
|  Input: "아버지가 방에"                                            |
|             |                                                     |
|             v                                                     |
|  +----------------------+                                         |
|  | CharPositions::new() |  Byte <-> Char mapping                  |
|  | SpacePositions::new()|  Track spaces for penalties            |
|  +----------------------+                                         |
|             |                                                     |
|             v                                                     |
|  +----------------------+                                         |
|  | Lattice::new()       |  Create BOS/EOS nodes                  |
|  | Text: "아버지가방에"  |  (spaces removed)                       |
|  +----------------------+                                         |
|             |                                                     |
|             v                                                     |
|  +----------------------+     +-------------------+               |
|  | For each position:   |<--->|  SystemDictionary |               |
|  |   common_prefix_     |     |  - Trie search    |               |
|  |   search()           |     |  - Get entries    |               |
|  +----------------------+     +-------------------+               |
|             |                                                     |
|             v                                                     |
|  +----------------------+     +-------------------+               |
|  | UnknownHandler:      |<--->|  Character Rules  |               |
|  |   handle_unknown()   |     |  - Char types     |               |
|  +----------------------+     +-------------------+               |
|             |                                                     |
|             v                                                     |
|  +----------------------+     +-------------------+               |
|  | ViterbiSearcher:     |<--->|  ConnectionMatrix |               |
|  |   forward_pass()     |     |  - Bigram costs   |               |
|  |   backward_pass()    |     +-------------------+               |
|  +----------------------+                                         |
|             |                         |                           |
|             v                         v                           |
|  +----------------------+     +-------------------+               |
|  | Lattice::best_path() |     |   SpacePenalty    |               |
|  +----------------------+     |  - Korean rules   |               |
|             |                 +-------------------+               |
|             v                                                     |
|  +----------------------+                                         |
|  | Token::from_node()   |  Convert nodes to tokens               |
|  +----------------------+                                         |
|             |                                                     |
|             v                                                     |
|  Output: Vec<Token>                                               |
|  [Token{아버지, NNG}, Token{가, JKS}, Token{방, NNG}, ...]       |
|                                                                   |
+------------------------------------------------------------------+
```

---

## Extension Points

MeCab-Ko provides multiple extension points for integration:

### 1. Python Bindings (PyO3)

```python
from mecab_ko import Mecab

mecab = Mecab()
# Or with custom dictionary
mecab = Mecab(dicpath="/path/to/dict")

# KoNLPy-compatible API
morphs = mecab.morphs("안녕하세요")    # ['안녕', '하', '세요']
nouns = mecab.nouns("오늘 날씨")       # ['오늘', '날씨']
pos = mecab.pos("나는 학생입니다")     # [('나', 'NP'), ...]
result = mecab.parse("테스트")         # MeCab format
```

### 2. WebAssembly (WASM)

```javascript
import { Mecab } from 'mecab-ko-wasm';

const mecab = new Mecab();

// Tokenization
const tokens = mecab.tokenize("안녕하세요");
tokens.forEach(token => {
    console.log(`${token.surface}: ${token.pos}`);
});

// Simple extraction
const morphs = mecab.morphs("형태소 분석");
const nouns = mecab.nouns("자연어 처리");
```

### 3. Node.js (N-API)

```javascript
const { Mecab } = require('@mecab-ko/node');

const mecab = new Mecab();
// Or with custom dictionary
const mecab = Mecab.withDict('/path/to/dict');

const tokens = mecab.tokenize('형태소 분석기');
const morphs = mecab.morphs('한국어');
const nouns = mecab.nouns('대한민국');
const pos = mecab.pos('안녕하세요');
const parsed = mecab.parse('형태소');
```

### 4. Elasticsearch Integration

```java
// JNI-based integration for Elasticsearch/Lucene
// Uses mecab-ko-elasticsearch crate
```

### Binding Architecture

```
+------------------------------------------------------------------+
|                    Binding Architecture                           |
+------------------------------------------------------------------+
|                                                                   |
|   +-------------+     +--------------+     +---------------+      |
|   |   Python    |     |   JavaScript |     |    Node.js    |      |
|   +------+------+     +------+-------+     +-------+-------+      |
|          |                   |                     |              |
|          v                   v                     v              |
|   +------+------+     +------+-------+     +-------+-------+      |
|   |    PyO3     |     | wasm-bindgen |     |    napi-rs    |      |
|   +------+------+     +------+-------+     +-------+-------+      |
|          |                   |                     |              |
|          +-------------------+---------------------+              |
|                              |                                    |
|                              v                                    |
|                    +---------+---------+                          |
|                    |   mecab-ko-core   |                          |
|                    |     (Rust)        |                          |
|                    +-------------------+                          |
|                                                                   |
+------------------------------------------------------------------+
```

---

## Performance Considerations

### 1. Memory Optimization

#### Object Pooling

```rust
pub struct PoolManager {
    token_pool: TokenPool,
    node_vec_pool: NodeVecPool,
    string_interner: StringInterner,
}

impl PoolManager {
    /// Get statistics
    pub fn stats(&self) -> PoolStats;

    /// Clear all pools
    pub fn clear(&self);
}
```

- Token and Node vector reuse
- String interning for duplicate surface forms
- Lattice reuse across tokenization calls

#### Memory-Mapped Dictionaries

```rust
pub struct MmapMatrix {
    lsize: usize,
    rsize: usize,
    mmap: memmap2::Mmap,  // OS-level memory mapping
}
```

- Shared memory across processes
- Lazy loading (pages loaded on access)
- Reduced memory footprint

### 2. SIMD Optimization (Optional)

When the `simd` feature is enabled:

```rust
#[cfg(feature = "simd")]
pub mod simd {
    /// SIMD-accelerated matrix lookup
    pub fn simd_forward_pass_position(...);

    /// SIMD-accelerated node cost update
    pub fn simd_update_node_cost(...);
}
```

### 3. Compression

Dictionary files support zstd compression:

```rust
// Compressed file loading
Trie::from_compressed_file("sys.dic.zst")?;
DenseMatrix::from_compressed_file("matrix.bin.zst")?;

// Compression level configurable (1-22)
TrieBuilder::save_to_compressed_file(bytes, path, 3)?;
```

### 4. Performance Benchmarks

The project includes comprehensive benchmarks:

```bash
# Run benchmarks
cd rust
cargo bench

# Available benchmarks:
# - tokenizer_bench: End-to-end tokenization
# - trie_bench: Dictionary lookup
# - matrix_bench: Connection cost lookup
# - viterbi_bench: Path search
# - memory_bench: Memory usage
```

### 5. Batch Processing

For high-throughput scenarios:

```rust
// Parallel processing with rayon
use rayon::prelude::*;

let texts: Vec<&str> = load_texts();
let results: Vec<Vec<Token>> = texts
    .par_iter()
    .map(|text| tokenizer.tokenize(text))
    .collect();
```

### 6. Hot Reload

Dictionary hot-reload without restart:

```rust
use mecab_ko_dict::{HotReloadDictionary, FileWatcher};

let dict = HotReloadDictionary::new(path)?;

// Watch for changes
let watcher = FileWatcher::new()?;
watcher.watch(path)?;

// Apply delta updates
dict.apply_delta(delta_update)?;
```

---

## File Structure

```
rust/crates/
+-- mecab-ko/                    # Integration crate
|   +-- src/lib.rs               # Re-exports
|
+-- mecab-ko-core/               # Core engine
|   +-- src/
|       +-- lib.rs               # Module exports
|       +-- tokenizer.rs         # Main Tokenizer
|       +-- lattice.rs           # Lattice DAG
|       +-- viterbi/mod.rs       # Viterbi algorithm
|       +-- unknown.rs           # Unknown word handler
|       +-- pos_tag.rs           # POS tag definitions
|       +-- pool.rs              # Memory pooling
|       +-- normalizer.rs        # Text normalization
|
+-- mecab-ko-dict/               # Dictionary management
|   +-- src/
|       +-- lib.rs               # Module exports
|       +-- trie.rs              # Double-Array Trie
|       +-- matrix/mod.rs        # Connection matrix
|       +-- dictionary.rs        # SystemDictionary
|       +-- user_dict.rs         # User dictionary
|       +-- hot_reload.rs        # Hot reload support
|
+-- mecab-ko-hangul/             # Korean utilities
|   +-- src/lib.rs               # Jamo processing
|
+-- mecab-ko-wasm/               # WASM bindings
+-- mecab-ko-python/             # Python bindings
+-- mecab-ko-node/               # Node.js bindings
+-- mecab-ko-elasticsearch/      # ES integration
+-- mecab-ko-cli/                # Command line tool
+-- mecab-ko-profiler/           # Profiling tools
+-- mecab-ko-dict-builder/       # Dictionary builder
+-- mecab-ko-dict-validator/     # Dictionary validation
+-- benchmarks/                  # Performance benchmarks
```

---

## References

- [MeCab Original Documentation](https://taku910.github.io/mecab/)
- [mecab-ko-dic Tag System](https://docs.google.com/spreadsheets/d/1-9blXKjtjeKZqsf4NzHeYJCrr49-nXeRF6D80udfcwY)
- [yada (Double-Array Trie)](https://crates.io/crates/yada)
- [fst (Finite State Transducer)](https://crates.io/crates/fst)

---

*Document Version: 1.0*
*Last Updated: 2024*
