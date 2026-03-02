# MeCab-Ko CLI - Collect Command Usage

## Overview

The `collect` subcommand allows batch collection of dictionary entries from external Korean dictionary APIs using a keyword list file.

## Basic Usage

```bash
mecab collect -k keywords.txt -o output.csv
```

## Prerequisites

### API Key

You need an API key from one of the supported dictionary sources:

1. **OpenDict (우리말샘)** - National Institute of Korean Language
   - Set environment variable: `OPENDICT_API_KEY`
   - Or use `--api-key` option

2. **KrDict (한국어기초사전/표준국어대사전)** - Korean Learners' Dictionary
   - Set environment variable: `KRDICT_API_KEY`
   - Or use `--api-key` option

### Keywords File Format

Create a text file with one keyword per line:

```text
# This is a comment
인공지능
메타버스
블록체인

# Empty lines are ignored
챗GPT
디지털노마드
```

## Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--keywords` | `-k` | Path to keywords file (required) | - |
| `--output` | `-o` | Output CSV file (required) | - |
| `--source` | - | Dictionary source (`opendict` or `krdict`) | `opendict` |
| `--api-key` | - | API key (can use env var instead) | - |
| `--max-per-keyword` | - | Maximum results per keyword | `10` |
| `--delay` | - | Delay between requests (milliseconds) | `100` |
| `--report` | - | Show collection report | `false` |

## Examples

### Basic Collection

```bash
export OPENDICT_API_KEY="your-api-key-here"
mecab collect -k keywords.txt -o output.csv
```

### With Report

```bash
mecab collect -k keywords.txt -o output.csv --report
```

Output:
```
=== 수집 리포트 ===
총 키워드: 8
성공: 8
실패: 0
수집된 항목: 67
중복 제거 후: 65
소요 시간: 0분 12초
출력 파일: output.csv
```

### Using KrDict API

```bash
export KRDICT_API_KEY="your-api-key-here"
mecab collect -k keywords.txt -o output.csv --source krdict
```

### Custom Settings

```bash
mecab collect \
  -k keywords.txt \
  -o output.csv \
  --max-per-keyword 20 \
  --delay 200 \
  --report
```

### With API Key in Command

```bash
mecab collect \
  -k keywords.txt \
  -o output.csv \
  --api-key "your-api-key" \
  --report
```

## Output Format

The output CSV file uses MeCab-Ko user dictionary format:

```csv
표면형,좌ID,우ID,비용,품사,*,*,*,읽기,원형,읽기,*
인공지능,0,0,0,NNG,*,*,*,인공지능,인공지능,인공지능,*
메타버스,0,0,0,NNG,*,*,*,메타버스,메타버스,메타버스,*
```

This CSV can be directly used as a user dictionary with the main `mecab` command:

```bash
mecab --user-dic output.csv "인공지능과 메타버스"
```

## Progress Display

During collection, you'll see a progress bar:

```
[00:00:12] ########################################  8/8 처리 중: 사물인터넷
```

## Error Handling

- Failed keyword searches are logged but don't stop the collection
- The report shows success/failure counts
- Duplicate entries (same surface + POS) are automatically removed

## Rate Limiting

Use `--delay` to avoid hitting API rate limits:

```bash
mecab collect -k keywords.txt -o output.csv --delay 500
```

This adds a 500ms delay between each API request.

## Tips

1. **Start Small**: Test with a small keyword list first
2. **Check API Limits**: Be aware of your API quota
3. **Use Appropriate Delay**: Respect API rate limits
4. **Review Output**: Check the generated CSV before using it
5. **Combine Results**: You can run collect multiple times and merge CSVs

## Troubleshooting

### "API 키가 필요합니다"

Set the environment variable or use `--api-key`:

```bash
export OPENDICT_API_KEY="your-key"
# or
mecab collect -k keywords.txt -o output.csv --api-key "your-key"
```

### "키워드 파일이 비어있습니다"

Make sure your keywords file has at least one non-comment, non-empty line.

### "API 검색 실패"

- Check your API key is valid
- Verify your internet connection
- Ensure you haven't exceeded rate limits
- Try increasing `--delay`

## See Also

- `mecab sync --help` - Single keyword synchronization
- `mecab --user-dic --help` - Using collected dictionaries
- [OpenDict API Documentation](https://opendict.korean.go.kr)
- [KrDict API Documentation](https://krdict.korean.go.kr)
