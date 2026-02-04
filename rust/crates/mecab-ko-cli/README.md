# mecab-ko-cli

한국어 형태소 분석기 CLI 도구 - Korean Morphological Analyzer Command-Line Tool

## Overview

`mecab-ko-cli` is a high-performance command-line interface for Korean morphological analysis, built on the Rust implementation of MeCab-Ko. It provides fast, accurate tokenization with multiple output formats, user dictionary support, and batch processing capabilities.

## Features

- **Fast Analysis**: Rust-based implementation for optimal performance
- **Multiple Output Formats**: Default MeCab format, Wakati, JSON, CSV, and more
- **User Dictionary Support**: Load custom dictionaries for domain-specific analysis
- **Interactive REPL**: Test and experiment with analysis in real-time
- **Batch Processing**: Process multiple files efficiently
- **Dictionary Management**: Add, remove, and manage dictionary entries on-the-fly
- **Version Control**: Track dictionary changes with automatic versioning and rollback
- **Shell Completions**: Generate completions for Bash, Zsh, Fish, and PowerShell

## Installation

### From Source

```bash
cd rust
cargo build --release
cargo install --path crates/mecab-ko-cli
```

### From crates.io (when published)

```bash
cargo install mecab-ko-cli
```

## Quick Start

### Basic Analysis

```bash
# Analyze text from stdin
echo "안녕하세요" | mecab-ko

# Analyze text directly
mecab-ko "오늘 날씨가 좋습니다"

# Analyze from file
cat input.txt | mecab-ko
```

### Output

```
안녕	NNG
하	XSV
세요	EF
EOS
```

## Usage

```bash
mecab-ko [OPTIONS] [INPUT] [COMMAND]
```

## Options

### Input/Output Options

- `-d, --dicdir <PATH>` - Dictionary directory path
- `-u, --user-dic <PATH>` - User dictionary file (CSV format)
- `-i, --input-file <PATH>` - Input files for batch processing (can be specified multiple times)
- `-o, --output <PATH>` - Output file or directory

### Formatting Options

- `-O, --output-format <FORMAT>` - Output format (default, wakati, dump, pos, json, simple, csv)
- `--separator <SEP>` - Separator for wakati mode (default: space)

### Analysis Options

- `-N, --nbest <N>` - N-best results count (default: 1)
- `-a, --all` - Show all analysis results (debug mode)
- `--no-line` - Disable line-by-line processing

### Other Options

- `-q, --quiet` - Suppress warning messages
- `--repl` - Start interactive REPL mode
- `-h, --help` - Display help information
- `-V, --version` - Display version information

## Output Formats

### Default Format

Standard MeCab output with tab-separated surface form and POS tag:

```bash
mecab-ko "형태소 분석"
```

Output:
```
형태소	NNG
분석	NNG
EOS
```

### Wakati Format

Space-separated tokens only (no POS tags):

```bash
mecab-ko -O wakati "형태소 분석 테스트"
```

Output:
```
형태소 분석 테스트
```

### POS Format

Surface/POS pairs, one per line:

```bash
mecab-ko -O pos "형태소 분석"
```

Output:
```
형태소/NNG
분석/NNG
```

### Simple Format

Space-separated surface/POS pairs:

```bash
mecab-ko -O simple "형태소 분석"
```

Output:
```
형태소/NNG 분석/NNG
```

### Dump Format

Debug information including byte positions:

```bash
mecab-ko -O dump "형태소"
```

Output:
```
[000] surface="형태소" pos=NNG span=[0,9)
```

### JSON Format

Machine-readable JSON array:

```bash
mecab-ko -O json "형태소"
```

Output:
```json
[
  {
    "surface": "형태소",
    "pos": "NNG",
    "start": 0,
    "end": 9,
    "reading": null,
    "lemma": null
  }
]
```

### CSV Format

Comma-separated values with header:

```bash
mecab-ko -O csv "형태소"
```

Output:
```csv
surface,pos,start,end,reading,lemma
형태소,NNG,0,9,,
```

## User Dictionary

### Creating a User Dictionary

Create a CSV file with custom entries:

```csv
surface,pos,cost,reading
카카오톡,NNP,-1000,
아이폰,NNP,-1000,
챗GPT,NNP,-1000,
```

Fields:
- `surface`: The surface form (required)
- `pos`: Part-of-speech tag (required)
- `cost`: Word cost - lower values get higher priority (optional, default: -1000)
- `reading`: Pronunciation or reading (optional)

### Using a User Dictionary

```bash
mecab-ko --user-dic custom.csv "카카오톡으로 메시지 보내기"
```

### Common POS Tags

- `NNG` - General noun (명사)
- `NNP` - Proper noun (고유명사)
- `NNB` - Bound noun (의존명사)
- `VV` - Verb (동사)
- `VA` - Adjective (형용사)
- `MAG` - General adverb (부사)
- `SL` - Foreign language (외국어)
- `SN` - Number (숫자)

## Interactive REPL Mode

Start an interactive session for testing:

```bash
mecab-ko --repl
```

### REPL Commands

- `:help` - Display help information
- `:format` - Change output format
- `:quit` or `:exit` - Exit the REPL
- `Ctrl+D` - Exit the REPL

### REPL Example

```
MeCab-Ko REPL v0.1.0
한국어 형태소 분석기 대화형 모드

mecab-ko> 안녕하세요
안녕	NNG
하	XSV
세요	EF
EOS

mecab-ko> :format
[Format selection menu]

mecab-ko> :quit
종료합니다.
```

## Batch Processing

Process multiple files at once:

```bash
mecab-ko -i file1.txt -i file2.txt -i file3.txt -o output_dir/
```

Each input file generates a corresponding output file with the `.analyzed` extension in the output directory.

### Batch Processing with Different Format

```bash
mecab-ko -O json -i input1.txt -i input2.txt -o results/
```

## Dictionary Management Commands

The CLI includes powerful dictionary management capabilities:

### Dictionary Commands

```bash
mecab-ko dict <SUBCOMMAND>
```

#### Available Subcommands

##### Management Commands

- `reload` - Reload system dictionary
- `add <surface> <pos>` - Add entry to user dictionary
- `remove <surface>` - Remove entry from user dictionary
- `clear` - Clear all user dictionary entries

##### Inspection Commands

- `list` - List user dictionary entries
- `info` - Show dictionary information
- `version` - Display version information

##### Import/Export Commands

- `export <file>` - Export user dictionary to CSV
- `import <file>` - Import user dictionary from CSV

##### Version Control Commands

- `version --history` - Show version history
- `rollback <version>` - Rollback to specific version

### Dictionary Management Examples

#### Adding Custom Words

```bash
# Add a proper noun
mecab-ko dict add "카카오톡" NNP -1000

# Add with reading
mecab-ko dict add "iPhone" NNP -1000 --reading "아이폰"

# Add with custom cost
mecab-ko dict add "API" SL -2000
```

#### Managing Entries

```bash
# List all entries
mecab-ko dict list

# Search for specific entries
mecab-ko dict list --pattern "카카오"

# Remove an entry
mecab-ko dict remove "카카오톡"

# Clear all entries (with confirmation)
mecab-ko dict clear

# Clear without confirmation
mecab-ko dict clear --yes
```

#### Backup and Restore

```bash
# Export user dictionary
mecab-ko dict export my-dictionary.csv

# Import user dictionary
mecab-ko dict import my-dictionary.csv
```

#### Version Management

```bash
# Check current version
mecab-ko dict version

# View version history
mecab-ko dict version --history

# Rollback to previous version
mecab-ko dict rollback 5
```

#### Dictionary Information

```bash
# Show dictionary info
mecab-ko dict info

# Show info for specific dictionary
mecab-ko dict info --dicdir /path/to/dict

# Reload dictionary files
mecab-ko dict reload
```

## Shell Completions

Generate shell completions for your shell:

### Bash

```bash
mecab-ko completions bash > /etc/bash_completion.d/mecab-ko
```

Or for user-only installation:

```bash
mecab-ko completions bash > ~/.local/share/bash-completion/completions/mecab-ko
```

### Zsh

```bash
mecab-ko completions zsh > ~/.zfunc/_mecab-ko
```

Add to `.zshrc`:

```bash
fpath=(~/.zfunc $fpath)
autoload -Uz compinit && compinit
```

### Fish

```bash
mecab-ko completions fish > ~/.config/fish/completions/mecab-ko.fish
```

### PowerShell

```powershell
mecab-ko completions powershell > mecab-ko.ps1
```

## Advanced Examples

### Custom Separator in Wakati Mode

```bash
mecab-ko -O wakati --separator "|" "형태소 분석 테스트"
# Output: 형태소|분석|테스트
```

### Processing with Custom Dictionary and Format

```bash
mecab-ko --user-dic custom.csv -O json "카스텀 단어 테스트" > output.json
```

### Batch Processing with Progress Messages

```bash
mecab-ko -i doc1.txt -i doc2.txt -i doc3.txt -o results/
```

### Quiet Mode for Scripts

```bash
mecab-ko -q --user-dic custom.csv input.txt -o output.txt
```

### File Output

```bash
# Single file input to single file output
mecab-ko "텍스트 분석" -o result.txt

# Stdin to file
cat input.txt | mecab-ko -o output.txt

# File with custom format
mecab-ko -O json input.txt -o output.json
```

## Performance Tips

1. **Use Batch Processing**: Process multiple files with `-i` for better performance
2. **Quiet Mode**: Use `-q` flag in scripts to reduce overhead from progress messages
3. **Simple Formats**: Use `wakati` or `simple` formats for faster processing when full information isn't needed
4. **User Dictionary**: Load user dictionaries once at startup rather than per-analysis
5. **Streaming**: For large files, pipe through stdin for memory-efficient processing

## Error Handling

The CLI returns appropriate exit codes:
- `0`: Success
- `1`: General error (parsing, I/O, dictionary loading, etc.)

All errors include context information to help with debugging.

## Environment Variables

Currently, no environment variables are used. Dictionary paths and options must be specified via command-line arguments.

## Examples by Use Case

### Basic Text Analysis

```bash
# Simple sentence
mecab-ko "서울시는 대한민국의 수도입니다"

# From stdin
echo "형태소 분석기" | mecab-ko

# From file
cat document.txt | mecab-ko
```

### Data Processing Pipeline

```bash
# Extract nouns only (requires jq for JSON processing)
mecab-ko -O json "텍스트 분석" | jq -r '.[] | select(.pos | startswith("NN")) | .surface'

# Count word frequencies
cat corpus.txt | mecab-ko -O wakati | tr ' ' '\n' | sort | uniq -c | sort -rn
```

### Integration with Other Tools

```bash
# Convert to JSON and process
mecab-ko -O json input.txt | jq '.[] | {word: .surface, tag: .pos}'

# CSV for spreadsheet import
mecab-ko -O csv document.txt > analysis.csv
```

## Troubleshooting

### Dictionary Not Found

If you see "Failed to load dictionary" errors:

```bash
# Specify dictionary path explicitly
mecab-ko -d /path/to/mecab-ko-dic "텍스트"

# Or check system dictionary installation
ls /usr/local/lib/mecab/dic/mecab-ko-dic
```

### User Dictionary Format Error

Ensure your CSV file follows the correct format:

```csv
surface,pos,cost,reading
단어1,NNG,-1000,
단어2,NNP,-1500,읽기
```

### REPL Not Starting

Make sure you're using the `--repl` flag:

```bash
mecab-ko --repl
```

## Development

### Building from Source

```bash
cd rust
cargo build --release
```

### Running Tests

```bash
cargo test --package mecab-ko-cli
```

### Running Clippy

```bash
cargo clippy --package mecab-ko-cli -- -D warnings
```

### Formatting Code

```bash
cargo fmt --package mecab-ko-cli
```

## Contributing

Contributions are welcome! Please ensure:

1. All tests pass: `cargo test`
2. Code is formatted: `cargo fmt`
3. No clippy warnings: `cargo clippy -- -D warnings`
4. Add tests for new features
5. Update documentation as needed

## License

MIT or Apache-2.0 (choose one)

## See Also

- [mecab-ko-core](../mecab-ko-core/) - Core tokenization engine
- [mecab-ko-dict](../mecab-ko-dict/) - Dictionary management
- [mecab-ko-hangul](../mecab-ko-hangul/) - Hangul utilities
- [MeCab-Ko Project Homepage](https://github.com/hephaex/mecab-ko)

## Support

For issues, questions, or contributions:
- GitHub Issues: https://github.com/hephaex/mecab-ko/issues
- Repository: https://github.com/hephaex/mecab-ko

## Authors

- hephaex <hephaex@gmail.com>

## Acknowledgments

Based on the original MeCab project by Taku Kudo and the MeCab-Ko Korean adaptation.
