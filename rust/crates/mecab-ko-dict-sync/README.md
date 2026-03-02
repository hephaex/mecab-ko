# mecab-ko-dict-sync

Dictionary synchronization and conversion utilities for MeCab-Ko.

## Features

- **POS Tag Mapping**: Convert POS tags from NIKL (국립국어원) format to MeCab-Ko format
- **Cost Calculation**: Automatic cost calculation based on word frequency and length
- **CSV Export**: Generate MeCab-Ko compatible CSV format for user dictionaries
- **Extensible**: Add custom POS mappings as needed

## Usage

### Basic Conversion

```rust
use mecab_ko_dict_sync::{DictConverter, DictEntry};

let converter = DictConverter::new();

let entry = DictEntry {
    surface: "챗GPT".to_string(),
    pos: "고유명사".to_string(),
    reading: Some("챗지피티".to_string()),
    frequency: Some(1000),
};

let user_entry = converter.convert_entry(&entry)?;
println!("{}", user_entry.to_csv_line());
// Output: 챗GPT,0,0,0,NNP,*,*,*,챗지피티,챗GPT,챗지피티,*
```

### Batch Conversion

```rust
use mecab_ko_dict_sync::{DictConverter, DictEntry};

let converter = DictConverter::new();
let entries = vec![
    DictEntry {
        surface: "챗GPT".to_string(),
        pos: "고유명사".to_string(),
        reading: Some("챗지피티".to_string()),
        frequency: Some(1000),
    },
    DictEntry {
        surface: "메타버스".to_string(),
        pos: "명사".to_string(),
        reading: Some("메타버스".to_string()),
        frequency: Some(500),
    },
];

let csv_lines = converter.convert_to_csv(&entries)?;
for line in csv_lines {
    println!("{line}");
}
```

### Custom POS Mappings

```rust
use mecab_ko_dict_sync::DictConverter;

let mut converter = DictConverter::new();
converter.add_pos_mapping("특수명사".to_string(), "NNG".to_string());

assert_eq!(converter.map_pos("특수명사")?, "NNG");
```

## POS Tag Mapping Table

| NIKL (국립국어원) | MeCab-Ko | Description |
|------------------|----------|-------------|
| 명사 | NNG | 일반명사 |
| 고유명사 | NNP | 고유명사 |
| 동사 | VV | 동사 |
| 형용사 | VA | 형용사 |
| 부사 | MAG | 일반부사 |
| 감탄사 | IC | 감탄사 |
| 관형사 | MM | 관형사 |
| 대명사 | NP | 대명사 |

See the source code for the complete mapping table.

## Cost Calculation

The converter automatically calculates costs based on:

- **High frequency** (≥1000): cost = 0
- **Medium frequency** (100-999): cost = 500
- **Low frequency** (<100): cost = 1000
- **No frequency data**: cost = 500 (default)
- **Long words** (>5 chars): cost -= 100 (prefer longer matches)

Lower cost means higher priority in MeCab's lattice search algorithm.

## CSV Format

The output CSV format follows MeCab-Ko's user dictionary specification:

```
표면형,좌ID,우ID,비용,품사,*,*,*,읽기,원형,읽기,*
```

Example:
```
챗GPT,0,0,0,NNP,*,*,*,챗지피티,챗GPT,챗지피티,*
```

## License

Dual-licensed under MIT OR Apache-2.0.
