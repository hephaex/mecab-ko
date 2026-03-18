# MeCab-Ko Examples

Practical example programs demonstrating mecab-ko usage.

## Prerequisites

Make sure the dictionary is compiled:

```bash
cd rust
cargo build --release -p mecab-ko
```

## Examples

### 1. CLI Analyzer

Interactive REPL for Korean morphological analysis with multiple output formats.

**Features:**
- Interactive REPL mode
- Multiple output formats (table, raw, JSON)
- Token details (surface, POS, reading, lemma, position)

**Usage:**

```bash
# Interactive REPL mode (with dictionary path)
cargo run --example cli_analyzer -p mecab-ko -- \
  --dict-path ../data/mecab-ko-dic-2.1.1-20180720

# Analyze single text
cargo run --example cli_analyzer -p mecab-ko -- \
  --dict-path ../data/mecab-ko-dic-2.1.1-20180720 \
  --text "아버지가방에들어가신다"

# JSON output format
cargo run --example cli_analyzer -p mecab-ko -- \
  --dict-path ../data/mecab-ko-dic-2.1.1-20180720 \
  --format json \
  --text "안녕하세요"

# Raw MeCab format
cargo run --example cli_analyzer -p mecab-ko -- \
  --dict-path ../data/mecab-ko-dic-2.1.1-20180720 \
  --format raw \
  --text "오늘 날씨가 좋습니다"
```

**REPL Commands:**
- `:format <table|raw|json>` - Change output format
- `:quit` or `:q` - Exit REPL
- `:help` - Show help

**Example Output (Table format):**
```
mecab> 아버지가방에들어가신다
┌────────┬──────┬─────────┬─────────┬──────────┬──────────┐
│ Surface│ POS  │ Reading │  Lemma  │ Start    │ End      │
├────────┼──────┼─────────┼─────────┼──────────┼──────────┤
│ 아버지    │ NNG  │ 아버지     │ 아버지     │        0 │        3 │
│ 가      │ JKS  │ 가       │ 가       │        3 │        4 │
│ 방      │ NNG  │ 방       │ 방       │        4 │        5 │
│ 에      │ JKB  │ 에       │ 에       │        5 │        6 │
│ 들어가    │ VV   │ 들어가     │ 들어가     │        6 │        9 │
│ 신다     │ EP+… │ 신다      │ 신다      │        9 │       11 │
└────────┴──────┴─────────┴─────────┴──────────┴──────────┘
Total tokens: 6
```

### 2. Keyword Extractor

Extract keywords from Korean text using noun frequency analysis.

**Features:**
- TF (Term Frequency) based scoring
- Configurable POS tags to include
- Minimum keyword length filter
- Top-N keyword selection

**Usage:**

```bash
# From stdin
echo "인공지능과 머신러닝은 현대 기술의 핵심입니다" | \
  cargo run --example keyword_extractor -p mecab-ko -- \
  --dict-path ../data/mecab-ko-dic-2.1.1-20180720

# From command line
cargo run --example keyword_extractor -p mecab-ko -- \
  --dict-path ../data/mecab-ko-dic-2.1.1-20180720 \
  --text "인공지능과 머신러닝은 현대 기술의 핵심입니다"

# Extract top 10 keywords with minimum length 3
cargo run --example keyword_extractor -p mecab-ko -- \
  --dict-path ../data/mecab-ko-dic-2.1.1-20180720 \
  --top 10 \
  --min-length 3 \
  --text "텍스트..."

# Custom POS tags (include verbs)
cargo run --example keyword_extractor -p mecab-ko -- \
  --dict-path ../data/mecab-ko-dic-2.1.1-20180720 \
  --pos NNG,NNP,VV \
  --text "텍스트..."
```

**Example Output:**
```
Keyword Extraction Results
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Text length: 49 characters
POS tags: NNG, NNP
Min length: 2 characters
Top-N: 5

┌─────┬──────────────────┬───────────┬─────────┐
│ Rank│ Keyword          │ Frequency │ Score   │
├─────┼──────────────────┼───────────┼─────────┤
│   1 │ 기술               │         2 │ 0.22222 │
│   2 │ 인공지능             │         2 │ 0.22222 │
│   3 │ 러닝               │         1 │ 0.11111 │
│   4 │ 머신               │         1 │ 0.11111 │
│   5 │ 발전               │         1 │ 0.11111 │
└─────┴──────────────────┴───────────┴─────────┘
```

## Dictionary Path

The examples require a dictionary path. You can either:

1. **Specify explicitly** (recommended):
   ```bash
   --dict-path /path/to/mecab-ko-dic-2.1.1-20180720
   ```

2. **Set environment variable**:
   ```bash
   export MECAB_DICDIR=/path/to/mecab-ko-dic-2.1.1-20180720
   ```

3. **Install dictionary** to default location:
   - `/usr/local/lib/mecab/dic/mecab-ko-dic`
   - `/usr/lib/mecab/dic/mecab-ko-dic`
   - `/opt/mecab/dic/mecab-ko-dic`

## Common POS Tags

- `NNG`: 일반 명사 (general noun)
- `NNP`: 고유 명사 (proper noun)
- `VV`: 동사 (verb)
- `VA`: 형용사 (adjective)
- `JKS`: 주격 조사 (subject marker)
- `JKB`: 부사격 조사 (adverbial marker)
- `EP`: 선어말 어미 (pre-final ending)
- `EF`: 종결 어미 (final ending)

See [세종 품사 태그](https://docs.rs/mecab-ko/latest/mecab_ko/pos_tag/) for complete list.

## Building Examples

```bash
# Build all examples
cargo build --examples -p mecab-ko

# Run with release optimization
cargo run --release --example cli_analyzer -p mecab-ko
```

## Troubleshooting

**Error: "Failed to initialize tokenizer"**
- Dictionary path is incorrect or missing
- Try specifying `--dict-path` explicitly

**Error: "Dictionary directory not found"**
- Check that dictionary files exist:
  - `sys.dic` (Trie)
  - `matrix.bin` (Connection matrix)
  - `entries.bin` or `entries.csv` (Feature strings)

**No tokens returned**
- Dictionary might not be loaded correctly
- Verify dictionary path with `ls -l /path/to/dict`
